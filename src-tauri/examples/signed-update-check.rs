//! Developer-only check: verifies the published package without installing or accessing USB.
//! The comparator accepts the same version only in this example, never in the application.
use sha2::{Digest, Sha256};
use tauri_plugin_updater::UpdaterExt;

fn main() {
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    let app = tauri::Builder::default().plugin(tauri_plugin_updater::Builder::new().build())
        .build(context).expect("Tauri updater initialization");
    let result = tauri::async_runtime::block_on(async {
        let updater = app.handle().updater_builder().version_comparator(|_, _| true)
            .timeout(std::time::Duration::from_secs(90)).build().map_err(|e| e.to_string())?;
        let candidate = updater.check().await.map_err(|e| e.to_string())?.ok_or("No published signed package")?;
        if candidate.version != env!("CARGO_PKG_VERSION") { return Err("Published version differs from this probe".into()); }
        let expected = format!("https://github.com/NightXXT/am8lab/releases/download/v{0}/AM8-Lab-Setup-v{0}.exe",candidate.version);
        if candidate.download_url.as_str() != expected { return Err("Unexpected download URL".into()); }
        let bytes = candidate.download(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
        println!("Signed version: {}\nVerified bytes: {}\nSHA256: {:x}\nInstallation: NOT executed\nUSB: NOT accessed",candidate.version,bytes.len(),Sha256::digest(&bytes));
        Ok::<(),String>(())
    });
    if let Err(error) = result { eprintln!("{error}"); std::process::exit(1); }
    app.cleanup_before_exit();
}
