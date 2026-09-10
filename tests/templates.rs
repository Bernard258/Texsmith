use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

// Serialize relocation tests: a concurrent process spawn can briefly inherit an
// open executable-copy descriptor on Unix, producing ETXTBSY in another test.
static RELOCATION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "texsmith-test-é space-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        // Only the executable is shipped into this workspace.
        fs::copy(
            env!("CARGO_BIN_EXE_texsmith"),
            path.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)),
        )
        .unwrap();
        Self(path)
    }
    fn compiler(&self) -> PathBuf {
        let path = self
            .0
            .join(format!("test compiler{}", std::env::consts::EXE_SUFFIX));
        let result = Command::new("rustc")
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/compiler.rs"))
            .arg("-o")
            .arg(&path)
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        path
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(
            self.0
                .join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)),
        )
        .current_dir(&self.0)
        .args(args)
        .output()
        .unwrap()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn relocated_binary_lists_and_extracts_every_template_without_source_files() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    let output = w.run(&["templates"]);
    assert!(output.status.success());
    let names = String::from_utf8(output.stdout).unwrap();
    assert!(names.lines().any(|name| name == "apa"));
    assert!(names.lines().any(|name| name == "mla"));
    for name in names.lines() {
        let report = format!("{name} paper");
        let output = w.run(&["new", &report, "--template", name]);
        assert!(output.status.success(), "{output:?}");
        let main = fs::read_to_string(w.0.join(&report).join("main.tex")).unwrap();
        assert!(main.contains(&report.replace('&', "\\&")));
        assert!(!main.contains("@@TITLE@@"));
        assert!(w.0.join(&report).join("figures").is_dir());
        if name == "ieee-conference" {
            assert!(w.0.join(&report).join("references.bib").is_file());
            assert!(w.0.join(&report).join("sections/body.tex").is_file());
        }
    }
    assert!(!w.0.join("templates").exists());
}

#[test]
fn runtime_templates_cannot_override_embedded_defaults() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    fs::create_dir_all(w.0.join("templates/default")).unwrap();
    fs::write(w.0.join("templates/default/main.tex"), "runtime override").unwrap();
    assert!(w.run(&["new", "one"]).status.success());
    let original = fs::read(w.0.join("one/main.tex")).unwrap();
    assert!(String::from_utf8_lossy(&original).contains("\\title{one}"));
    assert!(!w.run(&["new", "one"]).status.success());
    assert_eq!(fs::read(w.0.join("one/main.tex")).unwrap(), original);
    assert!(
        w.run(&["new", "--template", "brief", "two"])
            .status
            .success()
    );
}

#[test]
fn bad_arguments_leave_no_report() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    for args in [
        vec!["new", "bad", "--template", "missing"],
        vec!["new", "bad", "--template", "../default"],
        vec!["new", "bad", "--template"],
        vec!["new", "bad", "--template", "brief", "--template", "default"],
        vec!["new", "bad", "--unknown"],
        vec!["new", "../outside"],
    ] {
        assert!(!w.run(&args).status.success());
        assert!(!w.0.join("bad").exists());
    }
}

