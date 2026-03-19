//! Poll enabled services for readiness (dashboard + HelixTest pre-flight).

use serde::Serialize;

use crate::error::CoreError;
use crate::registry::ServiceRegistry;

#[derive(Debug, Clone, Serialize)]
pub struct ServiceHealth {
    pub service: String,
    pub url: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct HealthAggregator;

impl HealthAggregator {
    /// Synchronous HTTP GET; treats 2xx as healthy.
    pub fn poll(registry: &ServiceRegistry) -> Result<Vec<ServiceHealth>, CoreError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| CoreError::Health(e.to_string()))?;

        let mut out = Vec::new();
        for e in &registry.entries {
            let Some(url) = &e.health_url else {
                continue;
            };
            let url_s = url.to_string();
            match client.get(url.clone()).send() {
                Ok(resp) => {
                    let code = resp.status().as_u16();
                    let ok = resp.status().is_success();
                    out.push(ServiceHealth {
                        service: format!("{:?}", e.id).to_lowercase(),
                        url: url_s,
                        ok,
                        status_code: Some(code),
                        error: if ok {
                            None
                        } else {
                            Some(format!("HTTP {code}"))
                        },
                    });
                }
                Err(err) => out.push(ServiceHealth {
                    service: format!("{:?}", e.id).to_lowercase(),
                    url: url_s,
                    ok: false,
                    status_code: None,
                    error: Some(err.to_string()),
                }),
            }
        }
        Ok(out)
    }
}
