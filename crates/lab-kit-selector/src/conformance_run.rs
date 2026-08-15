//! Build the HelixTest argv. Bare `helixtest` is a successful no-op — always pass `--all`.

pub fn helix_test_command_args(enabled_services: &[String]) -> Vec<String> {
    let mut args = vec![
        "--all".into(),
        "--mode".into(),
        "ferrum".into(),
        "--report".into(),
        "json".into(),
    ];
    for svc in enabled_services {
        let only = match svc.to_ascii_lowercase().as_str() {
            "drs" => "drs",
            "htsget" => "htsget",
            "wes" => "wes",
            "tes" => "tes",
            "beacon" => "beacon",
            "trs" => "trs",
            "auth" => "auth",
            _ => continue,
        };
        args.push("--only".into());
        args.push(only.into());
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_passes_all_and_mode() {
        let args = helix_test_command_args(&["beacon".into()]);
        assert!(args
            .windows(2)
            .any(|w| w == ["--all".to_string(), "--mode".to_string()] || w[0] == "--all"));
        assert!(args.contains(&"--all".to_string()));
        assert!(args.contains(&"ferrum".to_string()));
        assert!(args.contains(&"--only".to_string()));
        assert!(args.contains(&"beacon".to_string()));
        assert!(!args.is_empty());
    }

    #[test]
    fn empty_enabled_still_requires_all() {
        let args = helix_test_command_args(&[]);
        assert_eq!(args[0], "--all");
    }
}
