//! Builds the native C ABIs (the combined `d2` library: drlg, pathfinding, item, itemdraw, net
//! and util in one archive) for whatever target cargo asked for, and links it statically so the
//! resulting binary needs no shared library beside it. One archive rather than one per library:
//! each Zig static library carries its own copy of the shared packages and runtime, so several of
//! them cannot be linked into one binary.
//!
//! Unlike the npm and NuGet packages, which ship a prebuilt binary per platform, a crate
//! cannot reasonably carry fourteen of them. So the native side is built here from source,
//! which means `zig` has to be on PATH. That is a real requirement and the error below says
//! so plainly rather than failing with a linker error three screens later.

use std::{env, path::PathBuf, process::Command};

fn main() {
    // docs.rs only ever runs `cargo doc`, which never links, and has no zig. Building the
    // engine there would fail the docs build for no gain.
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // PrimeMH: the Zig engine is not copied into this repository. It is fetched once from
    // jaenster/libd2 at a pinned commit into vendor/libd2/upstream (gitignored), then two small
    // tweaks are applied so it also links with the default Windows (MSVC) Rust toolchain.
    let upstream = manifest.join("upstream");
    ensure_upstream(&upstream);
    let drlg = upstream.join("packages").join("wasm");
    let packages = drlg.parent().unwrap().to_path_buf();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    for pkg in ["wasm", "drlg", "pathfinding", "item", "itemdraw", "net", "util", "core", "data", "formats", "save"] {
        println!("cargo:rerun-if-changed={}", packages.join(pkg).join("src").display());
        println!("cargo:rerun-if-changed={}", packages.join(pkg).join("include").display());
    }

    let target = env::var("TARGET").unwrap();
    let zig_target = zig_target_for(&target)
        .unwrap_or_else(|| panic!("libd2: no zig target known for the rust target '{target}'. \
             Open an issue; the mapping lives in build.rs."));

    let status = Command::new("zig")
        .current_dir(&drlg)
        .args([
            "build",
            &format!("-Dtarget={zig_target}"),
            "-Doptimize=ReleaseFast",
            "--cache-dir",
        ])
        // Zig defaults its cache to .zig-cache beside build.zig. That would write into the
        // crate's own sources, which cargo rejects during `cargo package` verification and
        // which is rude to a read-only registry checkout either way.
        .arg(out.join("zig-cache"))
        .arg("--prefix")
        .arg(&out)
        .status()
        .unwrap_or_else(|e| panic!(
            "libd2: could not run `zig` ({e}). The native engine is written in Zig, so building \
             this crate needs zig 0.17+ on PATH — see https://ziglang.org/download/"
        ));
    assert!(status.success(), "libd2: `zig build` failed for target {zig_target}");

    println!("cargo:rustc-link-search=native={}", out.join("lib").display());
    if target.contains("apple") {
        realign(&out.join("lib").join("libd2.a"), &out.join("realign"));
    }
    println!("cargo:rustc-link-lib=static=d2");
    if target.contains("windows") {
        // the zig runtime calls straight into the Windows system libraries
        println!("cargo:rustc-link-lib=ntdll");
        println!("cargo:rustc-link-lib=kernel32");
    }

    // Zig's static library carries no libc dependency of its own, but the Rust target's
    // runtime still needs its usual system libraries.
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
    }
}

/// Apple's linker refuses an archive whose 64-bit object member does not start 8-byte aligned, and
/// where zig's archive writer puts the one object depends on the length of the symbol table before
/// it. Rewriting the archive with `zig ar --format=darwin` pads it into place.
fn realign(archive: &std::path::Path, work: &std::path::Path) {
    let _ = std::fs::remove_dir_all(work);
    std::fs::create_dir_all(work).unwrap();
    let run = |args: &[&std::ffi::OsStr]| {
        let ok = Command::new("zig").current_dir(work).args(args).status().map(|s| s.success()).unwrap_or(false);
        assert!(ok, "libd2: `zig {:?}` failed while realigning the archive", args);
    };
    run(&["ar".as_ref(), "x".as_ref(), archive.as_os_str()]);
    let mut objects: Vec<std::ffi::OsString> = Vec::new();
    for e in std::fs::read_dir(work).unwrap() {
        let p = e.unwrap().path();
        #[allow(unused_mut)]
        let mut perm = std::fs::metadata(&p).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perm.set_mode(0o644);
        }
        std::fs::set_permissions(&p, perm).unwrap();
        objects.push(p.file_name().unwrap().to_owned());
    }
    std::fs::remove_file(archive).unwrap();
    let mut args: Vec<&std::ffi::OsStr> = vec!["ar".as_ref(), "--format=darwin".as_ref(), "rcs".as_ref(), archive.as_os_str()];
    args.extend(objects.iter().map(|o| o.as_os_str()));
    run(&args);
}