#[test]
fn current_directory_wins_over_ancestor_markers_and_binary_location() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    fs::write(w.0.join(".reports-root"), "legacy marker").unwrap();
    let working = w.0.join("nested working directory");
    fs::create_dir(&working).unwrap();
    let output = Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
        .current_dir(&working)
        .args(["new", "local-paper"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(working.join("local-paper/main.tex").is_file());
    assert!(!w.0.join("local-paper").exists());
    assert!(!working.join(".reports-root").exists());
    let output = Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
        .current_dir(&working)
        .arg("list")
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "local-paper\n");
    let other = working.join("explicit destination");
    fs::create_dir(&other).unwrap();
    let output = Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
        .current_dir(&working)
        .args(["--root", "explicit destination", "new", "selected-paper"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(other.join("selected-paper/main.tex").is_file());
    assert!(!working.join("selected-paper").exists());
}

#[test]
fn shorthand_commands_and_options_match_long_forms() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    assert_eq!(w.run(&["t"]).stdout, w.run(&["templates"]).stdout);
    for alias in ["h", "-h", "--help"] {
        assert_eq!(w.run(&[alias]).stdout, w.run(&["help"]).stdout);
    }
    let output = w.run(&["n", "short", "-t", "brief"]);
    assert!(output.status.success(), "{output:?}");
    let output = w.run(&["n", "My", "Unquoted", "Paper", "-t", "default"]);
    assert!(output.status.success(), "{output:?}");
    let output = w.run(&["n", "My Quoted Paper", "-t", "default"]);
    assert!(output.status.success(), "{output:?}");
    assert!(
        w.run(&["new", "long", "--template", "brief"])
            .status
            .success()
    );
    let short = fs::read_to_string(w.0.join("short/main.tex")).unwrap();
    let long = fs::read_to_string(w.0.join("long/main.tex")).unwrap();
    assert!(short.contains("\\title{short}"));
    assert!(long.contains("\\title{long}"));
    let unquoted = fs::read_to_string(w.0.join("My Unquoted Paper/main.tex")).unwrap();
    assert!(unquoted.contains("\\title{My Unquoted Paper}"));
    let quoted = fs::read_to_string(w.0.join("My Quoted Paper/main.tex")).unwrap();
    assert!(quoted.contains("\\title{My Quoted Paper}"));
    for alias in ["ls", "l"] {
        assert_eq!(w.run(&[alias]).stdout, w.run(&["list"]).stdout);
    }
    fs::create_dir(w.0.join("destination")).unwrap();
    assert!(
        w.run(&["-r", "destination", "n", "paper", "-t", "mla"])
            .status
            .success()
    );
    assert!(w.0.join("destination/paper/main.tex").is_file());
    for args in [
        vec!["n", "bad", "-t"],
        vec!["n", "bad", "-t", "brief", "--template", "default"],
        vec!["n", "bad", "-x"],
        vec!["-r"],
    ] {
        assert!(!w.run(&args).status.success());
        assert!(!w.0.join("bad").exists());
    }
}

#[test]
fn build_aliases_invoke_compiler_in_report_folders() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    assert!(w.run(&["n", "one"]).status.success());
    assert!(w.run(&["n", "two"]).status.success());
    let compiler = w.compiler();
    for args in [
        vec!["b", "one"],
        vec!["b", "-a"],
        vec!["build", "-a"],
        vec!["b", "--all"],
    ] {
        let output = Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
            .current_dir(&w.0)
            .env("TECTONIC", &compiler)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
    for name in ["one", "two"] {
        let directory = w.0.join(name);
        assert_eq!(
            fs::canonicalize(
                fs::read_to_string(directory.join("build/invoked-from"))
                    .unwrap()
                    .trim()
            )
            .unwrap(),
            fs::canonicalize(&directory).unwrap()
        );
        let args = fs::read_to_string(directory.join("build/invoked-with")).unwrap();
        assert!(args.ends_with("--outdir\nbuild\nmain.tex\n"));
    }
}

#[test]
fn builds_current_directory_and_multiple_paths_and_continues_after_failures() {
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    let compiler = w.compiler();
    for name in ["current paper", "other paper", "broken", "-literal"] {
        fs::create_dir(w.0.join(name)).unwrap();
        fs::write(w.0.join(name).join("main.tex"), "test").unwrap();
    }
    let current = w.0.join("current paper");
    let other = w.0.join("other paper");
    let run = |args: &[&str]| {
        Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
            .current_dir(&current)
            .env("TECTONIC", &compiler)
            .args(args)
            .output()
            .unwrap()
    };
    for args in [vec!["b"], vec!["build", "."]] {
        let output = run(&args);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            fs::canonicalize(
                fs::read_to_string(current.join("build/visited"))
                    .unwrap()
                    .trim()
            )
            .unwrap(),
            fs::canonicalize(&current).unwrap()
        );
    }
    let output = run(&["b", ".", "../other paper", other.to_str().unwrap()]);
    assert!(output.status.success(), "{output:?}");
    assert!(other.join("build/visited").is_file());
    fs::remove_file(other.join("build/visited")).unwrap();
    fs::write(w.0.join("broken/fail"), "").unwrap();
    let output = run(&["b", "../missing", "../broken", "../other paper"]);
    assert!(!output.status.success());
    let errors = String::from_utf8(output.stderr).unwrap();
    assert!(errors.contains("missing") && errors.contains("broken"));
    assert!(other.join("build/visited").is_file());
    assert!(!w.0.join("missing").exists());
    let output = run(&["-r", "../other paper", "b"]);
    assert!(output.status.success(), "{output:?}");
    let output = run(&["-r", "..", "b", "--", "-literal"]);
    assert!(output.status.success(), "{output:?}");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&other, current.join("linked")).unwrap();
        assert!(run(&["b", "linked"]).status.success());
    }
    // Relative compiler paths must be resolved against the caller, not the report.
    let output = Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
        .current_dir(&w.0)
        .env(
            "TECTONIC",
            PathBuf::from(".").join(compiler.file_name().unwrap()),
        )
        .args(["b", "other paper"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    // Invalid option combinations are rejected before any compilation starts.
    fs::remove_file(current.join("build/visited")).unwrap();
    for args in [
        vec!["b", ".", "-a"],
        vec!["b", "--all", "."],
        vec!["b", "--"],
        vec!["b", "-x"],
        vec!["b", ""],
    ] {
        assert!(!run(&args).status.success());
        assert!(!current.join("build/visited").exists());
    }
}

#[cfg(unix)]
#[test]
fn non_unicode_directory_arguments_do_not_panic() {
    use std::os::unix::ffi::OsStringExt;
    let _guard = RELOCATION_LOCK.lock().unwrap();
    let w = Workspace::new();
    let directory =
        w.0.join(std::ffi::OsString::from_vec(b"paper-\xff".to_vec()));
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("main.tex"), "test").unwrap();
    let compiler = w.compiler();
    let output = Command::new(w.0.join(format!("texsmith{}", std::env::consts::EXE_SUFFIX)))
        .current_dir(&w.0)
        .env("TECTONIC", compiler)
        .arg("b")
        .arg(&directory)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(directory.join("build/visited").is_file());
}
