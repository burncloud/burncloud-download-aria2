pub mod types;

use crate::error::Aria2Error;
use types::*;
use serde_json::json;

/// JSON-RPC client for aria2 communication
pub struct Aria2Client {
    rpc_url: String,
    secret: Option<String>,
    http_client: reqwest::Client,
}

impl Aria2Client {
    pub fn new(rpc_url: String, secret: Option<String>) -> Self {
        Self {
            rpc_url,
            secret,
            http_client: reqwest::Client::new(),
        }
    }

    async fn call_rpc(&self, method: String, params: Vec<serde_json::Value>) -> Result<serde_json::Value, Aria2Error> {
        let mut params_with_secret = Vec::new();
        if let Some(ref secret) = self.secret {
            params_with_secret.push(json!(format!("token:{}", secret)));
        }
        params_with_secret.extend(params);

        let request = JsonRpcRequest::new(method.clone(), params_with_secret);

        let response = self.http_client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Aria2Error::DaemonUnavailable(format!("Failed to connect: {}", e)))?;

        let rpc_response: JsonRpcResponse = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(Aria2Error::RpcError(error.code, error.message));
        }

        rpc_response.result.ok_or_else(|| Aria2Error::General("No result in response".to_string()))
    }

    /// Add URI download (HTTP/HTTPS/FTP/Magnet)
    pub async fn add_uri(&self, uris: Vec<String>, options: Aria2Options) -> Result<String, Aria2Error> {
        let result = self.call_rpc(
            "aria2.addUri".to_string(),
            vec![json!(uris), json!(options)]
        ).await?;

        result.as_str()
            .ok_or_else(|| Aria2Error::General("Invalid GID response".to_string()))
            .map(|s| s.to_string())
    }

    /// Add torrent download
    pub async fn add_torrent(&self, torrent_data: Vec<u8>, options: Aria2Options) -> Result<String, Aria2Error> {
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &torrent_data);
        let result = self.call_rpc(
            "aria2.addTorrent".to_string(),
            vec![json!(encoded), json!([]), json!(options)]
        ).await?;

        result.as_str()
            .ok_or_else(|| Aria2Error::General("Invalid GID response".to_string()))
            .map(|s| s.to_string())
    }

    /// Add metalink download
    pub async fn add_metalink(&self, metalink_data: Vec<u8>, options: Aria2Options) -> Result<String, Aria2Error> {
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &metalink_data);
        let result = self.call_rpc(
            "aria2.addMetalink".to_string(),
            vec![json!(encoded), json!(options)]
        ).await?;

        result.as_str()
            .ok_or_else(|| Aria2Error::General("Invalid GID response".to_string()))
            .map(|s| s.to_string())
    }

    /// Get download status - returns raw JSON for real-time data access
    pub async fn tell_status(&self, gid: &str) -> Result<serde_json::Value, Aria2Error> {
        self.call_rpc(
            "aria2.tellStatus".to_string(),
            vec![json!(gid)]
        ).await
    }

    /// Pause download
    pub async fn pause(&self, gid: &str) -> Result<(), Aria2Error> {
        self.call_rpc(
            "aria2.pause".to_string(),
            vec![json!(gid)]
        ).await?;
        Ok(())
    }

    /// Unpause download
    pub async fn unpause(&self, gid: &str) -> Result<(), Aria2Error> {
        self.call_rpc(
            "aria2.unpause".to_string(),
            vec![json!(gid)]
        ).await?;
        Ok(())
    }

    /// Remove download
    pub async fn remove(&self, gid: &str) -> Result<(), Aria2Error> {
        self.call_rpc(
            "aria2.remove".to_string(),
            vec![json!(gid)]
        ).await?;
        Ok(())
    }

    /// Get global status
    pub async fn get_global_stat(&self) -> Result<serde_json::Value, Aria2Error> {
        self.call_rpc("aria2.getGlobalStat".to_string(), vec![]).await
    }

    /// Get active downloads - returns raw JSON for real-time data access
    pub async fn tell_active(&self) -> Result<serde_json::Value, Aria2Error> {
        self.call_rpc("aria2.tellActive".to_string(), vec![]).await
    }

    /// Get stopped downloads - returns raw JSON for real-time data access
    pub async fn tell_stopped(&self, offset: i32, num: i32) -> Result<serde_json::Value, Aria2Error> {
        self.call_rpc(
            "aria2.tellStopped".to_string(),
            vec![json!(offset), json!(num)]
        ).await
    }

    /// Get waiting downloads - returns raw JSON for real-time data access
    pub async fn tell_waiting(&self, offset: i32, num: i32) -> Result<serde_json::Value, Aria2Error> {
        self.call_rpc(
            "aria2.tellWaiting".to_string(),
            vec![json!(offset), json!(num)]
        ).await
    }
}