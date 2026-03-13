// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
enum LinkKind {
    Static,
    Dynamic,
}

#[derive(Clone, Debug)]
struct Library {
    name: String,
    kind: LinkKind,
}

#[derive(Clone, Debug, Default)]
struct BuildArtifacts {
    include_dirs: Vec<PathBuf>,
    link_search_dirs: Vec<PathBuf>,
    libraries: Vec<Library>,
}

fn main() {
    println!("cargo:rerun-if-env-changed=JPEGLI_SYS_USE_SYSTEM");
    println!("cargo:rerun-if-env-changed=JPEGLI_SYS_ROOT");
    println!("cargo:rerun-if-env-changed=JPEGLI_SYS_STATIC");
    println!("cargo:rerun-if-env-changed=JPEGLI_SYS_CMAKE_TOOLCHAIN_FILE");
    println!("cargo:rerun-if-env-changed=JPEGLI_SYS_PKG_CONFIG");
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let vendor_root = manifest_dir.join("vendor").join("jpegli");

    emit_rerun_tree(&manifest_dir.join("src").join("shim"));
    emit_rerun_tree(&vendor_root.join("lib").join("jpegli"));
    emit_rerun_tree(&vendor_root.join("lib").join("base"));
    emit_rerun_tree(&vendor_root.join("third_party").join("libjpeg-turbo"));
    emit_rerun_tree(&vendor_root.join("third_party").join("highway"));
    emit_rerun_tree(&vendor_root.join("third_party").join("skcms"));
    println!(
        "cargo:rerun-if-changed={}",
        vendor_root.join("CMakeLists.txt").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor_root.join("lib").join("CMakeLists.txt").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor_root.join("lib").join("jpegli.cmake").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        vendor_root.join("lib").join("jxl_lists.cmake").display()
    );

    let use_system = env_flag("JPEGLI_SYS_USE_SYSTEM")
        || (feature_enabled("system") && !feature_enabled("vendored"));
    let prefer_static = env::var("JPEGLI_SYS_STATIC")
        .ok()
        .map(|v| v == "1")
        .unwrap_or_else(|| feature_enabled("static") || !feature_enabled("shared"));

    let artifacts = if use_system {
        link_system(&vendor_root, prefer_static)
    } else {
        build_vendored(&vendor_root, prefer_static)
    };

    build_shim(&manifest_dir, &artifacts.include_dirs);
    emit_link_flags(&artifacts);
    emit_cpp_runtime_flags();
    generate_bindings(&manifest_dir);
}

fn build_vendored(vendor_root: &Path, prefer_static: bool) -> BuildArtifacts {
    assert_required_vendor_files(vendor_root);

    let _ = prefer_static;

    if feature_enabled("shared") && !feature_enabled("system") {
        println!(
            "cargo:warning=the `shared` feature only affects system mode; vendored mode still links jpegli-static"
        );
    }

    let mut cfg = cmake::Config::new(vendor_root);
    cfg.build_target("jpegli-static");
    cfg.define("BUILD_TESTING", "OFF");
    cfg.define("BUILD_SHARED_LIBS", "OFF");
    cfg.define("JPEGXL_ENABLE_BENCHMARK", "OFF");
    cfg.define("JPEGXL_ENABLE_DEVTOOLS", "OFF");
    cfg.define("JPEGXL_ENABLE_DOXYGEN", "OFF");
    cfg.define("JPEGXL_ENABLE_FUZZERS", "OFF");
    cfg.define("JPEGXL_ENABLE_JNI", "OFF");
    cfg.define("JPEGXL_ENABLE_JPEGLI_LIBJPEG", "OFF");
    cfg.define("JPEGXL_ENABLE_MANPAGES", "OFF");
    cfg.define("JPEGXL_ENABLE_OPENEXR", "OFF");
    cfg.define("JPEGXL_ENABLE_SKCMS", "ON");
    cfg.define("JPEGXL_ENABLE_SJPEG", "OFF");
    cfg.define("JPEGXL_ENABLE_TOOLS", "OFF");
    cfg.define("JPEGXL_ENABLE_TCMALLOC", "OFF");

    if let Ok(toolchain_file) = env::var("JPEGLI_SYS_CMAKE_TOOLCHAIN_FILE") {
        cfg.define("CMAKE_TOOLCHAIN_FILE", toolchain_file);
    }

    match env::var("PROFILE").as_deref() {
        Ok("debug") => {
            cfg.profile("Debug");
        }
        Ok("release") | Ok("bench") => {
            cfg.profile("Release");
        }
        _ => {
            cfg.profile("RelWithDebInfo");
        }
    }

    let install_dir = cfg.build();
    let build_root = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR")).join("build");

    let generated_header = find_file(&build_root, "jconfig.h")
        .or_else(|| find_file(&install_dir, "jconfig.h"))
        .unwrap_or_else(|| {
            panic!(
                "failed to locate generated jconfig.h under {}",
                build_root.display()
            )
        });
    let generated_include_dir = generated_header
        .parent()
        .expect("generated header parent missing")
        .to_path_buf();

    let jpegli_lib = find_library_file(&build_root, "jpegli-static")
        .or_else(|| find_library_file(&install_dir, "jpegli-static"))
        .unwrap_or_else(|| {
            panic!(
                "failed to locate jpegli-static output under {}",
                build_root.display()
            )
        });
    let hwy_lib = find_library_file(&build_root, "hwy")
        .or_else(|| find_library_file(&install_dir, "hwy"))
        .unwrap_or_else(|| panic!("failed to locate hwy output under {}", build_root.display()));

    let mut artifacts = BuildArtifacts::default();
    artifacts.include_dirs.push(vendor_root.to_path_buf());
    artifacts
        .include_dirs
        .push(vendor_root.join("third_party").join("libjpeg-turbo"));
    artifacts.include_dirs.push(generated_include_dir);
    artifacts.link_search_dirs.push(
        jpegli_lib
            .parent()
            .expect("jpegli lib parent missing")
            .to_path_buf(),
    );
    artifacts.link_search_dirs.push(
        hwy_lib
            .parent()
            .expect("hwy lib parent missing")
            .to_path_buf(),
    );
    artifacts.libraries.push(Library {
        name: "jpegli-static".to_owned(),
        kind: LinkKind::Static,
    });
    artifacts.libraries.push(Library {
        name: "hwy".to_owned(),
        kind: LinkKind::Static,
    });
    artifacts
}

