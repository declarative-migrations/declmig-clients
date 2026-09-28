#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const TS_PACKAGE: &str = "@declarative-migrations/declmig-validation";
const RUST_PACKAGE: &str = "declmig-validation";
const RUST_CRATE: &str = "declmig_validation";
const GO_PACKAGE: &str = "github.com/declarative-migrations/declmig-lib-core/validation/golang";
const GLEAM_PACKAGE: &str = "declmig_validation";
const FORBIDDEN: [&str; 3] = ["declmig-server-core", "server-core", "migration-executor"];

#[derive(Debug, Default, Deserialize)]
struct NodeManifest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
struct PackageSection {
    #[serde(default)]
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct CargoManifest {
    #[serde(default)]
    package: PackageSection,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Default, Deserialize)]
struct GleamManifest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Check {
    id: &'static str,
    state: &'static str,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema: &'static str,
    repository: &'static str,
    root: String,
    status: &'static str,
    zero_unexplained_findings: bool,
    checks: Vec<Check>,
    errors: Vec<String>,
}

fn canonical_root(root: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(root)
        .map_err(|error| format!("failed to canonicalize {}: {error}", root.display()))
}

fn safe_read(root: &Path, relative: &str) -> Result<String, String> {
    let root = canonical_root(root)?;
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("failed to inspect {relative}: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("{relative} must not be a symlink"));
    }
    if !metadata.is_file() {
        return Err(format!("{relative} must be a regular file"));
    }
    let canonical = fs::canonicalize(&path)
        .map_err(|error| format!("failed to canonicalize {relative}: {error}"))?;
    if !canonical.starts_with(&root) {
        return Err(format!("{relative} resolves outside the repository root"));
    }
    fs::read_to_string(&canonical)
        .map_err(|error| format!("failed to read {relative} as UTF-8: {error}"))
}

fn collect_sources(
    root: &Path,
    relative: &str,
    extensions: &[&str],
) -> Result<Vec<String>, String> {
    let root = canonical_root(root)?;
    let start = root.join(relative);
    let metadata = fs::symlink_metadata(&start)
        .map_err(|error| format!("failed to inspect {relative}: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{relative} must be a real directory"));
    }

    fn visit(
        root: &Path,
        current: &Path,
        extensions: &[&str],
        output: &mut Vec<String>,
    ) -> Result<(), String> {
        let mut entries = fs::read_dir(current)
            .map_err(|error| format!("failed to read {}: {error}", current.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to enumerate {}: {error}", current.display()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "source path {} must not be a symlink",
                    path.display()
                ));
            }
            if metadata.is_dir() {
                visit(root, &path, extensions, output)?;
                continue;
            }
            if !metadata.is_file() {
                continue;
            }
            let extension = path.extension().and_then(OsStr::to_str).unwrap_or_default();
            if !extensions.contains(&extension) {
                continue;
            }
            let canonical = fs::canonicalize(&path)
                .map_err(|error| format!("failed to canonicalize {}: {error}", path.display()))?;
            if !canonical.starts_with(root) {
                return Err(format!(
                    "source path {} escaped the repository",
                    path.display()
                ));
            }
            output.push(
                fs::read_to_string(&canonical).map_err(|error| {
                    format!("failed to read {} as UTF-8: {error}", path.display())
                })?,
            );
        }
        Ok(())
    }

    let mut output = Vec::new();
    visit(&root, &start, extensions, &mut output)?;
    if output.is_empty() {
        return Err(format!("{relative} contains no reviewed source files"));
    }
    Ok(output)
}

