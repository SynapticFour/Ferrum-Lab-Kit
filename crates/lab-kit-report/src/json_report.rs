use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ReportError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResultRow {
    pub service: String,
    pub passed: bool,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceJsonReport {
    pub lab_name: String,
    pub generated_at: String,
    pub enabled_services: Vec<String>,
    pub results: Vec<ServiceResultRow>,
    pub overall_pass: bool,
    pub next_steps: Vec<String>,
}

/// Accept either an array of results or a `{ "results": [...] }` object from HelixTest exports.
pub fn build_from_helixtest_value(
    raw: &str,
    lab_name: &str,
) -> Result<ConformanceJsonReport, ReportError> {
    let v: Value = serde_json::from_str(raw)?;
    let rows = extract_rows(&v);
    let enabled_services: Vec<String> = rows.iter().map(|r| r.service.clone()).collect();
    let overall_pass = rows.iter().all(|r| r.passed);
    let mut next_steps = Vec::new();
    for r in &rows {
        if !r.passed {
            next_steps.push(format!(
                "Fix failing checks for {} — see HelixTest logs for {}",
                r.service,
                r.detail.as_deref().unwrap_or("details")
            ));
        }
    }
    if next_steps.is_empty() {
        next_steps.push("No failing checks — attach JSON/PDF to your application package.".into());
    }
    Ok(ConformanceJsonReport {
        lab_name: lab_name.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        enabled_services,
        results: rows,
        overall_pass,
        next_steps,
    })
}

fn extract_rows(v: &Value) -> Vec<ServiceResultRow> {
    let mut out = Vec::new();
    if let Some(arr) = v.as_array() {
        for item in arr {
            if let Some(obj) = item.as_object() {
                out.push(row_from_obj(obj));
            }
        }
        return out;
    }
    if let Some(arr) = v.get("results").and_then(|x| x.as_array()) {
        for item in arr {
            if let Some(obj) = item.as_object() {
                out.push(row_from_obj(obj));
            }
        }
        return out;
    }
    // Stub / minimal HelixTest file: single object
    if let Some(obj) = v.as_object() {
        out.push(row_from_obj(obj));
    }
    out
}

fn row_from_obj(obj: &serde_json::Map<String, Value>) -> ServiceResultRow {
    let service = obj
        .get("service")
        .or_else(|| obj.get("name"))
        .and_then(|x| x.as_str())
        .unwrap_or("unknown")
        .to_string();
    let passed = obj
        .get("passed")
        .or_else(|| obj.get("ok"))
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let detail = obj
        .get("message")
        .or_else(|| obj.get("error"))
        .and_then(|x| x.as_str())
        .map(String::from);
    ServiceResultRow {
        service,
        passed,
        detail,
    }
}
