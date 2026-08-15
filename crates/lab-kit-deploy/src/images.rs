//! Default Ferrum container image pins (must not float `:latest` for generated artefacts).

pub fn default_ferrum_image() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| first_pin(include_str!("../../../config/ci/ferrum-image.txt")))
        .as_str()
}

pub fn default_ferrum_image_arm64() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| first_pin(include_str!("../../../config/ci/ferrum-image-arm64.txt")))
        .as_str()
}

fn first_pin(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("ghcr.io/synapticfour/ferrum:6788bfe11860b5fe49bae72d120373f78a0b023f")
        .to_string()
}
