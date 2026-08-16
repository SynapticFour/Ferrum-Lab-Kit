// SPDX-License-Identifier: BUSL-1.1
//! Conformance report generation from HelixTest-style JSON.
//! **PDF** output requires a vendor-signed `flk1.` token in
//! `FERRUM_LAB_KIT_LICENSE_KEY` plus `lab-kit license activate`.

#![forbid(unsafe_code)]

mod error;
mod json_report;
mod license;
#[cfg(feature = "pdf")]
mod pdf_report;

pub use error::ReportError;
pub use json_report::{ConformanceJsonReport, ServiceResultRow};
pub use license::{
    activate_license, default_license_file, hash_license_key, license_key_well_formed,
    pdf_license_granted, verify_signed_license, LicenseActivation, LicensePayload,
    LICENSE_FILE_ENV, LICENSE_PUBKEY_ENV,
};

/// Environment variable holding the raw license key (must also be activated).
pub const LICENSE_KEY_ENV: &str = "FERRUM_LAB_KIT_LICENSE_KEY";

/// Build structured JSON + optional PDF from raw HelixTest JSON file.
pub fn generate_reports(
    helixtest_json_path: &std::path::Path,
    out_dir: &std::path::Path,
    lab_name: &str,
) -> Result<(), ReportError> {
    let raw = std::fs::read_to_string(helixtest_json_path)?;
    let report = json_report::build_from_helixtest_value(&raw, lab_name)?;
    if report.results.is_empty() || !report.had_executed_checks {
        return Err(ReportError::EmptyConformance(
            "HelixTest JSON contained no executed checks — refusing to mark conformance as passed"
                .into(),
        ));
    }
    std::fs::create_dir_all(out_dir)?;
    let json_path = out_dir.join("conformance-report.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(&report)?)?;

    #[cfg(feature = "pdf")]
    {
        let key = std::env::var(LICENSE_KEY_ENV).unwrap_or_default();
        let lic_path = default_license_file();
        match pdf_license_granted(&key, &lic_path) {
            Ok(()) => {
                let pdf_path = out_dir.join("conformance-report.pdf");
                pdf_report::write_pdf(&report, &pdf_path)?;
                tracing::info!(path = %pdf_path.display(), "wrote PDF report (licensed)");
            }
            Err(e) => {
                tracing::warn!(
                    "PDF skipped: {e}. JSON written to {}. Set {} and run `lab-kit license activate`.",
                    json_path.display(),
                    LICENSE_KEY_ENV
                );
            }
        }
    }
    #[cfg(not(feature = "pdf"))]
    {
        tracing::info!("PDF feature disabled at compile time; JSON only.");
    }
    Ok(())
}
