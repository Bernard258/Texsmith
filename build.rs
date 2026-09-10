use std::{
    env,
    error::Error,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

#[path = "src/names.rs"]
mod names;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("templates");
    println!("cargo:rerun-if-changed=src/names.rs");
    println!("cargo:rerun-if-changed={}", source.display());
    let mut generated = String::from(
        "// Generated from templates/ by build.rs. Do not edit.\nstatic TEMPLATES: &[Template] = &[\n",
    );
    let mut default_found = false;
    for entry in entries(&source)? {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "template names must be UTF-8")?;
        if matches!(name.as_str(), ".git" | "build" | "target") {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_file() {
            continue;
        } // Allow catalog documentation next to template folders.
        if !kind.is_dir() {
            return Err(format!("unsupported template entry: {}", entry.path().display()).into());
        }
        if !names::report_name(&name) {
            return Err(format!("invalid or non-portable template name: {name}").into());
        }
        let mut files = Vec::new();
        collect(&entry.path(), Path::new(""), &mut files)?;
        if !files.iter().any(|(path, _)| path == "main.tex") {
            return Err(format!("template {name} is missing main.tex").into());
        }
        default_found |= name == "default";
        writeln!(generated, "    Template {{ name: {name:?}, files: &[")?;
        for (relative, absolute) in files {
            writeln!(
                generated,
                "        ({relative:?}, include_bytes!({:?})),",
                absolute.to_str().ok_or("template path must be UTF-8")?
            )?;
        }
        generated.push_str("    ] },\n");
    }
    if !default_found {
        return Err("templates/default/main.tex is required".into());
    }
    generated.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("templates.rs"),
        generated,
    )?;
    Ok(())
}

fn entries(path: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(path)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut seen = std::collections::HashSet::new();
    for entry in &entries {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "template paths must be UTF-8")?;
        if !names::portable_component(&name) || !seen.insert(name.to_lowercase()) {
            return Err(format!(
                "non-portable or case-colliding template path: {}",
                entry.path().display()
            )
            .into());
        }
    }
    Ok(entries)
}

fn collect(root: &Path, relative: &Path, files: &mut Vec<(String, PathBuf)>) -> Result<()> {
    let directory = root.join(relative);
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in entries(&directory)? {
        if matches!(
            entry.file_name().to_str(),
            Some(".git" | "build" | "target")
        ) {
            continue;
        }
        let path = relative.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(root, &path, files)?;
        } else if kind.is_file() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
            if path.extension().is_some_and(|ext| ext == "tex") {
                fs::read_to_string(entry.path())
                    .map_err(|e| format!("{} must be UTF-8: {e}", entry.path().display()))?;
            }
            let portable = path
                .components()
                .map(|part| {
                    part.as_os_str()
                        .to_str()
                        .ok_or("template path must be UTF-8")
                })
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join("/");
            files.push((portable, entry.path()));
        } else {
            return Err(format!(
                "unsupported template file (no symbolic links): {}",
                entry.path().display()
            )
            .into());
        }
    }
    Ok(())
}
