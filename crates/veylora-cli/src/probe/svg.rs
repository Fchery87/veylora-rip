//! Diagnostic visualization of shared geometry, never a device raster.
use veylora_diagnostics::targets::{DiagnosticTarget, Shape, TargetKind};

pub fn render(target: &DiagnosticTarget) -> Result<String, String> {
    if target.kind() != TargetKind::ChannelIdentity {
        return Err("SVG adapter only supports channel identity targets".into());
    }
    let size = target.size();
    let mut svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}mm\" height=\"{}mm\" viewBox=\"0 0 {} {}\"><title>DIAGNOSTIC ONLY - NOT PRINTER-READY</title><desc>Logical coverage visualization. Approximate RGB colors, gray background for white. No physical channel mapping, screening or protocol encoding. Text uses viewer sans-serif, not fixed production glyphs.</desc><rect width=\"100%\" height=\"100%\" fill=\"#ddd\"/>", size.width_mm(), size.height_mm(), size.width_mm(), size.height_mm());
    for object in target.objects() {
        let b = object.bounds;
        let c = object.coverage;
        let rgb = |value: f64| ((1.0 - value) * (1.0 - c.k) * 255.0).round() as u8;
        let color = format!("#{:02x}{:02x}{:02x}", rgb(c.c), rgb(c.m), rgb(c.y));
        svg.push_str(&format!(
            "<g transform=\"translate({} {})\" fill=\"{}\" opacity=\"{}\"><title>{}</title>",
            b.x_mm,
            b.y_mm,
            color,
            object.alpha,
            escape(&object.label)
        ));
        let w = b.width_mm;
        let h = b.height_mm;
        match &object.shape {
            Shape::Rectangle => svg.push_str(&format!("<rect width=\"{w}\" height=\"{h}\"/>")),
            Shape::Text { text } => svg.push_str(&format!("<svg width=\"{w}\" height=\"{h}\" overflow=\"hidden\"><text x=\"0\" y=\"{}\" font-family=\"sans-serif\" font-size=\"{}\" textLength=\"{w}\" lengthAdjust=\"spacingAndGlyphs\">{}</text></svg>", h * 0.8, h * 0.8, escape(text))),
            Shape::AsymmetricMarker => svg.push_str(&format!("<path d=\"M0 0 H{} V{} H{w} V{h} H0 Z\"/>", w / 4.0, h * 0.75)),
            Shape::Ruler { vertical, tick_mm } => {
                let length = if *vertical { h } else { w };
                let count = (length / tick_mm).floor() as usize;
                if count > 1000 { return Err("SVG ruler exceeds diagnostic bound".into()); }
                for index in 0..=count {
                    let position = index as f64 * tick_mm;
                    let thickness = 0.15_f64.min(length - position);
                    if *vertical { svg.push_str(&format!("<rect x=\"0\" y=\"{position}\" width=\"{w}\" height=\"{thickness}\"/>")); }
                    else { svg.push_str(&format!("<rect x=\"{position}\" y=\"0\" width=\"{thickness}\" height=\"{h}\"/>")); }
                }
            }
            _ => return Err("unsupported SVG diagnostic shape".into()),
        }
        svg.push_str("</g>");
    }
    svg.push_str("<text x=\"5\" y=\"55\" font-family=\"sans-serif\" font-size=\"2.8\">DIAGNOSTIC ONLY - NOT PRINTER-READY</text><text x=\"5\" y=\"60\" font-family=\"sans-serif\" font-size=\"2.3\">Approximate logical colors. Not physical output.</text><text x=\"5\" y=\"65\" font-family=\"sans-serif\" font-size=\"2.3\">W shown white on gray. K + W shown black.</text></svg>");
    if svg.len() > 1024 * 1024 {
        return Err("SVG exceeds 1 MiB bound".into());
    }
    Ok(svg)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
