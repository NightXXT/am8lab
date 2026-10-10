//! Signed Windows updates. The webview cannot supply URLs, paths or installer arguments.
use crate::{protocol::Result, updates::{UpdateInfo, Updates, permitted_download}};
use std::{future::Future, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, task::Poll, time::{Duration, Instant}};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const MANIFEST_URL: &str = "https://raw.githubusercontent.com/NightXXT/am8lab/main/update.json";

#[derive(Default)]
pub struct UpdateFlow {
    busy: Arc<AtomicBool>,
    pending: Mutex<Option<Pending>>,
}
struct Pending { candidate: Update, bytes: Option<Vec<u8>> }
pub struct Lease(Arc<AtomicBool>);
impl Drop for Lease { fn drop(&mut self) { self.0.store(false, Ordering::SeqCst); } }
impl UpdateFlow {
    pub fn lease(&self) -> Result<Lease> {
        if self.busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err("Uma atualização já está em andamento.".into());
        }
        Ok(Lease(self.busy.clone()))
    }

    pub async fn check(&self, app: AppHandle, updates: Arc<Updates>) -> Result<UpdateInfo> {
        let _lease = self.lease()?;
        *self.pending.lock().map_err(|_| "Atualização indisponível")? = None;
        let worker = updates.clone();
        let info = tauri::async_runtime::spawn_blocking(move || worker.check()).await.map_err(|e| e.to_string())??;
        if info.status != crate::updates::UpdateStatus::Available { return Ok(info); }
        let result = async {
            let updater = app.updater_builder().endpoints(vec![MANIFEST_URL.parse().map_err(|_| "Endpoint inválido")?])
                .map_err(|e| e.to_string())?.timeout(Duration::from_secs(90)).build().map_err(|e| e.to_string())?;
            let candidate = updater.check().await.map_err(|e| format!("Manifesto de atualização indisponível: {e}"))?
                .ok_or("A versão publicada ainda não tem uma atualização assinada disponível.")?;
            validate_candidate(&info, &candidate.version, candidate.download_url.as_str(), &candidate.signature)?;
            *self.pending.lock().map_err(|_| "Atualização indisponível")? = Some(Pending { candidate, bytes: None });
            updates.set_direct_state(true, false)
        }.await;
        if let Err(ref error) = result { updates.set_direct_error(error.clone())?; }
        result
    }

    pub async fn download(&self, app: AppHandle, updates: Arc<Updates>) -> Result<UpdateInfo> {
        let _lease = self.lease()?;
        let candidate = self.pending.lock().map_err(|_| "Atualização indisponível")?.as_ref()
            .ok_or("Verifique as atualizações antes de baixar.")?.candidate.clone();
        let info = updates.info()?;
        validate_candidate(&info, &candidate.version, candidate.download_url.as_str(), &candidate.signature)?;
        let expected = info.download_size.ok_or("Tamanho do instalador indisponível")?;
        let exceeded = AtomicBool::new(false);
        let mut received = 0u64;
        let mut last = Instant::now() - Duration::from_secs(1);
        let mut download = std::pin::pin!(candidate.download(|chunk, total| {
            received = received.saturating_add(chunk as u64);
            if !within_download_limit(received, total, expected) { exceeded.store(true, Ordering::SeqCst); }
            if last.elapsed() >= Duration::from_millis(100) || received == expected {
                let _ = app.emit("update-progress", serde_json::json!({"phase":"downloading","received":received.min(expected),"total":expected}));
                last = Instant::now();
            }
        }, || {}));
        // Dropping the future cancels the request. A mismatching size can never become installable.
        let bytes = std::future::poll_fn(|cx| {
            let result = download.as_mut().poll(cx);
            if exceeded.load(Ordering::SeqCst) { return Poll::Ready(Err("O download excedeu o tamanho publicado.".to_string())); }
            result.map(|r| r.map_err(|e| format!("Falha ao baixar ou verificar a assinatura: {e}")))
        }).await?;
        if bytes.len() as u64 != expected { return Err("O tamanho recebido difere do instalador publicado.".into()); }
        self.pending.lock().map_err(|_| "Atualização indisponível")?.as_mut().ok_or("Atualização indisponível")?.bytes = Some(bytes);
        let _ = app.emit("update-progress", serde_json::json!({"phase":"verified","received":expected,"total":expected}));
        updates.set_direct_state(true, true)
    }

    /// Caller must hold a lease and restore the microphone before invoking install.
    pub fn installer(&self) -> Result<(Update, Vec<u8>)> {
        let pending = self.pending.lock().map_err(|_| "Atualização indisponível")?;
        let pending = pending.as_ref().ok_or("Nenhum download disponível")?;
        Ok((pending.candidate.clone(), pending.bytes.as_ref().ok_or("Baixe e verifique o instalador primeiro.")?.clone()))
    }
}

