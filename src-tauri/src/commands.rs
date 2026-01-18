use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;
use crate::stream::types::{Stream, StreamInput};

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    pub instance_id: String,
}

#[tauri::command]
pub async fn initialize(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let mut id = state.instance_id.write().await;
    *id = Some(instance_id.clone());
    
    let mut manager = state.stream_manager.write().await;
    manager.initialize(&instance_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_streams(state: State<'_, AppState>) -> Result<Vec<Stream>, String> {
    let manager = state.stream_manager.read().await;
    manager.get_streams().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_stream(
    state: State<'_, AppState>,
    stream: StreamInput,
) -> Result<Stream, String> {
    let mut manager = state.stream_manager.write().await;
    manager.add_stream(stream).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_stream(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut manager = state.stream_manager.write().await;
    manager.start_stream(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_stream(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut manager = state.stream_manager.write().await;
    manager.stop_stream(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_stream(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut manager = state.stream_manager.write().await;
    manager.delete_stream(&id).await.map_err(|e| e.to_string())
}
