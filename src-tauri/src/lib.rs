mod commands;
mod db;
mod stream;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::stream::manager::StreamManager;

pub struct AppState {
    pub stream_manager: Arc<RwLock<StreamManager>>,
    pub instance_id: RwLock<Option<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState {
                stream_manager: Arc::new(RwLock::new(StreamManager::new())),
                instance_id: RwLock::new(None),
            };
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::initialize,
            commands::get_streams,
            commands::add_stream,
            commands::start_stream,
            commands::stop_stream,
            commands::delete_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
