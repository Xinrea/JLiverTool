use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum IpcRequest {
    GetUserInfo { uid: u64 },
    GetRoomInfo { room_id: u64 },
    OpenUrl { url: String },
    GetFonts,
    SetClipboard { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum IpcResponse {
    Success(serde_json::Value),
    Error { message: String },
}

impl IpcResponse {
    pub fn success<T: Serialize>(data: T) -> Self {
        IpcResponse::Success(serde_json::to_value(data).unwrap_or(serde_json::Value::Null))
    }

    pub fn error(message: impl Into<String>) -> Self {
        IpcResponse::Error {
            message: message.into(),
        }
    }
}

pub struct IpcHandler {
    // Will hold references to API clients when needed
}

impl IpcHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle(&self, request: IpcRequest) -> Result<IpcResponse> {
        match request {
            IpcRequest::GetUserInfo { uid } => {
                // TODO: Implement user info fetching via Bilibili API
                log::warn!("GetUserInfo not yet implemented for uid: {}", uid);
                Ok(IpcResponse::error("Not implemented"))
            }
            IpcRequest::GetRoomInfo { room_id } => {
                // TODO: Implement room info fetching via Bilibili API
                log::warn!("GetRoomInfo not yet implemented for room_id: {}", room_id);
                Ok(IpcResponse::error("Not implemented"))
            }
            IpcRequest::OpenUrl { url } => {
                if let Err(e) = open::that(&url) {
                    log::error!("Failed to open URL {}: {}", url, e);
                    Ok(IpcResponse::error(format!("Failed to open URL: {}", e)))
                } else {
                    Ok(IpcResponse::success(serde_json::json!({"opened": true})))
                }
            }
            IpcRequest::GetFonts => {
                // TODO: Implement font enumeration
                log::warn!("GetFonts not yet implemented");
                Ok(IpcResponse::error("Not implemented"))
            }
            IpcRequest::SetClipboard { text } => {
                // TODO: Implement clipboard setting
                log::warn!("SetClipboard not yet implemented: {}", text);
                Ok(IpcResponse::error("Not implemented"))
            }
        }
    }
}
