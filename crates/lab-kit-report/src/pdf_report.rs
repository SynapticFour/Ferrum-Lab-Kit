use std::fs::File;
use std::io::BufWriter;

use printpdf::*;

use crate::json_report::ConformanceJsonReport;
use crate::ReportError;

pub fn write_pdf(
    report: &ConformanceJsonReport,
    path: &std::path::Path,
) -> Result<(), ReportError> {
    let (doc, page1, layer1) = PdfDocument::new(
        "Ferrum Lab Kit — Conformance Report",
        Mm(210.0),
        Mm(297.0),
        "Summary",
    );
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    let font_b = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| ReportError::Pdf(e.to_string()))?;

    let current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = Mm(280.0);
    let left = Mm(20.0);
    let line = Mm(6.0);

    fn text(layer: &PdfLayerReference, font: &IndirectFontRef, x: Mm, y: Mm, s: &str, size: f32) {
        layer.use_text(s, size, x, y, font);
    }

    text(
        &current_layer,
        &font_b,
        left,
        y,
        "Ferrum Lab Kit — GA4GH Conformance Summary",
        14.0,
    );
    y.0 -= line.0 * 2.0;
    text(
        &current_layer,
        &font,
        left,
        y,
        &format!("Lab: {}", report.lab_name),
        10.0,
    );
    y.0 -= line.0;
    text(
        &current_layer,
        &font,
        left,
        y,
        &format!("Generated: {}", report.generated_at),
        10.0,
    );
    y.0 -= line.0 * 2.0;
    text(&current_layer, &font_b, left, y, "Services exercised", 11.0);
    y.0 -= line.0;
    for s in &report.enabled_services {
        y.0 -= line.0;
        text(&current_layer, &font, left, y, &format!("- {s}"), 10.0);
    }
    y.0 -= line.0 * 2.0;
    text(
        &current_layer,
        &font_b,
        left,
        y,
        "Per-service results",
        11.0,
    );
    for r in &report.results {
        y.0 -= line.0;
        let status = if r.passed { "PASS" } else { "FAIL" };
        text(
            &current_layer,
            &font,
            left,
            y,
            &format!("{} — {}", r.service, status),
            10.0,
        );
    }
    y.0 -= line.0 * 2.0;
    text(
        &current_layer,
        &font_b,
        left,
        y,
        &format!(
            "Overall: {}",
            if report.overall_pass {
                "PASS"
            } else {
                "FAIL (see next steps)"
            }
        ),
        11.0,
    );
    y.0 -= line.0 * 2.0;
    text(&current_layer, &font_b, left, y, "Next steps", 11.0);
    for step in &report.next_steps {
        y.0 -= line.0;
        text(&current_layer, &font, left, y, step.as_str(), 9.0);
    }

    let file = File::create(path).map_err(ReportError::Io)?;
    let mut w = BufWriter::new(file);
    doc.save(&mut w)
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    Ok(())
}