fn strip_comments(source: &str) -> String {
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum State {
        Normal,
        Line,
        Block,
        Quoted(char),
    }

    let chars: Vec<char> = source.chars().collect();
    let mut output = String::with_capacity(source.len());
    let mut state = State::Normal;
    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];
        let next = chars.get(index + 1).copied();
        match state {
            State::Normal if current == '/' && next == Some('/') => {
                output.push(' ');
                output.push(' ');
                state = State::Line;
                index += 2;
            }
            State::Normal if current == '/' && next == Some('*') => {
                output.push(' ');
                output.push(' ');
                state = State::Block;
                index += 2;
            }
            State::Normal if matches!(current, '\'' | '"' | '`') => {
                output.push(current);
                state = State::Quoted(current);
                index += 1;
            }
            State::Normal => {
                output.push(current);
                index += 1;
            }
            State::Line if current == '\n' => {
                output.push('\n');
                state = State::Normal;
                index += 1;
            }
            State::Line => {
                output.push(' ');
                index += 1;
            }
            State::Block if current == '*' && next == Some('/') => {
                output.push(' ');
                output.push(' ');
                state = State::Normal;
                index += 2;
            }
            State::Block => {
                output.push(if current == '\n' { '\n' } else { ' ' });
                index += 1;
            }
            State::Quoted(_) if current == '\\' => {
                output.push(current);
                if let Some(escaped) = next {
                    output.push(escaped);
                    index += 2;
                } else {
                    index += 1;
                }
            }
            State::Quoted(quote) if current == quote => {
                output.push(current);
                state = State::Normal;
                index += 1;
            }
            State::Quoted(_) => {
                output.push(current);
                index += 1;
            }
        }
    }
    output
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn reject_forbidden(label: &str, values: impl IntoIterator<Item = String>) -> Result<(), String> {
    for value in values {
        for forbidden in FORBIDDEN {
            if value.contains(forbidden) {
                return Err(format!(
                    "{label} exposes forbidden server package {forbidden}"
                ));
            }
        }
    }
    Ok(())
}

fn dependency_path(value: &toml::Value) -> Option<&str> {
    value.as_table()?.get("path")?.as_str()
}

fn validate_typescript(root: &Path) -> Result<String, String> {
    let source = safe_read(root, "validation-consumer/typescript/package.json")?;
    let manifest: NodeManifest = serde_json::from_str(&source)
        .map_err(|error| format!("invalid TypeScript package.json: {error}"))?;
    if manifest.name != "@declarative-migrations/declmig-validation-consumer" {
        return Err(format!(
            "unexpected TypeScript package name {}",
            manifest.name
        ));
    }
    let dependency = manifest
        .dependencies
        .get(TS_PACKAGE)
        .or_else(|| manifest.dev_dependencies.get(TS_PACKAGE));
    if dependency.map(String::as_str) != Some("file:../../.deps/lib-core/validation/typescript") {
        return Err(
            "TypeScript validation dependency must use the reviewed local SDK path".to_owned(),
        );
    }
    reject_forbidden(
        "TypeScript manifest",
        manifest
            .dependencies
            .keys()
            .chain(manifest.dev_dependencies.keys())
            .cloned(),
    )?;

    let sources = collect_sources(root, "validation-consumer/typescript/src", &["ts", "tsx"])?;
    let mut imported = false;
    for source in sources {
        let stripped = strip_comments(&source);
        reject_forbidden("TypeScript source", [stripped.clone()])?;
        let normalized = compact(&stripped);
        imported |= [
            format!("from\"{TS_PACKAGE}\""),
            format!("from'{TS_PACKAGE}'"),
            format!("import\"{TS_PACKAGE}\""),
            format!("import'{TS_PACKAGE}'"),
            format!("require(\"{TS_PACKAGE}\")"),
            format!("require('{TS_PACKAGE}')"),
        ]
        .iter()
        .any(|needle| normalized.contains(needle));
    }
    if !imported {
        return Err(
            "TypeScript executable source does not import the public validation SDK".to_owned(),
        );
    }
    Ok("manifest path and executable import are exact".to_owned())
}

