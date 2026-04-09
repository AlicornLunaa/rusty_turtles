/// Helper functions for creating and decoding payloads
use serde_json::{Value, json};

pub enum Payload {
    Request{ id: u64, oneshot: bool, data: Value },
    Response{ id: u64, data: Value }
}

impl Payload {
    pub fn encode(&self) -> String {
        // Convert payload to string
        match self {
            Payload::Request{ id, oneshot, data } => json!({
                "type": "request",
                "req_id": id,
                "oneshot": oneshot,
                "data": data
            }),
            Payload::Response{ id, data } => json!({
                "type": "response",
                "res_id": id,
                "data": data
            }),
        }.to_string()
    }

    pub fn decode(text: &str) -> Result<Payload, String> {
        let payload: Value = serde_json::from_str(text).unwrap();

        match payload["type"].as_str() {
            Some("request") => {
                let id = payload["req_id"].as_u64().unwrap();
                let oneshot = payload["oneshot"].as_bool().unwrap();
                let data = payload["data"].clone();
                Ok(Payload::Request{ id, oneshot, data })
            },
            Some("response") => {
                let id = payload["res_id"].as_u64().unwrap();
                let data = payload["data"].clone();
                Ok(Payload::Response{ id, data })
            },
            _ => Err("Invalid payload type".to_string()),
        }
    }
}