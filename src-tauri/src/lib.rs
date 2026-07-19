mod audio;
mod microphone;
mod session;
mod settings;
mod virtual_mic;
mod websocket;

use session::SessionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SessionManager::default())
        .invoke_handler(tauri::generate_handler![
            audio::list_input_devices,
            audio::list_output_devices,
            settings::get_settings,
            settings::save_settings,
            session::start_session,
            session::stop_session,
            session::get_session_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
