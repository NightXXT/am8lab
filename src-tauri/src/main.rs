#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use am8_lab::{protocol::{Effect, Result}, session::{Session, Snapshot}, windows::{HidDevice, InstanceMutex, show_error}};
use std::{collections::BTreeMap, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};
use tauri::{Emitter, Manager};
use am8_lab::protocol::Transport;
use am8_lab::updates::{UpdateInfo, Updates};

#[derive(Clone)]
struct Shared { session: Arc<Mutex<Session>>, closing: Arc<AtomicBool>, updates: Arc<Updates> }

async fn operation(state: Shared, job: impl FnOnce(&mut Session, &mut HidDevice) -> Result<Snapshot> + Send + 'static) -> Result<Snapshot> {
    if smoke_mode() || preview_mode() { return Err("Prévia da interface sem acesso USB".into()); }
    if state.closing.load(Ordering::SeqCst) { return Err("Restauração para fechamento em andamento".into()); }
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = state.session.lock().map_err(|_| "Estado indisponível")?;
        if state.closing.load(Ordering::SeqCst) { return Err("O aplicativo está restaurando para fechar".into()); }
        let mut device = HidDevice::open()?;
        job(&mut session, &mut device)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn inspect(state: tauri::State<'_, Shared>) -> Result<Snapshot> {
    operation(state.inner().clone(), |s, t| s.inspect(t)).await
}
#[tauri::command]
async fn apply_effect(state: tauri::State<'_, Shared>, effect: Effect, enabled: bool, values: BTreeMap<String, i32>) -> Result<Snapshot> {
    operation(state.inner().clone(), move |s, t| s.apply(t, effect, enabled, values)).await
}
#[tauri::command]
async fn apply_headphone_mode(state: tauri::State<'_, Shared>, mode: String) -> Result<Snapshot> {
    operation(state.inner().clone(), move |s, t| s.apply_headphone_mode(t, &mode)).await
}
#[tauri::command]
async fn get_device_info(state: tauri::State<'_, Shared>) -> Result<am8_lab::device_info::DeviceInfo> {
    if smoke_mode() || preview_mode() { return Err("Prévia da interface sem leitura do dispositivo".into()); }
    let state = state.inner().clone();
    if state.closing.load(Ordering::SeqCst) { return Err("Restauração para fechamento em andamento".into()); }
    tauri::async_runtime::spawn_blocking(move || {
        let _session = state.session.lock().map_err(|_| "Estado indisponível")?;
        if state.closing.load(Ordering::SeqCst) { return Err("O aplicativo está restaurando para fechar".into()); }
        am8_lab::device_info::read(&mut HidDevice::open()?)
    }).await.map_err(|error| error.to_string())?
}
#[tauri::command]
async fn compare_original(state: tauri::State<'_, Shared>) -> Result<Snapshot> {
    operation(state.inner().clone(), |s, t| s.compare(t)).await
}
#[tauri::command]
async fn restore_all(state: tauri::State<'_, Shared>) -> Result<Snapshot> {
    operation(state.inner().clone(), |s, t| s.restore(t)).await
}
#[tauri::command]
async fn test_gain(state: tauri::State<'_, Shared>, db: i32) -> Result<Snapshot> {
    operation(state.inner().clone(), move |s, t| s.test_gain(t, db)).await
}
#[tauri::command]
fn smoke_mode() -> bool { std::env::args().any(|x| x == "--smoke-test") }
#[tauri::command]
fn preview_mode() -> bool { std::env::args().any(|x| x == "--design-preview") }
#[tauri::command]
fn recovery_pending(state: tauri::State<'_, Shared>) -> Result<bool> {
    Ok(state.session.lock().map_err(|_| "Estado indisponível")?.pending())
}
#[tauri::command]
fn finish_smoke(app: tauri::AppHandle, passed: bool) { if smoke_mode() { app.exit(if passed { 0 } else { 1 }); } }

#[tauri::command]
fn get_update_info(state: tauri::State<'_, Shared>) -> Result<UpdateInfo> { state.updates.info() }

fn allow_update_operation() -> Result<()> {
    if smoke_mode() || preview_mode() { return Err("Prévia da interface sem consulta de rede ou abertura do navegador".into()); }
    Ok(())
}

#[tauri::command]
async fn check_updates(state: tauri::State<'_, Shared>) -> Result<UpdateInfo> {
    allow_update_operation()?;
    let updates = state.updates.clone();
    tauri::async_runtime::spawn_blocking(move || updates.check()).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_releases() -> Result<()> {
    allow_update_operation()?;
    tauri::async_runtime::spawn_blocking(am8_lab::updates::open_releases).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn download_update(state: tauri::State<'_, Shared>) -> Result<()> {
    allow_update_operation()?;
    let updates = state.updates.clone();
    tauri::async_runtime::spawn_blocking(move || updates.download()).await.map_err(|e| e.to_string())?
}

fn main() {
    let smoke = smoke_mode() || preview_mode();
    let _instance = if smoke { None } else {
        match InstanceMutex::acquire() { Ok(m) => Some(m), Err(e) => { show_error(&e); return; } }
    };
    let path = if smoke { std::env::temp_dir().join("am8-rust-interface-smoke.json") } else {
        let Some(local) = std::env::var_os("LOCALAPPDATA") else { show_error("LOCALAPPDATA indisponível"); return; };
        std::path::PathBuf::from(local).join("AM8Lab").join("efeitos-recuperacao.json")
    };
    let session = match Session::open(path) { Ok(s) => s, Err(e) => { show_error(&e); return; } };
    tauri::Builder::default()
        .manage(Shared { session: Arc::new(Mutex::new(session)), closing: Arc::new(AtomicBool::new(false)), updates: Arc::new(Updates::new()) })
        .invoke_handler(tauri::generate_handler![inspect, apply_effect, apply_headphone_mode, get_device_info, compare_original, restore_all, test_gain, smoke_mode, preview_mode, finish_smoke, recovery_pending, get_update_info, check_updates, open_releases, download_update])
        .setup(move |app| {
            if !smoke { start_telemetry(app.handle().clone(), app.state::<Shared>().inner().clone()); }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Shared>().inner().clone();
                api.prevent_close();
                if state.closing.swap(true, Ordering::SeqCst) { return; }
                let app = window.app_handle().clone();
                let _ = window.emit("closing-restore", ());
                tauri::async_runtime::spawn_blocking(move || {
                    let result = (|| {
                        let mut s = state.session.lock().map_err(|_| "Estado indisponível")?;
                        if s.pending() { s.restore(&mut HidDevice::open()?).map(|_| ()) } else { Ok(()) }
                    })();
                    match result {
                        Ok(()) => app.exit(0),
                        Err(error) => {
                            state.closing.store(false, Ordering::SeqCst);
                            let _ = app.emit("operation-error", format!("Não foi possível restaurar para fechar. Reconecte e restaure: {error}"));
                        }
                    }
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar o AM8 Lab");
}


fn start_telemetry(app: tauri::AppHandle, state: Shared) {
    std::thread::spawn(move || {
        let mut device: Option<HidDevice> = None;
        let mut output=None;
        let mut output_time=std::time::Instant::now()-std::time::Duration::from_secs(10);
        let mut identity_time=std::time::Instant::now();
        loop {
            if state.closing.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            if output_time.elapsed().as_secs()>=5 { output=am8_lab::audio_status::default_output(); output_time=std::time::Instant::now(); }
            if let Ok(_session)=state.session.try_lock() {
                let result: Result<(u16,u16)>=(|| {
                    if device.is_none() { let mut d=HidDevice::open()?; d.guard()?; device=Some(d); identity_time=std::time::Instant::now(); }
                    let d=device.as_mut().unwrap();
                    if identity_time.elapsed().as_secs()>=5 {
                        let id=d.query(0,&[])?;
                        if id!=[0x42,0,7,1,2,43,2,2,23,2,2,b'B',b'5',1] { return Err("Identidade alterada".into()); }
                        identity_time=std::time::Instant::now();
                    }
                    Ok((d.meter_with_wait(0x91,8)?,d.meter_with_wait(0x82,8)?))
                })();
                match result {
                    Ok((voice,playback)) => {
                        let sampled_at=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                        let _=app.emit("meters",serde_json::json!({"connected":true,"voice":voice,"playback":playback,"output":output,"sampled_at":sampled_at}));
                    }
                    Err(error) => { device=None; let _=app.emit("meters",serde_json::json!({"connected":false,"voice":0,"playback":0,"output":output,"error":error})); }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(if device.is_some() { 10 } else { 1500 }));
        }
    });
}
