// build.rs
use std::process::Command;

fn main() {
    // 1. Run `git` to get the current revision.
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    // 2. Extract the hash, providing a fallback if git fails.
    let git_hash = match output {
        Ok(output) if output.status.success() => {
            // Trim the trailing newline
            String::from_utf8(output.stdout).unwrap().trim().to_string()
        }
        _ => {
            // Fallback if git isn't installed, not in a git repo, etc.
            "unknown".to_string()
        }
    };

    // 3. Set the `GIT_REVISION` environment variable for the crate.
    //    The `env!` macro in `main.rs` will read this.
    println!("cargo:rustc-env=GIT_REVISION={}", git_hash);

    // 4. (Optional but recommended) Tell Cargo to re-run this script if the Git HEAD changes.
    //    This ensures the hash is always up-to-date.
    println!("cargo:rerun-if-changed=.git/HEAD");
}
