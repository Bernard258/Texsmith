use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
};

mod names;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Template {
    name: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}
include!(concat!(env!("OUT_DIR"), "/templates.rs"));

const HELP: &str = "Texsmith - a standalone LaTeX report helper

USAGE
    texsmith [--root PATH | -r PATH] <command>

COMMANDS
    new, n NAME [TITLE] [-t NAME]
        Create a report (default template: default)

    templates, t
        List embedded templates

    list, ls, l
        List report folders

    build, b [DIR ...]
        Compile the current directory or the specified directories

    build, b --all
        Compile child report folders (-a is short for --all)

    help, h
        Show this help (-h and --help also work)

OPTIONS
    -t, --template NAME
        Template to use with the new command

    -r, --root PATH
        Use PATH instead of the current directory

    -a, --all
        Build all child report folders

EXAMPLES
    texsmith n my-paper -t ieee-conference
    texsmith b
    texsmith b my-paper ../other-paper
    texsmith b -a

NOTES
    Run locally with: cargo texsmith <command>
    Reports are created as NAME/main.tex in the current directory.
    List and build use the current directory; --root PATH overrides it.
    PDFs and logs are written to each report's build/ folder.
    Set TECTONIC to override the executable.";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args: Vec<std::ffi::OsString> = env::args_os().skip(1).collect();
    if args.is_empty() || matches!(args[0].to_str(), Some("help" | "h" | "--help" | "-h")) {
        println!("{HELP}");
        return Ok(());
    }
    let root = if matches!(args[0].to_str(), Some("--root" | "-r")) {
        if args.len() < 3 {
            return Err("--root requires a path and command".into());
        }
        let root = fs::canonicalize(&args[1])?;
        args.drain(..2);
        root
    } else {
        env::current_dir()?
    };
    if !root.is_dir() {
        return Err("--root must point to a directory".into());
    }
    if matches!(args[0].to_str(), Some("build" | "b")) {
        return build_many(build_targets(&root, &args[1..])?);
    }
    let text_args = args
        .iter()
        .map(|arg| {
            arg.to_str()
                .ok_or("command, report name, and title must be valid Unicode")
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    match text_args.as_slice() {
        ["help" | "h" | "--help" | "-h"] => {
            println!("{HELP}");
            Ok(())
        }
        ["new" | "n", rest @ ..] => {
            let (name, title, template) = parse_new(rest)?;
            create(&root, name, title, template)
        }
        ["templates" | "t"] => {
            for template in TEMPLATES {
                println!("{}", template.name);
            }
            Ok(())
        }
        ["list" | "ls" | "l"] => {
            let reports = discover(&root)?;
            if reports.is_empty() {
                println!("No reports yet. Run: texsmith new my-report \"My Report\"");
            }
            for path in reports {
                println!("{}", path.file_name().unwrap().to_string_lossy());
            }
            Ok(())
        }
        _ => Err(format!("invalid arguments\n\n{HELP}").into()),
    }
}

fn validate_name(name: &str) -> Result<()> {
    if !names::report_name(name) {
        return Err("report names must start with a letter or number and contain only letters, numbers, - or _; device names and src/target/templates are reserved".into());
    }
    Ok(())
}

fn escape_tex(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\\' => "\\textbackslash{}".into(),
            '{' => "\\{".into(),
            '}' => "\\}".into(),
            '$' => "\\$".into(),
            '&' => "\\&".into(),
            '#' => "\\#".into(),
            '%' => "\\%".into(),
            '_' => "\\_".into(),
            '~' => "\\textasciitilde{}".into(),
            '^' => "\\textasciicircum{}".into(),
            '\n' | '\r' => " ".into(),
            _ => c.to_string(),
        })
        .collect()
}

fn parse_new<'a>(args: &[&'a str]) -> Result<(&'a str, &'a str, &'a str)> {
    let mut positional = Vec::new();
    let mut template = None;
    let mut args = args.iter().copied();
    while let Some(arg) = args.next() {
        if matches!(arg, "--template" | "-t") {
            if template.is_some() {
                return Err("--template may only be specified once".into());
            }
            template = Some(args.next().ok_or("--template requires a name")?);
        } else if arg.starts_with('-') {
            return Err(format!("unknown option: {arg}").into());
        } else {
            positional.push(arg);
        }
    }
    match positional.as_slice() {
        [name] => Ok((name, name, template.unwrap_or("default"))),
        [name, title] => Ok((name, title, template.unwrap_or("default"))),
        _ => Err("usage: new NAME [TITLE] [--template NAME]".into()),
    }
}

// Only generated embedded data is consulted at runtime: no template folder is needed.
fn template_files(name: &str, title: &str) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    validate_name(name)?;
    let template = TEMPLATES
        .iter()
        .find(|template| template.name == name)
        .ok_or_else(|| format!("unknown template: {name}; run texsmith templates"))?;
    let title = escape_tex(title);
    template
        .files
        .iter()
        .map(|(relative, contents)| {
            let path = PathBuf::from(relative);
            let contents = if path.extension().is_some_and(|ext| ext == "tex") {
                std::str::from_utf8(contents)?
                    .replace("@@TITLE@@", &title)
                    .into_bytes()
            } else {
                contents.to_vec()
            };
            Ok((path, contents))
        })
        .collect()
}