fn validate_rust(root: &Path) -> Result<String, String> {
    let source = safe_read(root, "validation-consumer/rust/Cargo.toml")?;
    let manifest: CargoManifest =
        toml::from_str(&source).map_err(|error| format!("invalid Rust Cargo.toml: {error}"))?;
    if manifest.package.name != "declmig-validation-consumer" {
        return Err(format!(
            "unexpected Rust package name {}",
            manifest.package.name
        ));
    }
    let dependency = manifest
        .dependencies
        .get(RUST_PACKAGE)
        .ok_or_else(|| "Rust validation dependency is missing".to_owned())?;
    if dependency_path(dependency) != Some("../../.deps/lib-core/validation/rust") {
        return Err("Rust validation dependency must use the reviewed local SDK path".to_owned());
    }
    reject_forbidden("Rust manifest", manifest.dependencies.keys().cloned())?;

    let sources = collect_sources(root, "validation-consumer/rust/src", &["rs"])?;
    let mut imported = false;
    for source in sources {
        let stripped = strip_comments(&source);
        reject_forbidden("Rust source", [stripped.clone()])?;
        let normalized = compact(&stripped);
        imported |= normalized.contains(&format!("use{RUST_CRATE}"))
            || normalized.contains(&format!("externcrate{RUST_CRATE}"));
    }
    if !imported {
        return Err(
            "Rust executable source does not import the public validation crate".to_owned(),
        );
    }
    Ok("manifest path and executable import are exact".to_owned())
}

fn go_imports(source: &str, package: &str) -> bool {
    let source = strip_comments(source);
    let quoted = format!("\"{package}\"");
    let mut in_block = false;
    for line in source.lines() {
        let line = line.trim();
        if in_block {
            if line.starts_with(')') {
                in_block = false;
                continue;
            }
            if line.contains(&quoted) {
                return true;
            }
            continue;
        }
        if line == "import (" || line.starts_with("import(") {
            in_block = true;
            continue;
        }
        if line.starts_with("import ") && line.contains(&quoted) {
            return true;
        }
    }
    false
}

fn validate_go(root: &Path) -> Result<String, String> {
    let source = safe_read(root, "validation-consumer/golang/go.mod")?;
    reject_forbidden("Go manifest", [source.clone()])?;
    let normalized = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join(" ");
    if !normalized.contains(
        "module github.com/declarative-migrations/declmig-clients/validation-consumer/golang",
    ) {
        return Err("unexpected Go consumer module identity".to_owned());
    }
    if !normalized.contains(&format!("require {GO_PACKAGE} v0.0.0")) {
        return Err(
            "Go validation module must be required at the reviewed local placeholder version"
                .to_owned(),
        );
    }
    if !normalized.contains(&format!(
        "replace {GO_PACKAGE} => ../../.deps/lib-core/validation/golang"
    )) {
        return Err("Go validation module must resolve to the reviewed local SDK path".to_owned());
    }

    let sources = collect_sources(root, "validation-consumer/golang", &["go"])?;
    let mut imported = false;
    for source in sources {
        let stripped = strip_comments(&source);
        reject_forbidden("Go source", [stripped])?;
        imported |= go_imports(&source, GO_PACKAGE);
    }
    if !imported {
        return Err("Go executable source does not import the public validation module".to_owned());
    }
    Ok("module replacement and executable import are exact".to_owned())
}

fn validate_gleam(root: &Path) -> Result<String, String> {
    let source = safe_read(root, "validation-consumer/gleam/gleam.toml")?;
    let manifest: GleamManifest =
        toml::from_str(&source).map_err(|error| format!("invalid Gleam manifest: {error}"))?;
    if manifest.name != "declmig_validation_consumer" {
        return Err(format!("unexpected Gleam package name {}", manifest.name));
    }
    let dependency = manifest
        .dependencies
        .get(GLEAM_PACKAGE)
        .ok_or_else(|| "Gleam validation dependency is missing".to_owned())?;
    if dependency_path(dependency) != Some("../../.deps/lib-core/validation/gleam") {
        return Err("Gleam validation dependency must use the reviewed local SDK path".to_owned());
    }
    reject_forbidden("Gleam manifest", manifest.dependencies.keys().cloned())?;

    let sources = collect_sources(root, "validation-consumer/gleam/src", &["gleam"])?;
    let mut imported = false;
    for source in sources {
        let stripped = strip_comments(&source);
        reject_forbidden("Gleam source", [stripped.clone()])?;
        imported |= stripped.lines().any(|line| {
            let line = line.trim_start();
            line == "import declmig_validation"
                || line.starts_with("import declmig_validation.")
                || line.starts_with("import declmig_validation ")
        });
    }
    if !imported {
        return Err(
            "Gleam executable source does not import the public validation package".to_owned(),
        );
    }
    Ok("manifest path and executable import are exact".to_owned())
}

