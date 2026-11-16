pub mod health;
pub mod monitor;
pub mod runtime;
pub mod sensors;
pub mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // `--simulate-missing-temp` dipakai saat demo sidang untuk menunjukkan
    // perilaku "sensor tidak tersedia" (DESIGN.md §10 butir 5). `args_os`
    // dipakai supaya argumen non-Unicode tidak membuat aplikasi panic saat start.
    let simulate_missing_temp = std::env::args_os()
        .any(|arg| arg == std::ffi::OsStr::new("--simulate-missing-temp"));

    tauri::Builder::default()
        // Harus plugin pertama: instance kedua langsung menyerahkan argumen ke
        // instance yang sudah jalan lalu keluar.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            runtime::on_second_instance(app, &args);
        }))
        .setup(move |app| {
            runtime::init(&app.handle().clone(), simulate_missing_temp)?;
            Ok(())
        })
        .on_window_event(runtime::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            runtime::get_snapshot,
            runtime::get_settings,
            runtime::save_settings,
            runtime::configure_tray,
            runtime::set_tray_tooltip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
