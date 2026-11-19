use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_paper-graph"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run paper-graph {args:?}: {error}"))
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn help_and_version_are_successful_clap_paths() {
    for args in [vec!["--help"], vec!["--version"], vec!["sync", "--help"]] {
        let output = run(&args);
        assert!(output.status.success(), "{args:?}: {}", stderr(&output));
    }
    let sync_help = run(&["sync", "--help"]);
    assert!(String::from_utf8_lossy(&sync_help.stdout).contains("--limit"));
}

#[test]
fn missing_command_and_invalid_arguments_exit_nonzero() {
    for args in [
        vec![],
        vec!["not-a-command"],
        vec!["search"],
        vec!["search", "needle", "--mode", "unknown"],
        vec!["ui", "--port", "not-a-port"],
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            stderr(&output)
        );
        if args.is_empty() {
            assert!(
                stderr(&output).contains("Usage:"),
                "missing command did not print clap usage"
            );
        } else {
            assert!(
                stderr(&output).contains("error:"),
                "{args:?}: missing clap error"
            );
        }
    }
    // Seeded defect: replacing clap's parse error with a successful no-op makes
    // the exact exit-code assertion fail.
}

#[test]
fn limit_is_rejected_for_every_non_destructive_command() {
    for args in [
        vec!["probe", "--limit", "1"],
        vec!["stats", "--limit", "1"],
        vec!["search", "needle", "--limit", "1"],
        vec!["author", "Ada", "--limit", "1"],
        vec!["ui", "--limit", "1"],
    ] {
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            stderr(&output)
        );
        assert!(
            stderr(&output).contains("unexpected argument '--limit'"),
            "{args:?}: {}",
            stderr(&output)
        );
    }
    // Seeded defect: moving limit into SharedArgs would allow all five commands
    // through parsing; these invocations would no longer produce clap exit 2.
}
