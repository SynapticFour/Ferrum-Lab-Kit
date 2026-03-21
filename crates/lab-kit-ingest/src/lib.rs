//! Client for Ferrum’s versioned ingest API (`/api/v1/ingest/*`).
//!
//! Contract: [Ferrum `docs/INGEST-LAB-KIT.md`](https://github.com/SynapticFour/Ferrum/blob/main/docs/INGEST-LAB-KIT.md).

#![forbid(unsafe_code)]

use reqwest::{multipart::Form, Response};
use serde::{Deserialize, Serialize};

/// Register request body (`POST /api/v1/ingest/register`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub items: Vec<RegisterItem>,
}

/// One item in a register batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RegisterItem {
    Url {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        derived_from: Option<Vec<String>>,
    },
    ExistingObject {
        storage_backend: String,
        storage_key: String,
        size: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_encrypted: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        checksums: Option<Vec<ChecksumInput>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumInput {
    #[serde(rename = "type")]
    pub checksum_type: String,
    pub checksum: String,
}

/// Job envelope returned by register, upload, and job polling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestJobResponse {
    pub job_id: String,
    pub status: String,
    pub job_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}

/// Multipart upload options (`POST /api/v1/ingest/upload`).
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub encrypt: Option<bool>,
    pub expected_sha256: Option<String>,
    pub workspace_id: Option<String>,
    pub client_request_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid response JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("gateway HTTP {status}: {body}")]
    HttpBody { status: u16, body: String },
    #[error("gateway HTTP {status}: {code} — {message}")]
    Structured {
        status: u16,
        code: String,
        message: String,
    },
}

#[derive(Deserialize)]
struct ApiErrorBody {
    code: String,
    message: String,
}

/// HTTP client for `/api/v1/ingest/*`.
#[derive(Debug, Clone)]
pub struct IngestClient {
    http: reqwest::Client,
    base: String,
    token: Option<String>,
}

impl IngestClient {
    /// `base_url` is the gateway origin only (e.g. `http://localhost:8080`), no `/api/...` path.
    pub fn new(base_url: impl AsRef<str>, token: Option<String>) -> Result<Self, IngestError> {
        let base = base_url.as_ref().trim_end_matches('/').to_string();
        let http = reqwest::Client::builder().build()?;
        Ok(Self { http, base, token })
    }

    fn auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.token {
            Some(t) if !t.is_empty() => rb.bearer_auth(t),
            _ => rb,
        }
    }

    async fn parse_job_response(resp: Response) -> Result<IngestJobResponse, IngestError> {
        let status = resp.status();
        let code = status.as_u16();
        let text = resp.text().await?;
        if status.is_success() {
            return Ok(serde_json::from_str(&text)?);
        }
        if let Ok(body) = serde_json::from_str::<ApiErrorBody>(&text) {
            return Err(IngestError::Structured {
                status: code,
                code: body.code,
                message: body.message,
            });
        }
        Err(IngestError::HttpBody {
            status: code,
            body: text,
        })
    }

    /// `POST /api/v1/ingest/register`
    pub async fn register(&self, body: &RegisterRequest) -> Result<IngestJobResponse, IngestError> {
        let url = format!("{}/api/v1/ingest/register", self.base);
        let resp = self.auth(self.http.post(url).json(body)).send().await?;
        Self::parse_job_response(resp).await
    }

    /// `POST /api/v1/ingest/upload` (multipart).
    pub async fn upload(
        &self,
        file_part_name: &str,
        bytes: Vec<u8>,
        opts: UploadOptions,
    ) -> Result<IngestJobResponse, IngestError> {
        let url = format!("{}/api/v1/ingest/upload", self.base);
        let mut part = reqwest::multipart::Part::bytes(bytes).file_name(file_part_name.to_string());
        if let Some(ref m) = opts.mime_type {
            part = part.mime_str(m)?;
        }
        let mut form = Form::new().part("file", part);
        if let Some(n) = &opts.name {
            form = form.text("name", n.clone());
        }
        if let Some(true) = opts.encrypt {
            form = form.text("encrypt", "true");
        } else if let Some(false) = opts.encrypt {
            form = form.text("encrypt", "false");
        }
        if let Some(s) = &opts.expected_sha256 {
            form = form.text("expected_sha256", s.clone());
        }
        if let Some(w) = &opts.workspace_id {
            form = form.text("workspace_id", w.clone());
        }
        if let Some(c) = &opts.client_request_id {
            form = form.text("client_request_id", c.clone());
        }
        let resp = self
            .auth(self.http.post(url).multipart(form))
            .send()
            .await?;
        Self::parse_job_response(resp).await
    }

    /// `GET /api/v1/ingest/jobs/{job_id}`
    pub async fn get_job(&self, job_id: &str) -> Result<IngestJobResponse, IngestError> {
        let url = format!(
            "{}/api/v1/ingest/jobs/{}",
            self.base,
            urlencoding::encode(job_id)
        );
        let resp = self.auth(self.http.get(url)).send().await?;
        Self::parse_job_response(resp).await
    }
}

// Lightweight encoding for path segment (ULIDs are safe; this keeps the API honest).
mod urlencoding {
    pub fn encode(s: &str) -> std::borrow::Cow<'_, str> {
        if s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            std::borrow::Cow::Borrowed(s)
        } else {
            std::borrow::Cow::Owned(
                s.bytes()
                    .map(|b| match b {
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => {
                            char::from(b).to_string()
                        }
                        _ => format!("%{:02X}", b),
                    })
                    .collect(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_serializes() {
        let r = RegisterRequest {
            client_request_id: Some("id1".into()),
            workspace_id: None,
            items: vec![RegisterItem::Url {
                url: "https://example.com/f.txt".into(),
                name: Some("n".into()),
                mime_type: None,
                derived_from: None,
            }],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["items"][0]["kind"], "url");
    }
}
