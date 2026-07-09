fn main() {
    let git_commit = git_output(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let git_short_commit =
        git_output(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| git_commit.clone());

    println!("cargo:rustc-env=MONOGATARI_GIT_COMMIT={git_commit}");
    println!("cargo:rustc-env=MONOGATARI_GIT_SHORT_COMMIT={git_short_commit}");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../.git/HEAD");
    if let Some(head_ref) = git_output(&["symbolic-ref", "--quiet", "HEAD"]) {
        println!("cargo:rerun-if-changed=../../../.git/{head_ref}");
    }

    tauri_build::build()
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!value.is_empty()).then_some(value)
}
