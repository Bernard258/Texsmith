# Texsmith

Texsmith is a cross-platform Rust command-line tool for creating, managing, and
compiling LaTeX reports.

Templates are embedded directly into the executable, so creating a report does
not require a separate template directory. Each report remains an ordinary,
editable folder containing its LaTeX source and supporting files.

## Features

- Create reports from bundled templates with one command
- Compile one report, several reports, or every report in a directory
- Produce native Linux, Windows, and macOS executables
- Support absolute paths, relative paths, spaces, and Unicode filenames
- Keep generated PDFs and logs inside each report's `build/` directory
- Use Tectonic for reproducible, automatic LaTeX compilation

## Requirements

- Rust 1.85 or newer to build Texsmith
- Tectonic on `PATH` to compile reports
- Python 3.10 or newer only for the optional developer checks

## Quick start

From this source directory:

```sh
cargo texsmith templates
cargo texsmith new my-paper "My Paper" --template ieee-conference
cargo texsmith build my-paper
cargo texsmith list
cargo texsmith build --all
```

Edit `my-paper/main.tex` and its supporting files. The PDF is
`my-paper/build/main.pdf`. The included `example-report/` is ready to build.
`cargo texsmith` is a local alias for `cargo run --quiet --`.

## Shorthand commands

| Full form | Shorthand |
| --- | --- |
| `new` | `n` |
| `build` | `b` |
| `list` | `ls` or `l` |
| `templates` | `t` |
| `help` | `h`, `-h`, or `--help` |
| `--template NAME` | `-t NAME` |
| `--all` | `-a` |
| `--root PATH` (before the command) | `-r PATH` |

```sh
texsmith n my-paper "My Paper" -t ieee-conference
texsmith b my-paper
texsmith b -a
texsmith ls
texsmith t
texsmith -r ~/Documents n essay -t mla
```

Short and long forms can be mixed. Template names stay unchanged.

## Installation

### Install from source

```sh
cargo build --release
```

Distribute **only `target/release/texsmith`** (or **`target/release/texsmith.exe`** on Windows). It includes all template contents;
no template directory, Cargo, Rust sources, or generated Rust files are needed
on the destination machine. Tectonic must still be installed on `PATH` to
compile PDFs; the TeX engine and its resource cache are separate dependencies.
Creating and listing reports require only the manager binary.

Alternatively, install the executable with `cargo install --path .`. From an
existing directory where you want to keep reports:

```sh
texsmith templates
texsmith new essay "My Essay" --template mla
texsmith build essay
```

### Install from a release

On Linux or macOS, install the latest release with `wget`:

```sh
wget -qO- https://raw.githubusercontent.com/Bernard258/Texsmith/main/scripts/install.sh | sh
```

The installer puts `texsmith` in `~/.local/bin`. To install a specific release,
pass its tag:

```sh
wget -qO- https://raw.githubusercontent.com/Bernard258/Texsmith/main/scripts/install.sh | sh -s -- v0.1.0
```

Set `TEXSMITH_INSTALL_DIR` to use a different directory. To remove the
installed binary, run:

```sh
wget -qO- https://raw.githubusercontent.com/Bernard258/Texsmith/main/scripts/uninstall.sh | sh
```

The uninstall script also respects `TEXSMITH_INSTALL_DIR`.

Commands use the **current working directory**, with each report created as
`./NAME/main.tex` and supporting files inside `./NAME/`. No initialization or
workspace marker is needed. Ancestor directories and legacy `.reports-root`
files do not change where reports are created or discovered.

Use `texsmith --root /path/to/directory new my-paper` to explicitly choose a
different existing parent directory. The same override works for `list` and
`build`. Paths are relative to where you launch Texsmith, not where its binary
is stored. `texsmith --help` lists the commands.

## Windows, Linux, and macOS

The same source and commands are used on all three operating systems. Build a
native executable for the destination OS and CPU architecture; a Linux binary
cannot be run directly on Windows or macOS. All templates are embedded in each
native binary. Install the matching Tectonic executable and put it on `PATH`.
The manager launches it directly; Bash, PowerShell, and Python are not runtime
requirements.

From PowerShell, a local binary is invoked with an explicit relative path:

```powershell
.\texsmith.exe n paper -t apa
.\texsmith.exe b paper
$env:TECTONIC = 'C:\Program Files\Tectonic\tectonic.exe'
.\texsmith.exe b 'C:\Users\Me\My Papers\paper'
```

On Linux/macOS, use `./texsmith` for a local binary, or `texsmith` when installed
on `PATH`. Quote paths containing spaces. Native path separators, Unicode
paths, and absolute/relative report paths are supported. `TECTONIC` accepts an
executable name on `PATH` or an executable path; explicit relative paths are
resolved from the caller's directory before changing into a report folder.
Windows `.exe` suffixes are handled by the native process launcher.

New report names reject Windows device names such as `CON`, `NUL`, and `COM1`
on every OS. Bundled template paths are also checked for Windows-invalid names
and case collisions so the same templates can be extracted on case-insensitive
filesystems. Existing directories with other names can still be built by path.

The workflow at `.github/workflows/texsmith.yml` builds and tests natively on
Windows, Linux, and macOS. It includes a native compiler test
double to verify process arguments, working directories, and failure handling
without requiring Tectonic. Actual PDF checks use `scripts/check_templates.py`
with Tectonic installed. Python 3.10+ is needed only for developer checks; use
`python` on Windows or `python3` on Linux/macOS as appropriate.

## Templates

