use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Frame {
    /// Server -> client: tunnel assigned.
    Ready {
        tunnel_id: String,
        public_url: String,
    },
    /// Server -> client: forward this HTTP request to local port.
    Request {
        req_id: String,
        method: String,
        path: String,
        headers: BTreeMap<String, String>,
        /// base64-encoded body
        body_b64: String,
    },
    /// Client -> server: HTTP response for a prior request.
    Response {
        req_id: String,
        status: u16,
        headers: BTreeMap<String, String>,
        body_b64: String,
    },
    /// Client -> server (reply to Request) when forwarding failed locally.
    Error { req_id: String, message: String },
    /// Either direction: keep-alive.
    Ping,
    Pong,
}

impl Frame {
    pub fn encode(&self) -> String {
        serde_json::to_string(self).expect("Frame serialization is infallible")
    }

    pub fn decode(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
