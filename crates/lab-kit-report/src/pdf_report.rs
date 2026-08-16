// SPDX-License-Identifier: BUSL-1.1
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

    let mut page = page1;
    let mut layer = layer1;
    let mut y = Mm(280.0);
    let left = Mm(20.0);
    let line = Mm(6.0);
    let bottom = Mm(20.0);

    let ensure_space = |doc: &PdfDocumentReference,
                        y: &mut Mm,
                        page: &mut PdfPageIndex,
                        layer: &mut PdfLayerIndex| {
        if y.0 > bottom.0 {
            return;
        }
        let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "Continued");
        *page = p;
        *layer = l;
        y.0 = 280.0;
    };

    let text =
        |layer: &PdfLayerReference, font: &IndirectFontRef, x: Mm, y: Mm, s: &str, size: f32| {
            layer.use_text(s, size, x, y, font);
        };

    fn wrap_line(s: &str, width: usize) -> Vec<String> {
        if s.chars().count() <= width {
            return vec![s.to_string()];
        }
        let mut lines = Vec::new();
        let mut current = String::new();
        for word in s.split_whitespace() {
            if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > width {
                lines.push(std::mem::take(&mut current));
            }
            if current.is_empty() {
                if word.chars().count() > width {
                    let mut chunk = String::new();
                    for ch in word.chars() {
                        if chunk.chars().count() >= width {
                            lines.push(std::mem::take(&mut chunk));
                        }
                        chunk.push(ch);
                    }
                    current = chunk;
                } else {
                    current = word.to_string();
                }
            } else {
                current.push(' ');
                current.push_str(word);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }

    {
        let current_layer = doc.get_page(page).get_layer(layer);
        text(
            &current_layer,
            &font_b,
            left,
            y,
            "Ferrum Lab Kit — GA4GH Conformance Summary",
            14.0,
        );
    }
    y.0 -= line.0 * 2.0;

    let lines: Vec<(bool, String, f32)> = {
        let mut v = vec![
            (false, format!("Lab: {}", report.lab_name), 10.0),
            (false, format!("Generated: {}", report.generated_at), 10.0),
            (true, "Services exercised".into(), 11.0),
        ];
        for s in &report.enabled_services {
            v.push((false, format!("- {s}"), 10.0));
        }
        v.push((true, "Per-service results".into(), 11.0));
        for r in &report.results {
            let status = if r.passed { "PASS" } else { "FAIL" };
            v.push((false, format!("{} — {}", r.service, status), 10.0));
        }
        v.push((
            true,
            format!(
                "Overall: {}",
                if report.overall_pass {
                    "PASS"
                } else {
                    "FAIL (see next steps)"
                }
            ),
            11.0,
        ));
        v.push((true, "Next steps".into(), 11.0));
        for step in &report.next_steps {
            for line in wrap_line(step, 95) {
                v.push((false, line, 9.0));
            }
        }
        v
    };

    for (bold, s, size) in lines {
        y.0 -= line.0;
        ensure_space(&doc, &mut y, &mut page, &mut layer);
        let current_layer = doc.get_page(page).get_layer(layer);
        let f = if bold { &font_b } else { &font };
        text(&current_layer, f, left, y, &s, size);
    }

    let file = File::create(path).map_err(ReportError::Io)?;
    let mut w = BufWriter::new(file);
    doc.save(&mut w)
        .map_err(|e| ReportError::Pdf(e.to_string()))?;
    Ok(())
}
