//! Compile `proto/page.proto` and inject Markdown body attributes.

use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use buffa::Message;
use serde_markdown::buffa::{FileDescriptorSet, annotate_markdown_body};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PROTOBUF_INCLUDE");
    println!("cargo:rerun-if-env-changed=SERDE_MARKDOWN_PROTO_INCLUDE");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let proto_dir = manifest_dir.join("proto");
    let page_proto = proto_dir.join("page.proto");
    if !page_proto.is_file() {
        return Err(format!("missing proto file {}", page_proto.display()).into());
    }
    println!("cargo:rerun-if-changed={}", page_proto.display());

    let markdown_include = include_root(markdown_proto_include(), "markdown/options.proto")?;
    let options_proto = markdown_include.join("markdown/options.proto");
    println!("cargo:rerun-if-changed={}", options_proto.display());

    let mut includes = vec![proto_dir.clone(), markdown_include];
    if let Some(protobuf_include) = protobuf_include_dir() {
        includes.push(include_root(
            protobuf_include,
            "google/protobuf/descriptor.proto",
        )?);
    }

    let fds_path = out_dir.join("page.fdset");
    compile_descriptor_set(&page_proto, &includes, &fds_path)?;

    let fds_bytes = fs::read(&fds_path)?;
    let fds = FileDescriptorSet::decode_from_slice(&fds_bytes).map_err(|err| {
        format!(
            "failed to decode FileDescriptorSet {}: {err}",
            fds_path.display()
        )
    })?;
    let proto_name = fds
        .file
        .iter()
        .filter_map(|file| file.name.as_deref())
        .find(|name| Path::new(name).file_name() == Some("page.proto".as_ref()))
        .ok_or("FileDescriptorSet does not contain page.proto")?
        .to_owned();

    let attrs = annotate_markdown_body(&fds);
    let mut config = buffa_build::Config::new()
        .descriptor_set(&fds_path)
        .files(&[proto_name.as_str()])
        .includes(&includes)
        .generate_json(true)
        .include_file("_include.rs");
    for (path, attr) in attrs.message_attributes() {
        config = config.message_attribute(path, attr);
    }
    for (path, attr) in attrs.field_attributes() {
        config = config.field_attribute(path, attr);
    }
    config.compile()?;
    Ok(())
}

fn markdown_proto_include() -> PathBuf {
    env::var_os("SERDE_MARKDOWN_PROTO_INCLUDE").map_or_else(
        || PathBuf::from(serde_markdown::PROTO_INCLUDE),
        PathBuf::from,
    )
}

/// Resolve an include root that contains `needle`.
///
/// Bazel `$(execpath file)/../..` is not a real directory until `..` is
/// walked lexically, and the number of `..` segments depends on how deep
/// the proto file sits under the include root.
fn include_root(path: PathBuf, needle: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let normalized = normalize_path(&path);
    let mut cur = normalized.clone();
    if cur.is_file() {
        let _ = cur.pop();
    }
    loop {
        if cur.join(needle).is_file() {
            return Ok(cur);
        }
        if !cur.pop() {
            return Err(format!(
                "could not find {needle} above {} (from {})",
                normalized.display(),
                path.display()
            )
            .into());
        }
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::CurDir => {}
            rest => out.push(rest),
        }
    }
    out
}

fn protobuf_include_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("PROTOBUF_INCLUDE") {
        return Some(PathBuf::from(dir));
    }
    let protoc = resolve_protoc_path()?;
    let bin_dir = protoc.parent()?;
    let prefix = bin_dir.parent()?;
    let candidates = [prefix.join("include"), bin_dir.join("../include")];
    for candidate in candidates {
        if has_descriptor_proto(&candidate) {
            return Some(candidate);
        }
    }
    if let Ok(entries) = fs::read_dir(prefix.join("protoc")) {
        for entry in entries.flatten() {
            let include = entry.path().join("include");
            if has_descriptor_proto(&include) {
                return Some(include);
            }
        }
    }
    let usr = PathBuf::from("/usr/include");
    has_descriptor_proto(&usr).then_some(usr)
}

fn has_descriptor_proto(include: &Path) -> bool {
    include.join("google/protobuf/descriptor.proto").is_file()
}

fn resolve_protoc_path() -> Option<PathBuf> {
    let protoc = env::var_os("PROTOC").map_or_else(|| PathBuf::from("protoc"), PathBuf::from);
    if protoc.is_file() {
        return Some(protoc);
    }
    let name = protoc.as_os_str();
    env::split_paths(&env::var_os("PATH")?).find_map(|dir| {
        let candidate = dir.join(name);
        candidate.is_file().then_some(candidate)
    })
}

fn compile_descriptor_set(
    page_proto: &Path,
    includes: &[PathBuf],
    fds_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let protoc = env::var("PROTOC").unwrap_or_else(|_| "protoc".to_owned());
    let mut cmd = Command::new(&protoc);
    cmd.arg("--include_imports")
        .arg(format!("--descriptor_set_out={}", fds_path.display()));
    for include in includes {
        cmd.arg(format!("--proto_path={}", include.display()));
    }
    cmd.arg(page_proto);
    let output = cmd.output().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            format!(
                "protoc not found ({protoc}); set PROTOC to a protoc binary or install protoc on PATH"
            )
        } else {
            format!("failed to run protoc ({protoc}): {err}")
        }
    })?;
    if !output.status.success() {
        return Err(format!("protoc failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(())
}
