use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::sync::Arc;

use serde_json::{json, Value};

use crate::canonical::json_line;
use crate::error::{bad_request, forbidden, ApiError};
use crate::store::Store;

const MAX_BODY: usize = 1_048_576;
const MAX_HEADERS: usize = 100;

#[derive(Clone)]
pub struct Server {
    pub store: Arc<Store>,
}

pub struct Request {
    pub method: String,
    pub path: String,
    headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn read(stream: &mut TcpStream) -> Result<Self, ApiError> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Err(bad_request("empty request"));
        }
        let mut parts = line.split_whitespace();
        let method = parts
            .next()
            .ok_or_else(|| bad_request("missing method"))?
            .to_string();
        let target = parts
            .next()
            .ok_or_else(|| bad_request("missing request target"))?;
        let path = target.split('?').next().unwrap_or(target).to_string();

        let mut headers = HashMap::new();
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header)? == 0 || header == "\r\n" || header == "\n" {
                break;
            }
            let (name, value) = header
                .split_once(':')
                .ok_or_else(|| bad_request("malformed header"))?;
            if headers.len() >= MAX_HEADERS {
                return Err(bad_request("too many headers"));
            }
            headers.insert(
                name.trim().to_ascii_lowercase(),
                value.trim().to_string(),
            );
        }

        let length: usize = headers
            .get("content-length")
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or(0);
        if length > MAX_BODY {
            return Err(bad_request("request body is too large"));
        }
        let mut body = vec![0u8; length];
        reader.read_exact(&mut body)?;
        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }

    fn header(&self, name: &str) -> &str {
        self.headers.get(name).map(String::as_str).unwrap_or("")
    }

    fn body_json(&self) -> Result<Value, ApiError> {
        if self.body.is_empty() {
            Ok(json!({}))
        } else {
            Ok(serde_json::from_slice(&self.body)?)
        }
    }
}

pub struct Response {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Response {
    fn json(status: u16, value: &Value) -> Self {
        let body = json_line(value).unwrap_or_else(|_| b"{}".to_vec());
        Response {
            status,
            content_type: "application/json",
            body,
        }
    }

    fn error(error: &ApiError) -> Self {
        Self::json(error.status(), &json!({"error": error.message()}))
    }

    fn bytes(status: u16, body: Vec<u8>) -> Self {
        Response {
            status,
            content_type: "application/octet-stream",
            body,
        }
    }
}

pub fn error_response(error: &ApiError) -> Response {
    Response::error(error)
}

impl Server {
    pub fn handle(&self, request: &Request) -> Response {
        match self.route(request) {
            Ok(response) => response,
            Err(error) => Response::error(&error),
        }
    }

    fn route(&self, request: &Request) -> Result<Response, ApiError> {
        match request.method.as_str() {
            "GET" => self.get(request),
            "POST" => self.post(request),
            _ => Err(ApiError::NotFound),
        }
    }

    fn get(&self, request: &Request) -> Result<Response, ApiError> {
        if request.path == "/healthz" {
            return Ok(Response::json(200, &json!({"ok": true})));
        }
        if let Some(raw_name) = request.path.strip_prefix("/data/") {
            let name = raw_name.rsplit('/').next().unwrap_or("");
            if name.is_empty()
                || name.contains("..")
                || !(name.ends_with(".json") || name.ends_with(".sig") || name.ends_with(".jsonl"))
            {
                return Err(ApiError::NotFound);
            }
            for folder in ["", "signatures"] {
                let path = if folder.is_empty() {
                    self.store.data_dir().join(name)
                } else {
                    self.store.data_dir().join(folder).join(name)
                };
                if let Ok(bytes) = fs::read(path) {
                    return Ok(Response::bytes(200, bytes));
                }
            }
            return Err(ApiError::NotFound);
        }
        Err(ApiError::NotFound)
    }

    fn post(&self, request: &Request) -> Result<Response, ApiError> {
        match request.path.as_str() {
            "/challenge" => {
                let (code, poll_token) = self.store.new_challenge_pair()?;
                Ok(Response::json(
                    200,
                    &json!({"code": code, "poll_token": poll_token, "expires_in": 300}),
                ))
            }
            "/challenge/status" => {
                let body = request.body_json()?;
                let code = body.get("code").and_then(Value::as_str).unwrap_or("");
                let poll_token = body.get("poll_token").and_then(Value::as_str).unwrap_or("");
                match self.store.poll_challenge(code, poll_token)? {
                    Some(session) => Ok(Response::json(200, &json!({"status": "bound", "session": session}))),
                    None => Ok(Response::json(200, &json!({"status": "pending"}))),
                }
            }
            "/telegram/webhook" => self.webhook(request),
            "/admin/grant" | "/admin/revoke" => self.admin(request),
            _ => Err(ApiError::NotFound),
        }
    }

    fn webhook(&self, request: &Request) -> Result<Response, ApiError> {
        let expected = env::var("GRP_BOT_SECRET").unwrap_or_default();
        if expected.is_empty() || request.header("x-telegram-bot-api-secret-token") != expected {
            return Err(forbidden("bad secret"));
        }
        let body = request.body_json()?;
        let message = body.get("message").cloned().unwrap_or_else(|| json!({}));
        let text = message
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if !text.starts_with("/bind ") {
            return Ok(Response::json(200, &json!({"ok": true})));
        }
        let code = text[6..].trim().to_uppercase();
        let id = message.get("from").and_then(|from| from.get("id"));
        let telegram_id = match id {
            Some(Value::Number(number)) => number.as_u64().map(|value| value.to_string()),
            Some(Value::String(value)) => Some(value.clone()),
            _ => None,
        }
        .ok_or_else(|| forbidden("not an owner"))?;
        let session = self.store.bind(&code, &telegram_id)?;
        Ok(Response::json(
            200,
            &json!({"ok": true, "session": session}),
        ))
    }

    fn admin(&self, request: &Request) -> Result<Response, ApiError> {
        let token = request
            .header("authorization")
            .strip_prefix("Bearer ")
            .unwrap_or("");
        let body = request.body_json()?;
        let user_id = match body.get("user_id") {
            Some(Value::String(value)) => value.clone(),
            Some(Value::Number(value)) => value.to_string(),
            _ => String::new(),
        };
        let badge = body
            .get("badge")
            .and_then(Value::as_str)
            .unwrap_or("grapegram");
        let reason = body.get("reason").and_then(Value::as_str).unwrap_or("");
        let style = body.get("style");
        let enabled = request.path.ends_with("grant");
        let version = self
            .store
            .mutate(token, &user_id, badge, enabled, reason, style)?;
        Ok(Response::json(
            200,
            &json!({"ok": true, "version": version}),
        ))
    }
}

pub fn write_response(stream: &mut TcpStream, response: Response) -> std::io::Result<()> {
    let reason = match response.status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        reason,
        response.content_type,
        response.body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()
}
