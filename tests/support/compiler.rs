// Native executable test double; no shell or external interpreter required.
fn main() {
    let cwd = std::env::current_dir().unwrap();
    let args = std::env::args().skip(1).collect::<Vec<_>>().join("\n") + "\n";
    for filename in ["visited", "invoked-from"] {
        std::fs::write(cwd.join("build").join(filename), cwd.to_string_lossy().as_bytes()).unwrap();
    }
    std::fs::write(cwd.join("build/invoked-with"), args).unwrap();
    if cwd.join("fail").exists() { std::process::exit(1); }
}
