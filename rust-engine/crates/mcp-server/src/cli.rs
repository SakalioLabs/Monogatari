//! Minimal command-line contract for a stdio-only server.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::ServerOptions;

pub const HELP: &str = "Monogatari MCP server\n\nUsage:\n  monogatari-mcp --project-root <path> [--package-output-dir <path>] [--allow-write]\n\nOptions:\n  --project-root <path>        Fixed visual-novel project root containing settings.json\n  --package-output-dir <path>  Fixed directory for .monogatari inspect/validate/output\n  --allow-write                Enable fingerprint-confirmed transactions and package export\n  -h, --help                   Show this help\n  -V, --version                Show the server version";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    Run(ServerOptions),
    Help,
    Version,
}

pub fn parse_args<I>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let mut project_root = None;
    let mut package_output_dir = None;
    let mut allow_write = false;

    while let Some(argument) = args.next() {
        let Some(flag) = argument.to_str() else {
            return Err("Command-line option names must be valid UTF-8.".to_string());
        };
        match flag {
            "-h" | "--help" => return Ok(CliAction::Help),
            "-V" | "--version" => return Ok(CliAction::Version),
            "--allow-write" => {
                if allow_write {
                    return Err("--allow-write may only be supplied once.".to_string());
                }
                allow_write = true;
            }
            "--project-root" => {
                if project_root.is_some() {
                    return Err("--project-root may only be supplied once.".to_string());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--project-root requires a path value.".to_string())?;
                if value.is_empty() {
                    return Err("--project-root requires a non-empty path.".to_string());
                }
                project_root = Some(PathBuf::from(value));
            }
            "--package-output-dir" => {
                if package_output_dir.is_some() {
                    return Err("--package-output-dir may only be supplied once.".to_string());
                }
                let value = args
                    .next()
                    .ok_or_else(|| "--package-output-dir requires a path value.".to_string())?;
                if value.is_empty() {
                    return Err("--package-output-dir requires a non-empty path.".to_string());
                }
                package_output_dir = Some(PathBuf::from(value));
            }
            _ => return Err(format!("Unknown option `{flag}`.")),
        }
    }

    let project_root = project_root.ok_or_else(|| {
        "--project-root is required so tools cannot select an arbitrary filesystem root."
            .to_string()
    })?;
    Ok(CliAction::Run(ServerOptions {
        project_root,
        allow_write,
        package_output_dir,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn requires_one_fixed_project_root() {
        assert!(parse_args(args(&[])).is_err());
        assert!(parse_args(args(&["--project-root"])).is_err());
        assert!(parse_args(args(&["--project-root", "a", "--project-root", "b"])).is_err());

        assert_eq!(
            parse_args(args(&["--project-root", "project", "--allow-write"])).unwrap(),
            CliAction::Run(ServerOptions {
                project_root: PathBuf::from("project"),
                allow_write: true,
                package_output_dir: None,
            })
        );
    }

    #[test]
    fn accepts_one_fixed_package_directory() {
        assert_eq!(
            parse_args(args(&[
                "--project-root",
                "project",
                "--package-output-dir",
                "packages"
            ]))
            .unwrap(),
            CliAction::Run(ServerOptions {
                project_root: PathBuf::from("project"),
                allow_write: false,
                package_output_dir: Some(PathBuf::from("packages")),
            })
        );
        assert!(parse_args(args(&["--project-root", "project", "--package-output-dir"])).is_err());
        assert!(parse_args(args(&[
            "--project-root",
            "project",
            "--package-output-dir",
            "a",
            "--package-output-dir",
            "b"
        ]))
        .is_err());
    }

    #[test]
    fn rejects_unknown_or_duplicate_write_flags() {
        assert!(parse_args(args(&["--project", "data"])).is_err());
        assert!(parse_args(args(&[
            "--project-root",
            "data",
            "--allow-write",
            "--allow-write"
        ]))
        .is_err());
    }
}
