//! Pure portable-path rules shared by archive manifests and ZIP entries.

use std::collections::BTreeSet;

const MAX_PORTABLE_PATH_BYTES: usize = 512;
const MAX_PORTABLE_SEGMENTS: usize = 32;
const MAX_PORTABLE_SEGMENT_BYTES: usize = 160;

pub(super) fn validate_portable_path(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > MAX_PORTABLE_PATH_BYTES
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(format!(
            "{label} `{value}` is not a portable relative path."
        ));
    }
    let segments = value.split('/').collect::<Vec<_>>();
    if segments.is_empty() || segments.len() > MAX_PORTABLE_SEGMENTS {
        return Err(format!("{label} `{value}` has too many path segments."));
    }
    for segment in segments {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.len() > MAX_PORTABLE_SEGMENT_BYTES
            || segment.ends_with(' ')
            || segment.ends_with('.')
            || segment
                .chars()
                .any(|ch| matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
            || is_reserved_windows_segment(segment)
        {
            return Err(format!(
                "{label} `{value}` contains an unsafe path segment."
            ));
        }
    }
    Ok(())
}

pub(super) fn is_reserved_windows_segment(segment: &str) -> bool {
    let stem = segment
        .split('.')
        .next()
        .unwrap_or(segment)
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

pub(super) fn portable_case_key(value: &str) -> String {
    value.to_lowercase()
}

pub(super) fn add_directory_and_parents(target: &mut BTreeSet<String>, directory: &str) {
    let mut current = String::new();
    for segment in directory.split('/') {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(segment);
        target.insert(current.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_paths_accept_nested_content_and_build_case_keys() {
        validate_portable_path("assets/portraits/Guide.PNG", "Asset").unwrap();
        assert_eq!(portable_case_key("Assets/Guide.PNG"), "assets/guide.png");

        let mut directories = BTreeSet::new();
        add_directory_and_parents(&mut directories, "assets/portraits");
        assert_eq!(
            directories.into_iter().collect::<Vec<_>>(),
            ["assets", "assets/portraits"]
        );
    }

    #[test]
    fn portable_paths_reject_escape_reserved_and_platform_specific_shapes() {
        for path in [
            "../settings.json",
            "assets\\guide.png",
            "assets/CON.json",
            "assets/trailing. ",
            "C:/project.json",
        ] {
            assert!(validate_portable_path(path, "Asset").is_err(), "{path}");
        }
    }
}