pub fn validate_candidate(info: &UpdateInfo, version: &str, url: &str, signature: &str) -> Result<()> {
    let expected = permitted_download(info)?;
    if info.latest_version.as_deref() != Some(version) || expected != url || signature.trim().is_empty() || signature.len() > 4096 {
        return Err("O manifesto assinado não corresponde à versão e ao instalador da Release.".into());
    }
    Ok(())
}
fn within_download_limit(received: u64, total: Option<u64>, expected: u64) -> bool {
    received <= expected && total.is_none_or(|n| n == expected)
}

pub fn restore_then_install(restore: impl FnOnce() -> Result<()>, install: impl FnOnce() -> Result<()>) -> Result<()> {
    restore()?;
    install()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updates::select_release;
    fn info() -> UpdateInfo {
        select_release(br#"[{"draft":false,"prerelease":true,"published_at":"2026-10-10T00:00:00Z","tag_name":"v0.7.2","html_url":"https://github.com/NightXXT/am8lab/releases/tag/v0.7.2","assets":[{"name":"AM8-Lab-Setup-v0.7.2.exe","size":4000000,"state":"uploaded","browser_download_url":"https://github.com/NightXXT/am8lab/releases/download/v0.7.2/AM8-Lab-Setup-v0.7.2.exe"}]}]"#, "0.7.0").unwrap()
    }
    #[test] fn manifest_must_match_release() {
        let i = info(); let url = i.download_url.as_deref().unwrap();
        assert!(validate_candidate(&i, "0.7.2", url, "signature").is_ok());
        for (v,u,s) in [("0.7.1",url,"sig"),("0.7.2","https://evil.example/setup.exe","sig"),("0.7.2",url,"")] {
            assert!(validate_candidate(&i,v,u,s).is_err());
        }
        assert!(validate_candidate(&i,"0.7.2",url,&"s".repeat(4097)).is_err());
    }
    #[test] fn restoration_failure_never_starts_installer() {
        let started = AtomicBool::new(false);
        assert!(restore_then_install(|| Err("USB desconectado".into()), || { started.store(true,Ordering::SeqCst); Ok(()) }).is_err());
        assert!(!started.load(Ordering::SeqCst));
    }
    #[test] fn restores_before_install_and_preserves_install_error() {
        let restored = AtomicBool::new(false);
        let r = restore_then_install(|| {restored.store(true,Ordering::SeqCst);Ok(())}, || {
            assert!(restored.load(Ordering::SeqCst)); Err("Não iniciou".into())
        });
        assert_eq!(r.unwrap_err(),"Não iniciou");
    }
    #[test] fn update_lease_prevents_overlap_and_releases_after_failure() {
        let f = UpdateFlow::default(); let lease=f.lease().unwrap();
        assert!(f.lease().is_err()); drop(lease); assert!(f.lease().is_ok());
    }
    #[test] fn missing_or_unverified_download_cannot_install() {
        assert!(UpdateFlow::default().installer().is_err());
        let i=info(); assert!(!i.direct_update_ready && !i.downloaded);
    }
    #[test] fn download_size_rejects_truncation_metadata_and_oversized_stream() {
        assert!(within_download_limit(50,None,100));
        assert!(within_download_limit(100,Some(100),100));
        assert!(!within_download_limit(101,None,100));
        assert!(!within_download_limit(50,Some(200),100));
    }
}
