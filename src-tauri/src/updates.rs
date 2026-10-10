//! Manual release discovery. Signed download and installation live in in_app_updates.
use crate::protocol::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const REPOSITORY_URL: &str = "https://github.com/NightXXT/am8lab";
pub const RELEASES_URL: &str = "https://github.com/NightXXT/am8lab/releases";
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_INSTALLER_BYTES: u64 = 500 * 1024 * 1024;
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStatus { Idle, Available, Current, Ahead, NoRelease, Unavailable, Error }

#[derive(Clone, Debug, Serialize)]
pub struct UpdateInfo {
    pub direct_update_ready: bool,
    pub downloaded: bool,
    pub current_version: String,
    pub repository_url: String,
    pub releases_url: String,
    pub status: UpdateStatus,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub download_name: Option<String>,
    pub download_size: Option<u64>,
    pub release_url: Option<String>,
    pub release_name: Option<String>,
    pub release_notes: Option<String>,
    pub prerelease: bool,
    pub published_at: Option<String>,
    pub checked_at: Option<u64>,
    pub error: Option<String>,
}

impl UpdateInfo {
    fn idle(current: &str) -> Self {
        Self {
            direct_update_ready: false, downloaded: false,
            current_version: current.into(), repository_url: REPOSITORY_URL.into(),
            releases_url: RELEASES_URL.into(), status: UpdateStatus::Idle,
            latest_version: None, download_url: None, download_name: None, download_size: None,
            release_url: None, release_name: None, release_notes: None,
            prerelease: false, published_at: None, checked_at: None, error: None,
        }
    }
}

/// The network worker never holds the microphone session or a cache mutex.
/// A timed-out worker may finish later, but cannot replace the cached result.
pub struct Updates {
    cached: Mutex<UpdateInfo>,
    checking: Arc<AtomicBool>,
}
struct CheckLease(Arc<AtomicBool>);
impl Drop for CheckLease { fn drop(&mut self) { self.0.store(false, Ordering::SeqCst); } }

impl Default for Updates { fn default() -> Self { Self::new() } }
impl Updates {
    pub fn new() -> Self {
        Self { cached: Mutex::new(UpdateInfo::idle(env!("CARGO_PKG_VERSION"))), checking: Arc::new(AtomicBool::new(false)) }
    }

    pub fn info(&self) -> Result<UpdateInfo> {
        Ok(self.cached.lock().map_err(|_| "Estado das atualizações indisponível")?.clone())
    }

    pub fn check(&self) -> Result<UpdateInfo> {
        if self.checking.swap(true, Ordering::SeqCst) {
            return self.record_error("Uma consulta de atualizações ainda está em andamento. Tente novamente em alguns segundos.".into());
        }
        // Keep the lease until both caller and worker finish: neither overlapping
        // checks nor late cache writes are possible after a response timeout.
        let _lease = Arc::new(CheckLease(self.checking.clone()));
        let worker_lease = _lease.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        // Only one worker can exist, including after a response deadline expires.
        if let Err(e) = std::thread::Builder::new().name("am8-release-check".into()).spawn(move || {
            let _lease = worker_lease;
            let response = fetch_releases();
            let _ = sender.send(response);
        }) {
            return self.record_error(format!("Não foi possível iniciar a consulta: {e}"));
        }
        let response = receiver.recv_timeout(CHECK_TIMEOUT)
            .map_err(|_| "A consulta ao GitHub demorou mais de 15 segundos. Confira sua conexão e tente novamente.".to_string())
            .and_then(|r| r)
            .and_then(|bytes| select_release(&bytes, env!("CARGO_PKG_VERSION")));
        match response {
            Ok(mut info) => {
                info.checked_at = Some(now_millis());
                *self.cached.lock().map_err(|_| "Estado das atualizações indisponível")? = info.clone();
                Ok(info)
            }
            Err(e) => self.record_error(e),
        }
    }