fn audit(root: &Path) -> Report {
    let canonical = canonical_root(root)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| root.display().to_string());
    let validations: [(&str, Result<String, String>); 4] = [
        (
            "typescript-public-validation-boundary",
            validate_typescript(root),
        ),
        ("rust-public-validation-boundary", validate_rust(root)),
        ("go-public-validation-boundary", validate_go(root)),
        ("gleam-public-validation-boundary", validate_gleam(root)),
    ];
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    for (id, result) in validations {
        match result {
            Ok(detail) => checks.push(Check {
                id,
                state: "passed",
                detail,
            }),
            Err(error) => {
                checks.push(Check {
                    id,
                    state: "failed",
                    detail: error.clone(),
                });
                errors.push(format!("{id}: {error}"));
            }
        }
    }
    let passed = errors.is_empty();
    Report {
        schema: "ores.validation-import-check/v1",
        repository: "declarative-migrations/declmig-clients",
        root: canonical,
        status: if passed { "passed" } else { "failed" },
        zero_unexplained_findings: passed,
        checks,
        errors,
    }
}

fn write_report(path: &Path, report: &Report) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let temporary = path.with_extension("json.tmp");
    let encoded = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("failed to encode report: {error}"))?;
    fs::write(&temporary, encoded)
        .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
    fs::rename(&temporary, path)
        .map_err(|error| format!("failed to publish {}: {error}", path.display()))
}

fn usage() -> &'static str {
    "usage: declmig-validation-import-check [--root PATH] [--report PATH]"
}

fn parse_args() -> Result<(PathBuf, Option<PathBuf>), String> {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut report = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--root" => {
                root = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--root requires a path".to_owned())?,
                );
            }
            "--report" => {
                report = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--report requires a path".to_owned())?,
                ));
            }
            "-h" | "--help" => return Err(usage().to_owned()),
            _ => return Err(format!("unknown argument {argument:?}; {}", usage())),
        }
    }
    Ok((root, report))
}