| Template | Layout |
| --- | --- |
| `default` | General-purpose report |
| `brief` | Short summary, findings, and next steps |
| `research-article` | Single-column research paper, numbered citations |
| `preprint` | Single-column, 12-point paper, author–year citations |
| `ieee-conference` | IEEE conference, two columns, IEEE references |
| `ieee-journal` | IEEE journal, two columns, affiliation footnotes |
| `acm-conference` | ACM `sigconf` layout, local draft in `nonacm` mode |
| `elsevier` | Elsevier `elsarticle` preprint, numbered citations |
| `mla` | MLA student essay, surname/page header, Works Cited |
| `apa` | APA 7 student paper, title page, References |

```sh
texsmith new essay "My Essay" --template mla
texsmith new study "My Study" --template apa
```

Omitting `--template` selects `default`. The option can appear before or after
the report name and optional title. Titles are escaped as literal LaTeX text.
Each report gets independent editable copies of its selected template's files.
Existing report folders are never overwritten.

Most paper templates include `sections/body.tex`, `references.bib`, a sample
citation, and an illustrative table. MLA and APA instead include
`references.tex`: their citations and reference entries are **maintained
manually**, with current-style examples and hanging indents, requiring no Biber.
Replace demonstration sources and follow the template's copied README.

The IEEE templates use [IEEEtran](https://www.michaelshell.org/tex/ieeetran/);
see the [IEEE authoring guidance](https://conferences.ieeeauthorcenter.ieee.org/write-your-paper/authoring-tools-and-templates/).
ACM uses [acmart](https://ctan.org/pkg/acmart), and Elsevier uses
[elsarticle](https://ctan.org/pkg/elsarticle). MLA and APA follow student-paper
layouts described by the [MLA Style Center](https://style.mla.org/formatting-papers/)
and [APA student-paper guide](https://apastyle.apa.org/instructional-aids/student-paper-setup-guide.pdf).
Follow your venue or instructor's additional requirements.

## Adding or changing bundled templates

The **`templates/` source folder is the source of truth**. No template contents
or catalog entries are hard-coded in Rust. During `cargo build`, `build.rs`
scans its subfolders, validates them, and generates an internal manifest using
`include_bytes!`. Rust embeds the files directly in the executable. The generated
manifest is only a build intermediate under Cargo's target directory.

```text
templates/
  default/
    main.tex
  my-format/
    main.tex
    sections/methods.tex
    figures/logo.png
    references.bib
    README.md
```

To add a format, create `templates/my-format/main.tex` and supporting files, then
run `cargo build --release`. `texsmith templates` will include `my-format` in the
rebuilt binary automatically. Cargo tracks additions, edits, and removals.
Customize `templates/default/main.tex` to change the default for future reports.
The former `templates/main.tex` has been moved there to use the same convention.

All template files are embedded, including binary images. `@@TITLE@@` is replaced
in every `.tex` file when a report is created; `.tex` files must be UTF-8. Other
files are written unchanged. Nested directories are preserved when they contain
files. Empty directories are omitted, and `figures/` is always created.

Template/report names start with an ASCII letter or number and may contain
letters, numbers, hyphens, and underscores. `src`, `target`, and `templates` (case-insensitive) and Windows device names are
reserved report names. Build-time scanning skips entries named `.git`, `build`,
and `target`, rejects symbolic links and special files, and requires `main.tex`
in every template folder. Top-level files such as catalog documentation are
ignored. A `default` template is required.

Changing source templates requires rebuilding the executable. Runtime
`templates/` folders do not override the embedded templates. Add-ins are not
implemented yet. Existing reports remain unchanged by recompilation.

## Compilation

```sh
texsmith b                                  # Build ./main.tex
texsmith b .                                # Same, explicit current directory
texsmith b paper-one ../paper-two            # Build multiple report directories
texsmith b "/absolute/path/My Paper"         # Absolute paths and spaces work
texsmith b -a                               # Build immediate child report folders
texsmith -r /path/to/paper b                 # Build that directory's main.tex
```

Every selected directory must contain `main.tex`; its output goes into its own
`build/` folder. Explicit paths can point anywhere and are resolved relative to
the current directory, or to `--root` when supplied. Explicit directory symlinks
are resolved. `--all` continues to discover immediate child folders using the
report naming rules and skips symlinks; it does not recursively scan directories.
Do not combine `--all` with explicit paths. For a path starting with a hyphen,
use `texsmith b -- -paper` (or `texsmith b ./-paper`). All selected reports are
attempted even if an earlier directory is missing or fails to compile.


Install Tectonic and make it available on `PATH`. It downloads TeX packages and
fonts on demand; the first compilation may need internet access and take longer.
Tectonic handles LaTeX reruns and BibTeX for templates using `.bib` files.

The manager invokes Tectonic from inside the report folder, so relative images
and `\input` files work. PDFs, logs, and SyncTeX output go into `build/`.
Some classes, including ACM, read the log during compilation, causing Tectonic
to treat it as an intermediate. To retain that log, run this inside the report:

```sh
tectonic --keep-intermediates --keep-logs --outdir build main.tex
```

A failed build returns a nonzero exit status and names the failed directories. An old PDF may remain after failure; check the status.
To reset generated output, delete the report's `build/` folder. The repository
ignores report build folders and Cargo's `target/` directory.

Override Tectonic with an executable path (no additional arguments):

```sh
TECTONIC=/absolute/path/to/tectonic texsmith build my-paper
```

## Development checks

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
python3 scripts/check_embedding.py
python3 scripts/check_templates.py
```

The Rust tests exercise a relocated binary with no runtime template folder.
The embedding check rebuilds an isolated copy of the project after adding,
editing, and removing a template, and verifies binary assets and standalone use.
The template check compiles all embedded templates in a temporary workspace,
checks PDFs, references, fonts, and title substitution, and uses `pdftotext` when
available to verify the bibliography. It requires Python 3 and Tectonic.