    fn record_error(&self, error: String) -> Result<UpdateInfo> {
        let mut info = UpdateInfo::idle(env!("CARGO_PKG_VERSION"));
        info.status = UpdateStatus::Error;
        info.checked_at = Some(now_millis());
        info.error = Some(error.clone());
        *self.cached.lock().map_err(|_| "Estado das atualizações indisponível")? = info;
        Err(error)
    }

    /// No URL supplied by JavaScript is accepted here.
    pub fn download_destination(&self) -> Result<String> {
        if self.checking.load(Ordering::SeqCst) { return Err("Aguarde a consulta de atualizações terminar antes de baixar.".into()); }
        let info = self.info()?;
        let url = permitted_download(&info)?;
        Ok(url)
    }

    pub fn set_direct_state(&self, ready: bool, downloaded: bool) -> Result<UpdateInfo> {
        let mut info = self.cached.lock().map_err(|_| "Estado das atualizações indisponível")?;
        info.direct_update_ready = ready; info.downloaded = ready && downloaded; info.error = None;
        Ok(info.clone())
    }
    pub fn set_direct_error(&self, error: String) -> Result<()> {
        let mut info = self.cached.lock().map_err(|_| "Estado das atualizações indisponível")?;
        info.direct_update_ready = false; info.downloaded = false; info.error = Some(error);
        Ok(())
    }
}

fn now_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version([u32; 3]);
impl Version {
    fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split('.');
        let mut numbers = [0; 3];
        for n in &mut numbers {
            let p = parts.next()?;
            // Bound numeric conversion and require a single canonical spelling.
            if p.is_empty() || p.len() > 6 || !p.bytes().all(|c| c.is_ascii_digit()) || (p.len() > 1 && p.starts_with('0')) { return None; }
            *n = p.parse().ok()?;
        }
        if parts.next().is_some() { return None; }
        Some(Self(numbers))
    }
    fn text(self) -> String { format!("{}.{}.{}", self.0[0], self.0[1], self.0[2]) }
}

