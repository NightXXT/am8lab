fn main() {
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "inspect", "apply_effect", "apply_headphone_mode", "get_device_info", "compare_original", "restore_all", "test_gain",
                "smoke_mode", "preview_mode", "recovery_pending", "finish_smoke",
                "get_update_info", "check_updates", "open_releases", "download_update", "install_update",
            ])
        )).expect("Tauri build configuration");
        // Tauri embeds the Windows manifest in the application binary only.
        // Desktop examples also import TaskDialogIndirect and require Common Controls v6.
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            let manifest = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Build output directory"))
                .join("desktop-examples.manifest");
            std::fs::write(&manifest, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<dependency><dependentAssembly><assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" /></dependentAssembly></dependency>
</assembly>"#).expect("Write desktop example manifest");
            println!("cargo:rustc-link-arg-examples=/MANIFEST:EMBED");
            println!("cargo:rustc-link-arg-examples=/MANIFESTINPUT:{}", manifest.display());
        }
    }
}