fn create(root: &Path, name: &str, title: &str, template: &str) -> Result<()> {
    validate_name(name)?;
    let files = template_files(template, title)?;
    let path = root.join(name);
    fs::create_dir(&path).map_err(|e| format!("cannot create {}: {e}", path.display()))?;
    let result = (|| -> Result<()> {
        for (relative, contents) in files {
            let destination = path.join(relative);
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::write(destination, contents)?;
        }
        fs::create_dir_all(path.join("figures"))?;
        if !path.join("figures/.gitkeep").exists() {
            fs::write(path.join("figures/.gitkeep"), "")?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        // Only remove the new folder that this call created, never an existing report.
        if let Err(cleanup) = fs::remove_dir_all(&path) {
            eprintln!(
                "could not remove incomplete report {}: {cleanup}",
                path.display()
            );
        }
        return Err(error);
    }
    println!(
        "Created {} using template {template}. Edit main.tex, then run: texsmith build {name}",
        path.display()
    );
    Ok(())
}

fn discover(root: &Path) -> Result<Vec<PathBuf>> {
    let mut reports = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && validate_name(&entry.file_name().to_string_lossy()).is_ok()
            && entry.path().join("main.tex").is_file()
        {
            reports.push(entry.path());
        }
    }
    reports.sort();
    Ok(reports)
}

fn build_targets(root: &Path, args: &[std::ffi::OsString]) -> Result<Vec<PathBuf>> {
    if args.is_empty() {
        return Ok(vec![root.to_path_buf()]);
    }
    if args.len() == 1 && matches!(args[0].to_str(), Some("--all" | "-a")) {
        let reports = discover(root)?;
        if reports.is_empty() {
            return Err(format!("no child report folders found in {}", root.display()).into());
        }
        return Ok(reports);
    }
    let mut literal_paths = false;
    let mut paths = Vec::new();
    for arg in args {
        if !literal_paths && arg == "--" {
            literal_paths = true;
        } else if !literal_paths && matches!(arg.to_str(), Some("--all" | "-a")) {
            return Err("--all/-a cannot be combined with directory arguments".into());
        } else if !literal_paths && arg.to_string_lossy().starts_with('-') {
            return Err(format!(
                "unknown build option: {}; use -- before paths starting with -",
                arg.to_string_lossy()
            )
            .into());
        } else if arg.is_empty() {
            return Err("report directory paths cannot be empty".into());
        } else {
            paths.push(root.join(arg));
        }
    }
    if paths.is_empty() {
        return Err("expected a directory after --".into());
    }
    Ok(paths)
}

fn build_many(paths: Vec<PathBuf>) -> Result<()> {
    let mut failures = Vec::new();
    for path in paths {
        if let Err(error) = build(&path) {
            eprintln!("error: {error}");
            failures.push(path.display().to_string());
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("failed reports: {}", failures.join(", ")).into())
    }
}

// Resolve an explicit relative executable before changing the child's directory.
// Bare executable names retain the operating system's PATH lookup (.exe on Windows).
fn tectonic_executable() -> Result<PathBuf> {
    let program = PathBuf::from(env::var_os("TECTONIC").unwrap_or_else(|| "tectonic".into()));
    if program.as_os_str().is_empty() {
        return Err("TECTONIC must name an executable".into());
    }
    if program.is_absolute() || program.components().count() == 1 {
        Ok(program)
    } else {
        Ok(env::current_dir()?.join(program))
    }
}

fn build(path: &Path) -> Result<()> {
    let path = fs::canonicalize(path)
        .map_err(|e| format!("cannot access report directory {}: {e}", path.display()))?;
    if !path.is_dir() || !path.join("main.tex").is_file() {
        return Err(format!(
            "{} must be a report folder containing main.tex",
            path.display()
        )
        .into());
    }
    let output = path.join("build");
    fs::create_dir_all(&output)?;
    println!("Building {}", path.display());
    let executable = tectonic_executable()?;
    let status = Command::new(executable)
        .current_dir(&path)
        .args(["--keep-logs", "--synctex", "--outdir", "build", "main.tex"])
        .status()
        .map_err(|e| format!("could not start Tectonic: {e}; install it or set TECTONIC"))?;
    if !status.success() {
        return Err(format!("Tectonic failed for {} ({status})", path.display()).into());
    }
    println!("PDF: {}", output.join("main.pdf").display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn names_cannot_escape_workspace_or_collide_with_tooling() {
        for name in [
            "",
            "..",
            "../outside",
            "/tmp/report",
            "a/b",
            "--all",
            "templates",
            "src",
            "target",
        ] {
            assert!(validate_name(name).is_err(), "{name}");
        }
        assert!(validate_name("2026-example_report").is_ok());
    }
    #[test]
    fn title_is_literal_latex_text() {
        assert_eq!(
            escape_tex("A&B_50% {x}\\"),
            "A\\&B\\_50\\% \\{x\\}\\textbackslash{}"
        );
    }
}
