// SPDX-License-Identifier: BUSL-1.1
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ReportError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResultRow {
    pub service: String,
    pub passed: bool,
    #[serde(default)]
    pub executed_tests: u32,
    #[serde(default)]
    pub skipped_tests: u32,
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
    /// True when at least one non-skipped check ran.
    pub had_executed_checks: bool,
    pub next_steps: Vec<String>,
}

/// Accept HelixTest `OverallReport` JSON, a `{ "results": [...] }` object, or a raw array.
pub fn build_from_helixtest_value(
    raw: &str,
    lab_name: &str,
) -> Result<ConformanceJsonReport, ReportError> {
    let v: Value = serde_json::from_str(raw)?;
    let rows = extract_rows(&v);
    if rows.is_empty() {
        return Err(ReportError::EmptyConformance(
            "no service results in HelixTest JSON".into(),
        ));
    }
    let enabled_services = extract_enabled_services(&v, &rows);
    let had_executed_checks = rows.iter().any(|r| r.executed_tests > 0);
    if !had_executed_checks {
        return Err(ReportError::EmptyConformance(
            "every HelixTest check was skipped — not a pass".into(),
        ));
    }
    let overall_pass = rows.iter().all(|r| r.passed) && had_executed_checks;
    let mut next_steps = Vec::new();
    for r in &rows {
        if r.executed_tests == 0 {
            next_steps.push(format!(
                "{} reported only skipped checks — not counted as pass",
                r.service
            ));
        } else if !r.passed {
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
        had_executed_checks,
        next_steps,
    })
}

fn json_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        other if !other.is_null() => Some(other.to_string().trim_matches('"').to_string()),
        _ => None,
    }
}

fn extract_enabled_services(v: &Value, rows: &[ServiceResultRow]) -> Vec<String> {
    if let Some(arr) = v.get("enabled_services").and_then(|x| x.as_array()) {
        let names: Vec<String> = arr.iter().filter_map(json_string).collect();
        if !names.is_empty() {
            return names;
        }
    }
    rows.iter().map(|r| r.service.clone()).collect()
}

fn extract_rows(v: &Value) -> Vec<ServiceResultRow> {
    if let Some(arr) = v.get("services").and_then(|x| x.as_array()) {
        if arr.iter().any(|item| item.get("tests").is_some()) {
            return arr.iter().filter_map(row_from_helixtest_service).collect();
        }
        let rows: Vec<ServiceResultRow> = arr
            .iter()
            .filter_map(|item| item.as_object().and_then(row_from_obj))
            .collect();
        if !rows.is_empty() {
            return rows;
        }
    }
    if let Some(arr) = v.as_array() {
        return arr
            .iter()
            .filter_map(|item| item.as_object().and_then(row_from_obj))
            .collect();
    }
    if let Some(arr) = v.get("results").and_then(|x| x.as_array()) {
        return arr
            .iter()
            .filter_map(|item| item.as_object().and_then(row_from_obj))
            .collect();
    }
    if let Some(obj) = v.as_object() {
        if let Some(row) = row_from_obj(obj) {
            return vec![row];
        }
    }
    Vec::new()
}

/// HelixTest `ServiceReport`: `{ "service": "Wes", "tests": [{ "name", "passed", "status", "error" }] }`.
fn row_from_helixtest_service(item: &Value) -> Option<ServiceResultRow> {
    let obj = item.as_object()?;
    let service = obj.get("service").and_then(json_string)?;
    let Some(tests) = obj.get("tests").and_then(|t| t.as_array()) else {
        return row_from_obj(obj);
    };
    let mut failed = Vec::new();
    let mut executed = 0u32;
    let mut skipped = 0u32;
    for t in tests {
        let status = t
            .get("status")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if status == "skip" {
            skipped += 1;
            continue;
        }
        executed += 1;
        let name = t.get("name").and_then(|x| x.as_str()).unwrap_or("test");
        let passed = t.get("passed").and_then(|x| x.as_bool()).unwrap_or(false) || status == "pass";
        if !passed {
            let err = t.get("error").and_then(|x| x.as_str()).unwrap_or("failed");
            failed.push(format!("{name}: {err}"));
        }
    }
    let passed = failed.is_empty() && executed > 0;
    Some(ServiceResultRow {
        service,
        passed,
        executed_tests: executed,
        skipped_tests: skipped,
        detail: if executed == 0 {
            Some("all checks skipped".into())
        } else if failed.is_empty() {
            None
        } else {
            Some(failed.join("; "))
        },
    })
}

fn row_from_obj(obj: &serde_json::Map<String, Value>) -> Option<ServiceResultRow> {
    let service = obj
        .get("service")
        .or_else(|| obj.get("name"))
        .or_else(|| obj.get("service_id"))
        .and_then(json_string);
    let passed = obj
        .get("passed")
        .or_else(|| obj.get("ok"))
        .or_else(|| obj.get("success"))
        .and_then(|x| x.as_bool());
    if service.is_none() && passed.is_none() {
        return None;
    }
    let detail = obj
        .get("message")
        .or_else(|| obj.get("error"))
        .or_else(|| obj.get("detail"))
        .and_then(|x| x.as_str())
        .map(String::from);
    let passed = passed.unwrap_or(false);
    Some(ServiceResultRow {
        service: service.unwrap_or_else(|| "unknown".into()),
        passed,
        executed_tests: if passed || detail.is_some() { 1 } else { 0 },
        skipped_tests: 0,
        detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_helixtest_overall_report() {
        let raw = r#"{
            "enabled_services": ["Wes", "Drs"],
            "services": [
                {
                    "service": "Wes",
                    "tests": [
                        {"name": "schema", "status": "pass", "passed": true, "error": null},
                        {"name": "skip-me", "status": "skip", "passed": false, "error": "skipped: n/a"}
                    ]
                },
                {
                    "service": "Drs",
                    "tests": [
                        {"name": "checksum", "status": "fail", "passed": false, "error": "mismatch"}
                    ]
                }
            ]
        }"#;
        let r = build_from_helixtest_value(raw, "lab").unwrap();
        assert_eq!(r.enabled_services, vec!["Wes", "Drs"]);
        assert_eq!(r.results.len(), 2);
        assert!(r.results[0].passed);
        assert_eq!(r.results[0].executed_tests, 1);
        assert!(!r.results[1].passed);
        assert!(!r.overall_pass);
        assert!(r.results[1].detail.as_ref().unwrap().contains("mismatch"));
    }

    #[test]
    fn skips_envelope_without_service() {
        let raw = r#"{"ok": true, "results": [{"service": "Beacon", "passed": true}]}"#;
        let r = build_from_helixtest_value(raw, "lab").unwrap();
        assert_eq!(r.results.len(), 1);
        assert!(r.overall_pass);
        assert!(r.had_executed_checks);
    }

    #[test]
    fn empty_json_is_not_a_pass() {
        let err = build_from_helixtest_value("{}", "lab").unwrap_err();
        assert!(err.to_string().contains("empty") || err.to_string().contains("no service"));
    }

    #[test]
    fn skip_only_service_is_not_a_pass() {
        let raw = r#"{
            "services": [{
                "service": "Wes",
                "tests": [{"name": "skip-me", "status": "skip", "passed": false}]
            }]
        }"#;
        let err = build_from_helixtest_value(raw, "lab").unwrap_err();
        assert!(err.to_string().contains("skipped"));
    }
}