fn link_system(vendor_root: &Path, prefer_static: bool) -> BuildArtifacts {
    let mut artifacts = BuildArtifacts::default();
    artifacts.include_dirs.push(vendor_root.to_path_buf());
    artifacts
        .include_dirs
        .push(vendor_root.join("third_party").join("libjpeg-turbo"));

    if let Some(pkg) = env::var_os("JPEGLI_SYS_PKG_CONFIG")
        .map(PathBuf::from)
        .or_else(which_pkg_config)
        && let Some(pkg_info) = query_pkg_config(&pkg, prefer_static)
    {
        artifacts.include_dirs.extend(pkg_info.include_dirs);
        artifacts.link_search_dirs.extend(pkg_info.link_search_dirs);
        artifacts.libraries.extend(pkg_info.libraries);
    }

    if let Ok(root) = env::var("JPEGLI_SYS_ROOT") {
        let root = PathBuf::from(root);
        if let Some(include_dir) =
            find_file(&root, "jconfig.h").and_then(|p| p.parent().map(Path::to_path_buf))
        {
            artifacts.include_dirs.push(include_dir);
        }

        for candidate in ["lib", "lib64", "build", "out"] {
            let dir = root.join(candidate);
            if dir.exists() {
                artifacts.link_search_dirs.push(dir);
            }
        }

        if let Some(found) = find_library_file(
            &root,
            if prefer_static {
                "jpegli-static"
            } else {
                "jpegli"
            },
        ) {
            artifacts.link_search_dirs.push(
                found
                    .parent()
                    .expect("system library parent missing")
                    .to_path_buf(),
            );
        }

        if let Some(found) = find_library_file(&root, "hwy") {
            artifacts.link_search_dirs.push(
                found
                    .parent()
                    .expect("hwy library parent missing")
                    .to_path_buf(),
            );
            if !artifacts.libraries.iter().any(|lib| lib.name == "hwy") {
                artifacts.libraries.push(Library {
                    name: "hwy".to_owned(),
                    kind: LinkKind::Static,
                });
            }
        }
    }

    if !artifacts
        .include_dirs
        .iter()
        .any(|dir| dir.join("jconfig.h").exists())
    {
        panic!(
            "system mode requires headers that provide jconfig.h; set JPEGLI_SYS_ROOT to a compatible JPEGli install/build tree"
        );
    }

    if !artifacts
        .libraries
        .iter()
        .any(|lib| lib.name == "jpegli" || lib.name == "jpegli-static")
    {
        artifacts.libraries.push(Library {
            name: if prefer_static {
                "jpegli-static".to_owned()
            } else {
                "jpegli".to_owned()
            },
            kind: if prefer_static {
                LinkKind::Static
            } else {
                LinkKind::Dynamic
            },
        });
    }

    artifacts
}

fn build_shim(manifest_dir: &Path, include_dirs: &[PathBuf]) {
    let mut build = cc::Build::new();
    build.cpp(true);
    build.warnings(false);
    build.file(
        manifest_dir
            .join("src")
            .join("shim")
            .join("jpegli_rs_shim.cc"),
    );
    for dir in include_dirs {
        build.include(dir);
    }

    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.flag_if_supported("/std:c++17");
    } else {
        build.flag_if_supported("-std=c++17");
    }

    build.compile("jpegli_rs_shim");
}

