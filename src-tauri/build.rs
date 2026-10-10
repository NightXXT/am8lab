fn main() {
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "inspect", "apply_effect", "apply_headphone_mode", "get_device_info", "compare_original", "restore_all", "test_gain",
                "smoke_mode", "preview_mode", "recovery_pending", "finish_smoke",
                "get_update_info", "check_updates", "open_releases", "download_update", "install_update",
            ])
        )).expect("Tauri build configuration");
    }
}