fn main() -> ExitCode {
    let (root, report_path) = match parse_args() {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let report = audit(&root);
    if let Some(path) = report_path {
        let path = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        if let Err(error) = write_report(&path, &report) {
            eprintln!("validation import audit: {error}");
            return ExitCode::from(2);
        }
    }
    if report.zero_unexplained_findings {
        println!("validated 4 public validation consumer boundaries");
        ExitCode::SUCCESS
    } else {
        eprintln!("validation import audit failed:");
        for error in &report.errors {
            eprintln!(" - {error}");
        }
        ExitCode::from(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, content).expect("write fixture");
    }

    fn valid_fixture() -> tempfile::TempDir {
        let fixture = tempfile::tempdir().expect("fixture");
        let root = fixture.path();
        write(
            root,
            "validation-consumer/typescript/package.json",
            r#"{"name":"@declarative-migrations/declmig-validation-consumer","dependencies":{"@declarative-migrations/declmig-validation":"file:../../.deps/lib-core/validation/typescript"}}"#,
        );
        write(
            root,
            "validation-consumer/typescript/src/index.ts",
            r#"import { RequestMetaSchema } from "@declarative-migrations/declmig-validation"; export { RequestMetaSchema };"#,
        );
        write(
            root,
            "validation-consumer/rust/Cargo.toml",
            "[package]\nname = \"declmig-validation-consumer\"\nversion = \"0.1.0\"\n[dependencies]\ndeclmig-validation = { path = \"../../.deps/lib-core/validation/rust\" }\n",
        );
        write(
            root,
            "validation-consumer/rust/src/lib.rs",
            "use declmig_validation::RequestMeta; pub fn accept(_: RequestMeta) {}\n",
        );
        write(
            root,
            "validation-consumer/golang/go.mod",
            "module github.com/declarative-migrations/declmig-clients/validation-consumer/golang\n\ngo 1.25.0\nrequire github.com/declarative-migrations/declmig-lib-core/validation/golang v0.0.0\nreplace github.com/declarative-migrations/declmig-lib-core/validation/golang => ../../.deps/lib-core/validation/golang\n",
        );
        write(
            root,
            "validation-consumer/golang/consumer.go",
            "package validationconsumer\nimport public \"github.com/declarative-migrations/declmig-lib-core/validation/golang\"\nvar _ public.RequestMeta\n",
        );
        write(
            root,
            "validation-consumer/gleam/gleam.toml",
            "name = \"declmig_validation_consumer\"\nversion = \"0.1.0\"\n[dependencies]\ndeclmig_validation = { path = \"../../.deps/lib-core/validation/gleam\" }\n",
        );
        write(
            root,
            "validation-consumer/gleam/src/consumer.gleam",
            "import declmig_validation\npub fn validate(value) { declmig_validation.decode_request_meta(value) }\n",
        );
        fixture
    }

    #[test]
    fn valid_fixture_passes() {
        let fixture = valid_fixture();
        assert!(audit(fixture.path()).zero_unexplained_findings);
    }

    #[test]
    fn readme_mentions_do_not_replace_executable_imports() {
        let fixture = valid_fixture();
        write(
            fixture.path(),
            "validation-consumer/typescript/src/index.ts",
            "export const value = 1;\n",
        );
        write(
            fixture.path(),
            "validation-consumer/typescript/README.md",
            "@declarative-migrations/declmig-validation\n",
        );
        let report = audit(fixture.path());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("TypeScript executable source"))
        );
    }

    #[test]
    fn wrong_manifest_path_fails_closed() {
        let fixture = valid_fixture();
        write(
            fixture.path(),
            "validation-consumer/rust/Cargo.toml",
            "[package]\nname = \"declmig-validation-consumer\"\nversion = \"0.1.0\"\n[dependencies]\ndeclmig-validation = { path = \"../../server-core\" }\n",
        );
        let report = audit(fixture.path());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Rust validation dependency"))
        );
    }

    #[test]
    fn forbidden_server_package_in_source_is_rejected() {
        let fixture = valid_fixture();
        write(
            fixture.path(),
            "validation-consumer/gleam/src/consumer.gleam",
            "import declmig_validation\nimport migration-executor\n",
        );
        let report = audit(fixture.path());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("forbidden server package"))
        );
    }

    #[test]
    fn go_manifest_without_exact_replace_is_rejected() {
        let fixture = valid_fixture();
        write(
            fixture.path(),
            "validation-consumer/golang/go.mod",
            "module github.com/declarative-migrations/declmig-clients/validation-consumer/golang\nrequire github.com/declarative-migrations/declmig-lib-core/validation/golang v0.0.0\nreplace github.com/declarative-migrations/declmig-lib-core/validation/golang => ../../wrong\n",
        );
        let report = audit(fixture.path());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Go validation module"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_source_is_rejected() {
        use std::os::unix::fs::symlink;
        let fixture = valid_fixture();
        let outside = fixture.path().join("outside.ts");
        fs::write(&outside, format!("import x from \"{TS_PACKAGE}\";\n")).unwrap();
        let source = fixture
            .path()
            .join("validation-consumer/typescript/src/index.ts");
        fs::remove_file(&source).unwrap();
        symlink(&outside, &source).unwrap();
        let report = audit(fixture.path());
        assert!(report.errors.iter().any(|error| error.contains("symlink")));
    }
}
