/// Reject file components that cannot be moved between supported platforms.
pub fn portable_component(name: &str) -> bool {
    if name.is_empty()
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return false;
    }
    let stem = name.split('.').next().unwrap().to_ascii_uppercase();
    !matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) && !["COM", "LPT"].iter().any(|prefix| {
        stem.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    })
}

#[allow(dead_code)]
pub fn report_name(name: &str) -> bool {
    portable_component(name)
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        && !matches!(
            name.to_ascii_lowercase().as_str(),
            "src" | "target" | "templates"
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_names_reject_windows_devices_on_every_os() {
        for name in [
            "CON",
            "con",
            "NUL",
            "COM1",
            "Lpt9",
            "aux.txt",
            "trailing.",
            "trailing ",
            "a:b",
            "a\\b",
        ] {
            assert!(!portable_component(name), "{name}");
        }
        assert!(report_name("2026-paper"));
        assert!(!report_name("Templates"));
        assert!(portable_component("résumé.tex"));
    }
}