fn emit_link_flags(artifacts: &BuildArtifacts) {
    let mut seen_dirs = BTreeSet::new();
    for dir in &artifacts.link_search_dirs {
        if seen_dirs.insert(dir.clone()) {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }

    let mut seen_libs = BTreeSet::new();
    for lib in &artifacts.libraries {
        let kind = match lib.kind {
            LinkKind::Static => "static",
            LinkKind::Dynamic => "dylib",
        };
        if seen_libs.insert((kind.to_owned(), lib.name.clone())) {
            println!("cargo:rustc-link-lib={}={}", kind, lib.name);
        }
    }
}

fn emit_cpp_runtime_flags() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=dylib=c++");
        }
        "windows" => {}
        _ => {
            if target_env == "gnu" {
                println!("cargo:rustc-link-lib=dylib=stdc++");
            }
            println!("cargo:rustc-link-lib=dylib=m");
            println!("cargo:rustc-link-lib=dylib=pthread");
        }
    }
}

fn assert_required_vendor_files(vendor_root: &Path) {
    for relative in [
        "CMakeLists.txt",
        "lib/jpegli/encode.h",
        "third_party/highway/CMakeLists.txt",
        "third_party/libjpeg-turbo/jpeglib.h",
        "third_party/skcms/skcms.h",
    ] {
        let path = vendor_root.join(relative);
        if !path.exists() {
            panic!(
                "vendored JPEGli is incomplete; expected {} to exist",
                path.display()
            );
        }
    }
}

fn emit_rerun_tree(path: &Path) {
    if !path.exists() {
        return;
    }
    if path.is_file() {
        println!("cargo:rerun-if-changed={}", path.display());
        return;
    }

    let entries = fs::read_dir(path).unwrap_or_else(|err| {
        panic!("failed to read {}: {}", path.display(), err);
    });
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|err| panic!("failed to walk {}: {}", path.display(), err));
        emit_rerun_tree(&entry.path());
    }
}

fn find_file(root: &Path, name: &str) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }
    if root.is_file() {
        return (root.file_name() == Some(OsStr::new(name))).then(|| root.to_path_buf());
    }

    for entry in fs::read_dir(root).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        } else if path.file_name() == Some(OsStr::new(name)) {
            return Some(path);
        }
    }
    None
}

fn find_library_file(root: &Path, stem: &str) -> Option<PathBuf> {
    let candidates = [
        format!("lib{stem}.a"),
        format!("lib{stem}.so"),
        format!("lib{stem}.dylib"),
        format!("{stem}.lib"),
        format!("{stem}.dll"),
    ];

    candidates
        .iter()
        .find_map(|candidate| find_file(root, candidate))
}

fn feature_enabled(name: &str) -> bool {
    env::var_os(format!(
        "CARGO_FEATURE_{}",
        name.replace('-', "_").to_ascii_uppercase()
    ))
    .is_some()
}

fn env_flag(name: &str) -> bool {
    env::var(name).map(|value| value == "1").unwrap_or(false)
}

fn which_pkg_config() -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join("pkg-config"))
        .find(|candidate| candidate.exists())
}

fn query_pkg_config(pkg_config: &Path, prefer_static: bool) -> Option<BuildArtifacts> {
    for package in ["jpegli-static", "jpegli"] {
        let mut command = Command::new(pkg_config);
        if prefer_static {
            command.arg("--static");
        }
        let output = command
            .args(["--cflags", "--libs", package])
            .output()
            .ok()?;
        if !output.status.success() {
            continue;
        }

        let mut artifacts = BuildArtifacts::default();
        let stdout = String::from_utf8(output.stdout).ok()?;
        for token in stdout.split_whitespace() {
            if let Some(path) = token.strip_prefix("-I") {
                artifacts.include_dirs.push(PathBuf::from(path));
            } else if let Some(path) = token.strip_prefix("-L") {
                artifacts.link_search_dirs.push(PathBuf::from(path));
            } else if let Some(name) = token.strip_prefix("-l") {
                artifacts.libraries.push(Library {
                    name: name.to_owned(),
                    kind: if prefer_static {
                        LinkKind::Static
                    } else {
                        LinkKind::Dynamic
                    },
                });
            }
        }
        return Some(artifacts);
    }

    None
}

fn generate_bindings(manifest_dir: &Path) {
    #[cfg(feature = "generate-bindings")]
    {
        let header = manifest_dir
            .join("src")
            .join("shim")
            .join("jpegli_rs_shim.h");
        let out_path =
            PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR")).join("bindings.rs");
        let bindings = bindgen::Builder::default()
            .header(header.display().to_string())
            .allowlist_function("jpegli_rs_.*")
            .allowlist_type("jpegli_rs_.*")
            .allowlist_var("JPEGLI_RS_.*")
            .generate()
            .expect("failed to generate bindings");
        bindings
            .write_to_file(&out_path)
            .expect("failed to write generated bindings");
    }

    #[cfg(not(feature = "generate-bindings"))]
    let _ = manifest_dir;
}