/// Rust target triples and zig's differ in the vendor field, so map rather than munge.
fn zig_target_for(rust_target: &str) -> Option<&'static str> {
    Some(match rust_target {
        "x86_64-unknown-linux-gnu" => "x86_64-linux-gnu",
        "aarch64-unknown-linux-gnu" => "aarch64-linux-gnu",
        "arm-unknown-linux-gnueabihf" | "armv7-unknown-linux-gnueabihf" => "arm-linux-gnueabihf",
        "x86_64-unknown-linux-musl" => "x86_64-linux-musl",
        "aarch64-unknown-linux-musl" => "aarch64-linux-musl",
        "riscv64gc-unknown-linux-gnu" => "riscv64-linux-gnu",
        "x86_64-unknown-freebsd" => "x86_64-freebsd",
        "aarch64-unknown-freebsd" => "aarch64-freebsd",
        "x86_64-apple-darwin" => "x86_64-macos",
        "aarch64-apple-darwin" => "aarch64-macos",
        "i686-pc-windows-gnu" => "x86-windows-gnu",
        "x86_64-pc-windows-gnu" => "x86_64-windows-gnu",
        // PrimeMH: the default Windows Rust toolchain
        "x86_64-pc-windows-msvc" => "x86_64-windows-msvc",
        "aarch64-pc-windows-gnullvm" => "aarch64-windows-gnu",
        _ => return None,
    })
}

/// The libd2 commit this binding was taken from. Bump both together.
const UPSTREAM_URL: &str = "https://github.com/jaenster/libd2";
const UPSTREAM_REV: &str = "f6f6f793e56b4c4bc451c17ce3a1d6ac0c04ed86";

/// Clone jaenster/libd2 at UPSTREAM_REV (once) and apply PrimeMH's tweaks to it.
fn ensure_upstream(dir: &std::path::Path) {
    println!("cargo:rerun-if-changed=build.rs");
    let git = |args: &[&str], cwd: &std::path::Path| -> bool {
        Command::new("git").current_dir(cwd).args(args).status().map(|s| s.success()).unwrap_or(false)
    };
    let have_rev = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == UPSTREAM_REV)
        .unwrap_or(false);
    if !have_rev {
        if !dir.join(".git").exists() {
            let _ = std::fs::remove_dir_all(dir);
            let parent = dir.parent().unwrap();
            assert!(
                git(&["clone", "--quiet", UPSTREAM_URL, dir.file_name().unwrap().to_str().unwrap()], parent),
                "libd2: could not clone {UPSTREAM_URL} (is git installed and on PATH?)"
            );
        } else {
            git(&["fetch", "--quiet", "origin"], dir);
        }
        assert!(git(&["checkout", "--quiet", "--force", UPSTREAM_REV], dir), "libd2: could not check out {UPSTREAM_REV}");
    }

    // A Rust MSVC build links with link.exe, which has no compiler_rt of its own.
    let build_zig = dir.join("packages").join("wasm").join("build.zig");
    let src = std::fs::read_to_string(&build_zig).expect("libd2: missing packages/wasm/build.zig");
    let anchor = "const lib = b.addLibrary(.{ .name = \"d2\", .linkage = .static, .root_module = mod });";
    let tweak = "        if (target.result.os.tag == .windows and target.result.abi == .msvc) lib.bundle_compiler_rt = true;\n";
    if !src.contains("lib.bundle_compiler_rt = true;") {
        // insert on the line after the anchor (works with \n and \r\n line endings)
        let at = src.find(anchor).expect("libd2: build.zig changed upstream; update PrimeMH's tweak");
        let line_end = src[at..].find('\n').map(|i| at + i + 1).unwrap_or(src.len());
        let mut patched = String::with_capacity(src.len() + tweak.len());
        patched.push_str(&src[..line_end]);
        patched.push_str(tweak);
        patched.push_str(&src[line_end..]);
        std::fs::write(&build_zig, patched).unwrap();
    }
}