#[derive(Deserialize)]
struct Release {
    draft: bool,
    prerelease: bool,
    published_at: Option<String>,
    tag_name: String,
    html_url: String,
    #[serde(default)] name: Option<String>,
    #[serde(default)] body: Option<String>,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset { name: String, size: u64, state: String, browser_download_url: String }

fn safe_tag(tag: &str) -> bool {
    !tag.is_empty() && tag.len() <= 100 && tag != "." && tag != ".." && !tag.contains("..") &&
        tag.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

fn installer_version(name: &str) -> Option<Version> {
    Version::parse(name.strip_prefix("AM8-Lab-Setup-v")?.strip_suffix(".exe")?)
}

fn asset_url(tag: &str, name: &str) -> String { format!("{RELEASES_URL}/download/{tag}/{name}") }
fn tag_url(tag: &str) -> String { format!("{RELEASES_URL}/tag/{tag}") }

/// Public to allow a developer example to verify the live API without device access.
pub fn select_release(bytes: &[u8], current: &str) -> Result<UpdateInfo> {
    if bytes.len() > MAX_RESPONSE_BYTES { return Err("A resposta do GitHub excedeu o limite de 1 MiB.".into()); }
    let current_version = Version::parse(current).ok_or("Versão instalada inválida")?;
    let list: Vec<serde_json::Value> = serde_json::from_slice(bytes)
        .map_err(|_| "O GitHub enviou uma lista de versões inválida.".to_string())?;
    if list.len() > 30 { return Err("O GitHub enviou mais versões que o limite solicitado.".into()); }
    let mut info = UpdateInfo::idle(current);
    if list.is_empty() { info.status = UpdateStatus::NoRelease; return Ok(info); }
    let mut highest: Option<(Version, Release, Asset)> = None;
    for value in list {
        let Ok(release) = serde_json::from_value::<Release>(value) else { continue; };
        if release.draft || !safe_tag(&release.tag_name) ||
            !release.published_at.as_ref().is_some_and(|s| !s.trim().is_empty() && s.len() <= 64) ||
            (release.html_url != tag_url(&release.tag_name) && release.html_url != format!("{RELEASES_URL}/{}", release.tag_name)) { continue; }
        let mut best_asset = None;
        for asset in &release.assets {
            let Some(version) = installer_version(&asset.name) else { continue; };
            if asset.state != "uploaded" || asset.size == 0 || asset.size > MAX_INSTALLER_BYTES ||
                asset.browser_download_url != asset_url(&release.tag_name, &asset.name) { continue; }
            if best_asset.as_ref().is_none_or(|(v, _)| version > *v) { best_asset = Some((version, asset)); }
        }
        if let Some((version, asset)) = best_asset {
            let asset = Asset { name: asset.name.clone(), size: asset.size, state: asset.state.clone(), browser_download_url: asset.browser_download_url.clone() };
            if highest.as_ref().is_none_or(|(v, _, _)| version > *v) { highest = Some((version, release, asset)); }
        }
    }
    let Some((version, release, asset)) = highest else {
        info.status = UpdateStatus::Unavailable;
        info.error = Some("Não encontrei um instalador Windows válido nas versões publicadas. Abra as versões no GitHub para conferir.".into());
        return Ok(info);
    };
    info.status = if version > current_version { UpdateStatus::Available } else if version == current_version { UpdateStatus::Current } else { UpdateStatus::Ahead };
    info.latest_version = Some(version.text());
    info.download_url = Some(asset_url(&release.tag_name, &asset.name));
    info.download_name = Some(asset.name);
    info.download_size = Some(asset.size);
    info.release_url = Some(tag_url(&release.tag_name));
    info.release_name = release.name.filter(|n| !n.trim().is_empty()).map(|n| n.chars().take(120).collect());
    // The UI must use textContent, never treat notes as HTML or remote instructions.
    info.release_notes = release.body.map(|s| s.chars().take(4000).collect());
    info.prerelease = release.prerelease;
    info.published_at = release.published_at;
    Ok(info)
}

pub(crate) fn permitted_download(info: &UpdateInfo) -> Result<String> {
    if info.status != UpdateStatus::Available { return Err("Consulte as atualizações e encontre uma versão mais nova antes de baixar.".into()); }
    let release = info.release_url.as_deref().ok_or("Versão publicada indisponível")?;
    let tag = release.strip_prefix(&format!("{RELEASES_URL}/tag/")).ok_or("Destino da versão inválido")?;
    let name = info.download_name.as_deref().ok_or("Nome do instalador indisponível")?;
    let version = installer_version(name).ok_or("Nome do instalador inválido")?;
    let current = Version::parse(&info.current_version).ok_or("Versão instalada inválida")?;
    let expected = asset_url(tag, name);
    if !safe_tag(tag) || version <= current || info.latest_version.as_deref() != Some(version.text().as_str()) ||
        info.download_url.as_deref() != Some(expected.as_str()) ||
        !info.download_size.is_some_and(|n| n > 0 && n <= MAX_INSTALLER_BYTES) { return Err("Destino do instalador inválido".into()); }
    Ok(expected)
}

pub fn open_releases() -> Result<()> { open_browser(RELEASES_URL) }

#[cfg(not(windows))]
fn fetch_releases() -> Result<Vec<u8>> { Err("A consulta de atualizações está disponível no Windows.".into()) }
#[cfg(not(windows))]
fn open_browser(_: &str) -> Result<()> { Err("A abertura do navegador está disponível no Windows.".into()) }

#[cfg(windows)]
fn open_browser(url: &str) -> Result<()> {
    use windows::{core::{PCWSTR, w}, Win32::{Foundation::HWND, UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL}}};
    let wide: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
    let result = unsafe { ShellExecuteW(Some(HWND(std::ptr::null_mut())), w!("open"), PCWSTR(wide.as_ptr()), None, None, SW_SHOWNORMAL) };
    if result.0 as isize <= 32 { return Err("Não foi possível abrir o navegador padrão. Abra a página de versões do AM8 Lab no GitHub.".into()); }
    Ok(())
}

#[cfg(windows)]
fn fetch_releases() -> Result<Vec<u8>> {
    use std::{ffi::c_void, ptr, time::Instant};
    use windows::{core::{PCWSTR, w}, Win32::Networking::WinHttp::*};
    struct Handle(*mut c_void);
    impl Drop for Handle { fn drop(&mut self) { let _ = unsafe { WinHttpCloseHandle(self.0) }; } }
    fn handle(raw: *mut c_void) -> Result<Handle> {
        if raw.is_null() { Err("Não foi possível iniciar a conexão HTTPS com o GitHub.".into()) } else { Ok(Handle(raw)) }
    }
    fn net<T>(result: windows::core::Result<T>) -> Result<T> {
        result.map_err(|e| format!("Não foi possível consultar o GitHub. Confira sua conexão e tente novamente. Windows: {e}"))
    }
    let started = Instant::now();
    let agent: Vec<u16> = format!("AM8Lab/{}", env!("CARGO_PKG_VERSION")).encode_utf16().chain(Some(0)).collect();
    let session = handle(unsafe { WinHttpOpen(PCWSTR(agent.as_ptr()), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, None, None, 0) })?;
    net(unsafe { WinHttpSetTimeouts(session.0, 2000, 3000, 3000, 5000) })?;
    net(unsafe { WinHttpSetOption(Some(session.0), WINHTTP_OPTION_CONNECT_RETRIES, Some(&1u32.to_ne_bytes())) })?;
    let connection = handle(unsafe { WinHttpConnect(session.0, w!("api.github.com"), 443, 0) })?;
    let request = handle(unsafe {
        WinHttpOpenRequest(connection.0, w!("GET"), w!("/repos/NightXXT/am8lab/releases?per_page=30"), None, None, ptr::null(), WINHTTP_FLAG_SECURE)
    })?;
    // No cookies, automatic credentials, redirects or relaxed certificate checks.
    let disabled = WINHTTP_DISABLE_REDIRECTS | WINHTTP_DISABLE_COOKIES | WINHTTP_DISABLE_AUTHENTICATION;
    net(unsafe { WinHttpSetOption(Some(request.0), WINHTTP_OPTION_DISABLE_FEATURE, Some(&disabled.to_ne_bytes())) })?;
    let headers: Vec<u16> = "Accept: application/vnd.github+json\r\nX-GitHub-Api-Version: 2022-11-28\r\n".encode_utf16().collect();
    net(unsafe { WinHttpSendRequest(request.0, Some(&headers), None, 0, 0, 0) })?;
    net(unsafe { WinHttpReceiveResponse(request.0, ptr::null_mut()) })?;
    let mut status = 0u32;
    let mut status_size = 4;
    net(unsafe {
        WinHttpQueryHeaders(request.0, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, None,
            Some((&mut status as *mut u32).cast()), &mut status_size, ptr::null_mut())
    })?;
    match status {
        200 => {},
        403 | 429 => return Err("O GitHub limitou temporariamente as consultas. Tente novamente mais tarde ou abra a página de versões.".into()),
        404 => return Err("O repositório de atualizações não foi encontrado no GitHub.".into()),
        300..=399 => return Err("O GitHub solicitou um redirecionamento inesperado; a consulta foi interrompida.".into()),
        _ => return Err(format!("Não foi possível consultar as versões: o GitHub respondeu HTTP {status}.")),
    }
    let mut response = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let remaining = CHECK_TIMEOUT.checked_sub(started.elapsed()).ok_or("A consulta ao GitHub excedeu 15 segundos.")?;
        let receive_ms = remaining.as_millis().clamp(1, 5000) as i32;
        net(unsafe { WinHttpSetTimeouts(request.0, 2000, 3000, 3000, receive_ms) })?;
        let mut read = 0u32;
        net(unsafe { WinHttpReadData(request.0, buffer.as_mut_ptr().cast(), buffer.len() as u32, &mut read) })?;
        if read == 0 { break; }
        if response.len() + read as usize > MAX_RESPONSE_BYTES { return Err("A resposta do GitHub excedeu o limite de 1 MiB.".into()); }
        response.extend_from_slice(&buffer[..read as usize]);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    fn release(tag: &str, version: &str) -> Value {
        let name = format!("AM8-Lab-Setup-v{version}.exe");
        json!({"draft":false,"prerelease":false,"published_at":"2026-10-07T00:00:00Z","tag_name":tag,
            "html_url":tag_url(tag),"name":"AM8 Lab","body":"Melhorias","assets":[
                {"name":name,"state":"uploaded","size":4000000,"browser_download_url":asset_url(tag,&name)}]})
    }
    fn choose(values: Vec<Value>, current: &str) -> UpdateInfo {
        select_release(&serde_json::to_vec(&values).unwrap(), current).unwrap()
    }
    #[test]
    fn versions_are_numeric_canonical_and_bounded() {
        assert!(Version::parse("0.6.10") > Version::parse("0.6.9"));
        assert!(Version::parse("1.0.0") > Version::parse("0.99.999"));
        for bad in ["", "1.2", "1.2.3.4", "v1.2.3", "1.2.3-beta", "01.2.3", "+1.2.3", "-1.2.3", "1.2. 3", "1.2.３", "9999999999999999999999.0.0"] {
            assert!(Version::parse(bad).is_none(), "{bad}");
        }
    }
    #[test]
    fn setup_asset_version_is_independent_of_legacy_tag_and_numeric_order() {
        let info = choose(vec![release("Progams", "0.6.3"), release("v0.6.9", "0.6.9"), release("new", "0.6.10")], "0.6.6");
        assert_eq!(info.latest_version.as_deref(), Some("0.6.10"));
        assert_eq!(info.status, UpdateStatus::Available);
        assert_eq!(permitted_download(&info).unwrap(), asset_url("new", "AM8-Lab-Setup-v0.6.10.exe"));
        assert_eq!(choose(vec![release("Progams", "0.6.3")], "0.6.3").status, UpdateStatus::Current);
    }
    #[test]
    fn newer_local_version_cannot_download_or_downgrade() {
        let ahead = choose(vec![release("Progams", "0.6.3")], "0.6.6");
        assert_eq!(ahead.status, UpdateStatus::Ahead);
        assert!(permitted_download(&ahead).is_err());
        assert!(permitted_download(&choose(vec![release("a", "0.6.6")], "0.6.6")).is_err());
    }
    #[test]
    fn published_prerelease_is_supported_but_draft_and_unpublished_are_ignored() {
        let mut pre = release("candidate", "0.6.7"); pre["prerelease"] = json!(true);
        let mut draft = release("draft", "9.0.0"); draft["draft"] = json!(true);
        let mut unpublished = release("pending", "10.0.0"); unpublished["published_at"] = Value::Null;
        let info = choose(vec![draft, unpublished, pre], "0.6.6");
        assert_eq!(info.latest_version.as_deref(), Some("0.6.7")); assert!(info.prerelease);
    }
    #[test]
    fn unsafe_tags_and_destination_aliases_are_rejected() {
        for tag in ["", ".", "..", "../other", "v1%2Fother", "x?y", "x#z", "@other", "a\\b", "こんにちは"] {
            assert_eq!(choose(vec![release(tag, "0.6.7")], "0.6.6").status, UpdateStatus::Unavailable, "{tag}");
        }
        for url in ["https://evil.example/a.exe", "http://github.com/NightXXT/am8lab/releases/download/v0.6.7/AM8-Lab-Setup-v0.6.7.exe",
            "https://github.com.evil.example/a.exe", "https://github.com@evil.example/a.exe",
            "https://github.com/NightXXT/other/releases/download/v0.6.7/AM8-Lab-Setup-v0.6.7.exe",
            "https://github.com/NightXXT/am8lab/releases/download/v0.6.7/AM8-Lab-Setup-v0.6.7.exe?x=1"] {
            let mut r = release("v0.6.7", "0.6.7"); r["assets"][0]["browser_download_url"] = json!(url);
            assert_eq!(choose(vec![r], "0.6.6").status, UpdateStatus::Unavailable, "{url}");
        }
        let mut r = release("v0.6.7", "0.6.7"); r["html_url"] = json!("https://evil.example/release");
        assert_eq!(choose(vec![r], "0.6.6").status, UpdateStatus::Unavailable);
    }
    #[test]
    fn only_uploaded_nonempty_exact_windows_installers_are_selected() {
        for (field, value) in [("name", json!("")), ("name", json!("AM8-Lab-v0.6.7.zip")), ("size", json!(0)),
            ("size", json!(MAX_INSTALLER_BYTES + 1)), ("state", json!("starter"))] {
            let mut r = release("v0.6.7", "0.6.7"); r["assets"][0][field] = value;
            assert_eq!(choose(vec![r], "0.6.6").status, UpdateStatus::Unavailable);
        }
        let mut r = release("v0.6.7", "0.6.7"); r["assets"] = json!([]);
        assert_eq!(choose(vec![r], "0.6.6").status, UpdateStatus::Unavailable);
        assert_eq!(choose(vec![], "0.6.6").status, UpdateStatus::NoRelease);
    }
    #[test]
    fn malformed_response_size_and_entry_counts_are_bounded() {
        for bytes in [&b"{}"[..], &b"null"[..], &b"42"[..], &b"invalid"[..]] { assert!(select_release(bytes, "0.6.6").is_err()); }
        assert!(select_release(&vec![b' '; MAX_RESPONSE_BYTES + 1], "0.6.6").is_err());
        assert!(select_release(&serde_json::to_vec(&vec![release("v0.6.7", "0.6.7"); 31]).unwrap(), "0.6.6").is_err());
        let mut malformed = release("bad", "20.0.0"); malformed["draft"] = json!("false");
        assert_eq!(choose(vec![malformed, release("valid", "0.6.7")], "0.6.6").latest_version.as_deref(), Some("0.6.7"));
    }
    #[test]
    fn notes_are_literal_unicode_text_with_character_limit() {
        let mut r = release("v0.6.7", "0.6.7");
        r["body"] = json!(format!("<script>window.run()</script>{}", "á".repeat(8000)));
        let info = choose(vec![r], "0.6.6");
        let notes = info.release_notes.unwrap();
        assert!(notes.starts_with("<script>window.run()</script>"));
        assert_eq!(notes.chars().count(), 4000);
    }
    #[test]
    fn cache_error_discards_old_download_and_browser_gate_revalidates() {
        let updates = Updates::new();
        let info = choose(vec![release("v0.6.7", "0.6.7")], "0.6.6");
        *updates.cached.lock().unwrap() = info.clone();
        assert!(updates.record_error("offline".into()).is_err());
        let cached = updates.info().unwrap();
        assert_eq!(cached.status, UpdateStatus::Error); assert!(cached.download_url.is_none());
        let mut tampered = info; tampered.download_url = Some("https://evil.example".into());
        assert!(permitted_download(&tampered).is_err());
    }
    #[test]
    fn check_lease_stays_busy_until_timed_out_worker_finishes() {
        let checking = Arc::new(AtomicBool::new(true));
        let caller = Arc::new(CheckLease(checking.clone()));
        let worker = caller.clone();
        drop(caller);
        assert!(checking.load(Ordering::SeqCst));
        drop(worker);
        assert!(!checking.load(Ordering::SeqCst));
    }
    #[test]
    fn overlapping_check_and_download_do_not_use_stale_available_result() {
        let updates = Updates::new();
        *updates.cached.lock().unwrap() = choose(vec![release("v0.6.7", "0.6.7")], "0.6.6");
        updates.checking.store(true, Ordering::SeqCst);
        assert!(updates.download_destination().unwrap_err().contains("Aguarde"));
        assert!(updates.check().unwrap_err().contains("em andamento"));
        assert_eq!(updates.info().unwrap().status, UpdateStatus::Error);
        assert!(updates.info().unwrap().download_url.is_none());
    }
}
