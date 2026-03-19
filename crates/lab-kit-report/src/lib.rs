//! Conformance report generation from HelixTest-style JSON.
//! **PDF** output requires `FERRUM_LAB_KIT_LICENSE_KEY` at runtime (open-core boundary).

#![forbid(unsafe_code)]

mod error;
mod json_report;
#[cfg(feature = "pdf")]
mod pdf_report;

pub use error::ReportError;
pub use json_report::{ConformanceJsonReport, ServiceResultRow};

/// Environment variable checked before emitting PDF (commercial tier).
pub const LICENSE_KEY_ENV: &str = "FERRUM_LAB_KIT_LICENSE_KEY";

/// Build structured JSON + optional PDF from raw HelixTest JSON file.
pub fn generate_reports(
    helixtest_json_path: &std::path::Path,
    out_dir: &std::path::Path,
    lab_name: &str,
) -> Result<(), ReportError> {
    let raw = std::fs::read_to_string(helixtest_json_path)?;
    let report = json_report::build_from_helixtest_value(&raw, lab_name)?;
    std::fs::create_dir_all(out_dir)?;
    let json_path = out_dir.join("conformance-report.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(&report)?)?;

    #[cfg(feature = "pdf")]
    {
        if std::env::var(LICENSE_KEY_ENV)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
        {
            let pdf_path = out_dir.join("conformance-report.pdf");
            pdf_report::write_pdf(&report, &pdf_path)?;
            tracing::info!(path = %pdf_path.display(), "wrote PDF report (licensed)");
        } else {
            tracing::warn!(
                "PDF skipped: set {} for licensed PDF output. JSON written to {}",
                LICENSE_KEY_ENV,
                json_path.display()
            );
        }
    }
    #[cfg(not(feature = "pdf"))]
    {
        tracing::info!("PDF feature disabled at compile time; JSON only.");
    }
    Ok(())
}
