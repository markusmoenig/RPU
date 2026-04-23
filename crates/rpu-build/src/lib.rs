use anyhow::{Context, Result, bail};
use image::{ImageBuffer, Rgba};
use rpu_core::{Diagnostic, RpuProject};
use std::fs;
use std::io::Read;
use std::net::TcpListener;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use tiny_http::{Header, Response, Server, StatusCode};

pub fn new_project(name: &str, path: Option<&Path>) -> Result<()> {
    let root = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(name));
    RpuProject::create(&root, name)?;
    println!("Created RPU project at {}", root.display());
    Ok(())
}

pub fn run_project(project_root: &Path) -> Result<()> {
    let project = RpuProject::load(project_root)?;
    rpu_runtime::run(project)
}

pub fn build_project(project_root: &Path) -> Result<()> {
    let project = RpuProject::load(project_root)?;
    let compiled = project.compile()?;
    let build_dir = project.root().join("build");
    fs::create_dir_all(&build_dir)
        .with_context(|| format!("failed to create {}", build_dir.display()))?;

    let summary = format!(
        "RPU build placeholder\nproject = {}\nversion = {}\nroot = {}\nscene_defs = {}\nscene_files = {}\nscripts = {}\ncameras = {}\nrects = {}\nsprites = {}\nhandlers = {}\nops = {}\nassets = {}\nwarnings = {}\nerrors = {}\n\n{}",
        compiled.name,
        compiled.version,
        project.root().display()
        ,
        compiled.scene_count(),
        compiled.scenes.len(),
        compiled.bytecode_scripts.len(),
        compiled.camera_count(),
        compiled.rect_count(),
        compiled.sprite_count(),
        compiled.handler_count(),
        compiled.op_count(),
        compiled.assets.len(),
        compiled.warning_count(),
        compiled.error_count(),
        format_diagnostics(&compiled.diagnostics)
    );
    fs::write(build_dir.join("BUILD.txt"), summary)
        .with_context(|| format!("failed to write {}", build_dir.join("BUILD.txt").display()))?;
    export_terrain_debug_images(&compiled, project.root(), &build_dir)?;

    println!("Wrote build placeholder to {}", build_dir.display());
    Ok(())
}

pub fn build_web_project(project_root: &Path) -> Result<()> {
    ensure_web_prerequisites()?;
    let project = RpuProject::load(project_root)?;
    let compiled = project.compile()?;
    if compiled.has_errors() {
        bail!("project has compile errors; fix them before building for web");
    }

    let out_root = project.root().join("build/web");
    let app_root = out_root.join(".app");
    let src_root = app_root.join("src");
    fs::create_dir_all(&src_root)
        .with_context(|| format!("failed to create {}", src_root.display()))?;

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("failed to resolve repository root")?
        .to_path_buf();

    fs::write(app_root.join("Cargo.toml"), generated_web_cargo_toml(&repo_root))
        .with_context(|| format!("failed to write {}", app_root.join("Cargo.toml").display()))?;
    let _ = fs::remove_file(src_root.join("main.rs"));
    fs::write(src_root.join("lib.rs"), generated_web_main_rs(&project, &compiled))
        .with_context(|| format!("failed to write {}", src_root.join("lib.rs").display()))?;

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
        ])
        .arg(app_root.join("Cargo.toml"))
        .status()
        .context("failed to execute cargo build for web export")?;
    if !status.success() {
        bail!("cargo build for web export failed");
    }

    let wasm_bindgen = find_wasm_bindgen()
        .context("wasm-bindgen CLI is required; install it with `cargo install wasm-bindgen-cli`")?;
    let wasm_path = app_root
        .join("target/wasm32-unknown-unknown/release/rpu_web_export.wasm");
    let status = Command::new(wasm_bindgen)
        .args(["--target", "web", "--out-dir"])
        .arg(&out_root)
        .args(["--no-typescript"])
        .arg(&wasm_path)
        .status()
        .context("failed to run wasm-bindgen for web export")?;
    if !status.success() {
        bail!("wasm-bindgen failed for web export");
    }

    fs::write(out_root.join("index.html"), generated_web_index_html(&compiled.name))
        .with_context(|| format!("failed to write {}", out_root.join("index.html").display()))?;

    println!("Prepared web build at {}", out_root.display());
    Ok(())
}

pub fn serve_web_project(project_root: &Path, port: u16) -> Result<()> {
    build_web_project(project_root)?;
    let web_root = project_root.join("build/web");
    let addr = format!("127.0.0.1:{port}");
    let probe = TcpListener::bind(&addr)
        .with_context(|| format!("port {} is not available", port))?;
    drop(probe);
    let server =
        Server::http(&addr).map_err(|error| anyhow::anyhow!("failed to start server at {addr}: {error}"))?;
    println!("Serving {} at http://{addr}", web_root.display());

    for request in server.incoming_requests() {
        let url = request.url().trim_start_matches('/');
        let path = if url.is_empty() {
            web_root.join("index.html")
        } else {
            web_root.join(url)
        };
        let path = if path.is_dir() {
            path.join("index.html")
        } else {
            path
        };

        if !path.exists() {
            let _ = request.respond(
                Response::from_string("Not Found").with_status_code(StatusCode(404)),
            );
            continue;
        }

        let mut bytes = Vec::new();
        fs::File::open(&path)
            .and_then(|mut file| file.read_to_end(&mut bytes))
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut response = Response::from_data(bytes);
        if let Some(content_type) = content_type_for(&path) {
            response.add_header(
                Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                    .expect("valid content-type header"),
            );
        }
        let _ = request.respond(response);
    }

    Ok(())
}

pub fn export_xcode(project_root: &Path, output: Option<&Path>) -> Result<()> {
    ensure_xcode_export_prerequisites()?;
    let project = RpuProject::load(project_root)?;
    let compiled = project.compile()?;
    let output_root = output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project.root().join("build/apple"));
    if output_root.exists() && !output_root.is_dir() {
        bail!("output path is not a directory: {}", output_root.display());
    }

    fs::create_dir_all(&output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;
    let app_dir = output_root.join("App");
    let rust_dir = output_root.join("RustBridge");
    let rust_src_dir = rust_dir.join("src");
    let bundle_project_dir = output_root.join("Project");
    let xcodeproj_dir = output_root.join("RPUAppleApp.xcodeproj");
    let workspace_dir = xcodeproj_dir.join("project.xcworkspace");

    fs::create_dir_all(&app_dir)
        .with_context(|| format!("failed to create {}", app_dir.display()))?;
    fs::create_dir_all(&rust_src_dir)
        .with_context(|| format!("failed to create {}", rust_src_dir.display()))?;
    fs::create_dir_all(&bundle_project_dir)
        .with_context(|| format!("failed to create {}", bundle_project_dir.display()))?;
    fs::create_dir_all(&workspace_dir)
        .with_context(|| format!("failed to create {}", workspace_dir.display()))?;

    copy_project_export_sources(project.root(), &bundle_project_dir)?;

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("failed to resolve repository root")?
        .to_path_buf();
    let app_display_name = project
        .display_name()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&compiled.name)
        .to_string();
    let app_identifier = project
        .bundle_id()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| bundle_identifier_for(&compiled.name));
    let app_identifier = sanitize_bundle_identifier(&app_identifier);
    let default_window_size = scaled_window_size(
        compiled.window.width,
        compiled.window.height,
        compiled.window.default_scale,
    );

    fs::write(
        rust_dir.join("Cargo.toml"),
        generated_xcode_cargo_toml(&repo_root),
    )
    .with_context(|| format!("failed to write {}", rust_dir.join("Cargo.toml").display()))?;
    fs::write(
        rust_src_dir.join("lib.rs"),
        generated_xcode_lib_rs(),
    )
    .with_context(|| format!("failed to write {}", rust_src_dir.join("lib.rs").display()))?;
    build_generated_xcode_bridge(&rust_dir)?;

    fs::write(
        app_dir.join("RPUAppleApp.swift"),
        generated_xcode_app_swift(&app_display_name, default_window_size),
    )
    .with_context(|| format!("failed to write {}", app_dir.join("RPUAppleApp.swift").display()))?;
    fs::write(
        app_dir.join("main.swift"),
        generated_xcode_main_swift(),
    )
    .with_context(|| format!("failed to write {}", app_dir.join("main.swift").display()))?;
    fs::write(
        app_dir.join("ContentView.swift"),
        generated_xcode_content_view_swift(),
    )
    .with_context(|| format!("failed to write {}", app_dir.join("ContentView.swift").display()))?;
    fs::write(
        app_dir.join("MetalView.swift"),
        generated_xcode_metal_view_swift(),
    )
    .with_context(|| format!("failed to write {}", app_dir.join("MetalView.swift").display()))?;
    fs::write(
        app_dir.join("RPUFFI.swift"),
        generated_xcode_ffi_swift(),
    )
    .with_context(|| format!("failed to write {}", app_dir.join("RPUFFI.swift").display()))?;

    fs::write(
        xcodeproj_dir.join("project.pbxproj"),
        generated_xcode_pbxproj(&app_display_name, &app_identifier),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            xcodeproj_dir.join("project.pbxproj").display()
        )
    })?;
    fs::write(
        workspace_dir.join("contents.xcworkspacedata"),
        generated_xcode_workspace_data(),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            workspace_dir.join("contents.xcworkspacedata").display()
        )
    })?;

    let readme = format!(
        "# Xcode Export\n\nProject: {}\nVersion: {}\nBundle Identifier: {}\n\nThis export uses the native Apple view/surface created by Xcode and renders into that `CAMetalLayer` from Rust via FFI. It does **not** use a second renderer.\n\n## Generated Layout\n\n- `App/` SwiftUI macOS host app\n- `RustBridge/` Rust static library crate used by Xcode\n- `Project/` bundled RPU scenes, scripts, and assets\n- `RPUAppleApp.xcodeproj/` Xcode project\n\n## Build Notes\n\n- Open `RPUAppleApp.xcodeproj` in Xcode\n- Build and run the macOS app target\n- The export already includes a prebuilt Rust static library in `RustBridge/build/`\n- Rust render output is presented directly into a `CAMetalLayer`\n- If you change the generated Rust bridge, rebuild it manually:\n  - `cd RustBridge`\n  - `cargo build`\n  - `mkdir -p build && cp target/debug/librpu_apple_export.a build/librpu_apple_export.a`\n\n## Diagnostics\n\n{}\n",
        compiled.name,
        compiled.version,
        app_identifier,
        format_diagnostics(&compiled.diagnostics)
    );
    fs::write(output_root.join("README.md"), readme).with_context(|| {
        format!("failed to write {}", output_root.join("README.md").display())
    })?;

    println!("Prepared Xcode export at {}", output_root.display());
    Ok(())
}

fn build_generated_xcode_bridge(rust_dir: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(rust_dir.join("Cargo.toml"))
        .status()
        .context("failed to execute cargo build for generated Xcode bridge")?;
    if !status.success() {
        bail!("cargo build for generated Xcode bridge failed");
    }

    let built_lib = rust_dir.join("target/debug/librpu_apple_export.a");
    let build_dir = rust_dir.join("build");
    fs::create_dir_all(&build_dir)
        .with_context(|| format!("failed to create {}", build_dir.display()))?;
    fs::copy(&built_lib, build_dir.join("librpu_apple_export.a")).with_context(|| {
        format!(
            "failed to copy generated Xcode bridge archive from {}",
            built_lib.display()
        )
    })?;
    Ok(())
}

fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return "none\n".to_string();
    }

    let mut out = String::new();
    for diagnostic in diagnostics {
        use std::fmt::Write as _;
        match (&diagnostic.path, diagnostic.line) {
            (Some(path), Some(line)) => {
                let _ = writeln!(
                    out,
                    "- {:?}: {} ({}:{})",
                    diagnostic.severity,
                    diagnostic.message,
                    path.display(),
                    line
                );
            }
            (Some(path), None) => {
                let _ = writeln!(
                    out,
                    "- {:?}: {} ({})",
                    diagnostic.severity,
                    diagnostic.message,
                    path.display()
                );
            }
            (None, Some(line)) => {
                let _ = writeln!(
                    out,
                    "- {:?}: {} (line {})",
                    diagnostic.severity,
                    diagnostic.message,
                    line
                );
            }
            (None, None) => {
                let _ = writeln!(out, "- {:?}: {}", diagnostic.severity, diagnostic.message);
            }
        }
    }
    out
}

fn generated_web_cargo_toml(repo_root: &Path) -> String {
    format!(
        r#"[package]
name = "rpu_web_export"
version = "0.1.0"
edition = "2024"

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1.0"
wasm-bindgen = "=0.2.100"
rpu-core = {{ path = "{}" }}
rpu-runtime = {{ path = "{}" }}
rpu-scenevm = {{ path = "{}" }}
"#,
        repo_root.join("crates/rpu-core").display(),
        repo_root.join("crates/rpu-runtime").display(),
        repo_root.join("crates/rpu-scenevm").display(),
    )
}

fn generated_web_main_rs(project: &RpuProject, compiled: &rpu_core::CompiledProject) -> String {
    let manifest = rust_raw_literal(&canonical_display(project.root().join("rpu.toml")));
    let scenes = compiled
        .scenes
        .iter()
        .map(|scene| {
            let absolute = rust_raw_literal(&canonical_display(project.root().join(&scene.relative_path)));
            format!(
                r#"(PathBuf::from("{}"), include_str!({}).to_string())"#,
                scene.relative_path.display(),
                absolute
            )
        })
        .collect::<Vec<_>>()
        .join(",\n        ");
    let scripts = compiled
        .scripts
        .iter()
        .map(|script| {
            let absolute = rust_raw_literal(&canonical_display(project.root().join(&script.relative_path)));
            format!(
                r#"(PathBuf::from("{}"), include_str!({}).to_string())"#,
                script.relative_path.display(),
                absolute
            )
        })
        .collect::<Vec<_>>()
        .join(",\n        ");
    let assets = compiled
        .assets
        .iter()
        .map(|asset| {
            let absolute = rust_raw_literal(&canonical_display(project.root().join(asset)));
            format!(
                r#"
    rpu_scenevm::register_web_asset("{}", include_bytes!({}));
    asset_files.push((PathBuf::from("{}"), include_bytes!({}).to_vec()));"#,
                asset.display(),
                absolute,
                asset.display(),
                absolute
            )
        })
        .collect::<String>();

    format!(
        r#"use anyhow::Result;
use rpu_core::BundledProject;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {{
    let mut asset_files: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    {assets}

    let project = BundledProject::new(
        include_str!({manifest}),
        vec![
            {scenes}
        ],
        vec![
            {scripts}
        ],
        asset_files,
    ).map_err(|error| JsValue::from_str(&format!("bundled project init failed: {{error:#}}")))?;

    rpu_runtime::run_bundled(project, "assets")
        .map_err(|error| JsValue::from_str(&format!("web runtime start failed: {{error:#}}")))?;
    Ok(())
}}
"#,
        assets = assets,
        manifest = manifest,
        scenes = scenes,
        scripts = scripts,
    )
}

fn copy_project_export_sources(project_root: &Path, out_dir: &Path) -> Result<()> {
    let files = ["rpu.toml"];
    for file in files {
        let src = project_root.join(file);
        if src.exists() {
            fs::copy(&src, out_dir.join(file))
                .with_context(|| format!("failed to copy {}", src.display()))?;
        }
    }
    for dir in ["assets", "scenes", "scripts"] {
        let src = project_root.join(dir);
        if src.exists() {
            copy_dir_all(&src, &out_dir.join(dir))?;
        }
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("failed to create {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to read {}", src.display()))? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .with_context(|| format!("failed to copy {}", src_path.display()))?;
        }
    }
    Ok(())
}

fn bundle_identifier_for(name: &str) -> String {
    let mut slug = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('.') {
            slug.push('.');
        }
    }
    while slug.ends_with('.') {
        slug.pop();
    }
    if slug.is_empty() {
        "org.rpu.app".to_string()
    } else {
        format!("org.rpu.{slug}")
    }
}

fn sanitize_bundle_identifier(identifier: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_was_separator = false;
    for ch in identifier.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            previous_was_separator = false;
            ch.to_ascii_lowercase()
        } else if ch == '.' || ch == '-' {
            if previous_was_separator {
                continue;
            }
            previous_was_separator = true;
            ch
        } else {
            if previous_was_separator {
                continue;
            }
            previous_was_separator = true;
            '-'
        };
        sanitized.push(mapped);
    }
    sanitized = sanitized
        .trim_matches(|ch: char| ch == '.' || ch == '-')
        .to_string();
    if sanitized.is_empty() {
        "org.rpu.app".to_string()
    } else {
        sanitized
    }
}

fn scaled_window_size(width: u32, height: u32, default_scale: f32) -> (u32, u32) {
    let scale = if default_scale.is_finite() && default_scale > 0.0 {
        default_scale
    } else {
        1.0
    };
    let scaled_width = ((width.max(1) as f32) * scale).round().max(1.0) as u32;
    let scaled_height = ((height.max(1) as f32) * scale).round().max(1.0) as u32;
    (scaled_width, scaled_height)
}

fn generated_xcode_cargo_toml(repo_root: &Path) -> String {
    format!(
        r#"[package]
name = "rpu_apple_export"
version = "0.1.0"
edition = "2024"

[workspace]

[lib]
crate-type = ["staticlib"]

[dependencies]
anyhow = "1.0"
rpu-core = {{ path = "{}" }}
rpu-runtime = {{ path = "{}" }}
rpu-scenevm = {{ path = "{}" }}
"#,
        repo_root.join("crates/rpu-core").display(),
        repo_root.join("crates/rpu-runtime").display(),
        repo_root.join("crates/rpu-scenevm").display(),
    )
}

fn generated_xcode_lib_rs() -> String {
    r#"use std::ffi::{CStr, c_char, c_void};
use std::path::Path;

#[cfg(target_os = "macos")]
use rpu_core::RpuProject;
#[cfg(target_os = "macos")]
use rpu_runtime::RuntimeApp;
#[cfg(target_os = "macos")]
use rpu_scenevm::MetalLayerRunner;

#[cfg(target_os = "macos")]
struct RpuAppleRunner {
    runner: MetalLayerRunner<RuntimeApp>,
}

#[cfg(target_os = "macos")]
fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().ok().map(ToString::to_string)
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_create(
    project_root: *const c_char,
    layer_ptr: *mut c_void,
    width: u32,
    height: u32,
    scale: f32,
) -> *mut c_void {
    let Some(project_root) = cstr_to_string(project_root) else {
        return std::ptr::null_mut();
    };
    let Ok(project) = RpuProject::load(Path::new(&project_root)) else {
        return std::ptr::null_mut();
    };
    let Ok(app) = RuntimeApp::new(project) else {
        return std::ptr::null_mut();
    };
    let Ok(runner) = MetalLayerRunner::new(app, layer_ptr, width, height, scale) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(RpuAppleRunner { runner })).cast()
}

#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_create(
    _project_root: *const c_char,
    _layer_ptr: *mut c_void,
    _width: u32,
    _height: u32,
    _scale: f32,
) -> *mut c_void {
    std::ptr::null_mut()
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_destroy(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr.cast::<RpuAppleRunner>())); }
    }
}

#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_destroy(_ptr: *mut c_void) {}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_resize(
    ptr: *mut c_void,
    width: u32,
    height: u32,
    scale: f32,
) {
    if let Some(runner) = unsafe { ptr.cast::<RpuAppleRunner>().as_mut() } {
        runner.runner.resize(width, height, scale);
    }
}

#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_resize(
    _ptr: *mut c_void,
    _width: u32,
    _height: u32,
    _scale: f32,
) {}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_render(ptr: *mut c_void) -> i32 {
    if let Some(runner) = unsafe { ptr.cast::<RpuAppleRunner>().as_mut() } {
        return if runner.runner.render().is_ok() { 0 } else { -1 };
    }
    -1
}

#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_render(_ptr: *mut c_void) -> i32 {
    -1
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_down(ptr: *mut c_void, key: *const c_char) {
    if let (Some(runner), Some(key)) = (unsafe { ptr.cast::<RpuAppleRunner>().as_mut() }, cstr_to_string(key)) {
        runner.runner.key_down(&key);
    }
}

#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_down(_ptr: *mut c_void, _key: *const c_char) {}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_up(ptr: *mut c_void, key: *const c_char) {
    if let (Some(runner), Some(key)) = (unsafe { ptr.cast::<RpuAppleRunner>().as_mut() }, cstr_to_string(key)) {
        runner.runner.key_up(&key);
    }
}

#[cfg(not(target_os = "macos"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_up(_ptr: *mut c_void, _key: *const c_char) {}
"#
    .to_string()
}

fn generated_xcode_app_swift(app_name: &str, size: (u32, u32)) -> String {
    let escaped_name = app_name.replace('"', "\\\"");
    format!(
        r#"import AppKit

enum RPUWindowConfig {{
    static let contentWidth: CGFloat = {width}
    static let contentHeight: CGFloat = {height}
}}

final class AppDelegate: NSObject, NSApplicationDelegate {{
    private var window: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {{
        installMainMenu()

        let contentSize = NSSize(width: RPUWindowConfig.contentWidth, height: RPUWindowConfig.contentHeight)
        let styleMask: NSWindow.StyleMask = [.titled, .closable, .miniaturizable, .resizable]
        let window = NSWindow(
            contentRect: NSRect(origin: .zero, size: contentSize),
            styleMask: styleMask,
            backing: .buffered,
            defer: false
        )
        window.title = "{app_name}"
        window.isOpaque = true
        window.backgroundColor = .black
        window.contentMinSize = contentSize
        window.contentAspectRatio = contentSize
        window.setContentSize(contentSize)
        window.center()

        let metalView = MetalContainer(frame: NSRect(origin: .zero, size: contentSize))
        metalView.autoresizingMask = [.width, .height]
        window.contentView = metalView
        window.makeFirstResponder(metalView)
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        self.window = window
    }}

    private func installMainMenu() {{
        let mainMenu = NSMenu()
        let appMenuItem = NSMenuItem()
        mainMenu.addItem(appMenuItem)

        let appMenu = NSMenu()
        let quitItem = NSMenuItem(
            title: "Quit {app_name}",
            action: #selector(NSApplication.terminate(_:)),
            keyEquivalent: "q"
        )
        quitItem.keyEquivalentModifierMask = [.command]
        appMenu.addItem(quitItem)

        appMenuItem.submenu = appMenu
        NSApp.mainMenu = mainMenu
    }}

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {{
        true
    }}
}}
"#,
        app_name = escaped_name,
        width = size.0,
        height = size.1
    )
}

fn generated_xcode_main_swift() -> String {
    r#"import AppKit

let app = NSApplication.shared
enum RPUAppBootstrap {
    static let delegate = AppDelegate()
}
app.delegate = RPUAppBootstrap.delegate
app.setActivationPolicy(.regular)
app.activate(ignoringOtherApps: true)
app.run()
"#
    .to_string()
}

fn generated_xcode_content_view_swift() -> String {
    "import Foundation\n"
    .to_string()
}

fn generated_xcode_ffi_swift() -> String {
    r#"import Foundation
import QuartzCore

@_silgen_name("rpu_runner_create")
func rpu_runner_create(
    _ project_root: UnsafePointer<CChar>?,
    _ layer_ptr: UnsafeMutableRawPointer?,
    _ width: UInt32,
    _ height: UInt32,
    _ scale: Float
) -> UnsafeMutableRawPointer?

@_silgen_name("rpu_runner_destroy")
func rpu_runner_destroy(_ ptr: UnsafeMutableRawPointer?)

@_silgen_name("rpu_runner_resize")
func rpu_runner_resize(_ ptr: UnsafeMutableRawPointer?, _ width: UInt32, _ height: UInt32, _ scale: Float)

@_silgen_name("rpu_runner_render")
func rpu_runner_render(_ ptr: UnsafeMutableRawPointer?) -> Int32

@_silgen_name("rpu_runner_key_down")
func rpu_runner_key_down(_ ptr: UnsafeMutableRawPointer?, _ key: UnsafePointer<CChar>?)

@_silgen_name("rpu_runner_key_up")
func rpu_runner_key_up(_ ptr: UnsafeMutableRawPointer?, _ key: UnsafePointer<CChar>?)

final class RPUHandle {
    private var runner: UnsafeMutableRawPointer?
    private weak var layer: CAMetalLayer?
    private let projectRoot: String

    init?(layer: CAMetalLayer, size: CGSize, scale: CGFloat) {
        guard let projectURL = Bundle.main.resourceURL?.appendingPathComponent("Project") else {
            return nil
        }
        self.layer = layer
        self.projectRoot = projectURL.path
        let ptr = Unmanaged.passUnretained(layer).toOpaque()
        let width = UInt32(max(layer.drawableSize.width, size.width * scale))
        let height = UInt32(max(layer.drawableSize.height, size.height * scale))
        self.projectRoot.withCString { cstr in
            runner = rpu_runner_create(cstr, ptr, width, height, Float(scale))
        }
        if runner == nil {
            return nil
        }
    }

    func resize(size: CGSize, scale: CGFloat) {
        guard let runner else { return }
        let drawable = layer?.drawableSize ?? CGSize(width: size.width * scale, height: size.height * scale)
        let width = UInt32(max(drawable.width, 1))
        let height = UInt32(max(drawable.height, 1))
        rpu_runner_resize(runner, width, height, Float(scale))
    }

    func render() {
        guard let runner else { return }
        _ = rpu_runner_render(runner)
    }

    func keyDown(_ key: String) {
        guard let runner else { return }
        key.withCString { cstr in
            rpu_runner_key_down(runner, cstr)
        }
    }

    func keyUp(_ key: String) {
        guard let runner else { return }
        key.withCString { cstr in
            rpu_runner_key_up(runner, cstr)
        }
    }

    deinit {
        if let runner {
            rpu_runner_destroy(runner)
        }
    }
}
"#
    .to_string()
}

fn generated_xcode_metal_view_swift() -> String {
    r#"import AppKit
import QuartzCore
import Metal

final class MetalContainer: NSView {
    private let metalLayer = CAMetalLayer()
    private var handle: RPUHandle?
    private var renderTimer: Timer?

    override var acceptsFirstResponder: Bool { true }
    override var canBecomeKeyView: Bool { true }

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        wantsLayer = true
        layer?.backgroundColor = NSColor.black.cgColor
        metalLayer.device = MTLCreateSystemDefaultDevice()
        metalLayer.pixelFormat = .bgra8Unorm
        metalLayer.framebufferOnly = false
        metalLayer.backgroundColor = NSColor.black.cgColor
        layer = metalLayer
        renderTimer = Timer.scheduledTimer(withTimeInterval: 1.0 / 60.0, repeats: true) { [weak self] _ in
            self?.drawFrame()
        }
        if let renderTimer {
            RunLoop.main.add(renderTimer, forMode: .common)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    deinit {
        renderTimer?.invalidate()
    }

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.window?.backgroundColor = .black
            self.window?.isOpaque = true
            self.window?.makeFirstResponder(self)
        }
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
        true
    }

    override func layout() {
        super.layout()
        metalLayer.frame = bounds
        let scale = window?.backingScaleFactor ?? NSScreen.main?.backingScaleFactor ?? 2.0
        metalLayer.contentsScale = scale
        metalLayer.drawableSize = CGSize(width: bounds.width * scale, height: bounds.height * scale)

        if handle == nil && bounds.width > 0 && bounds.height > 0 {
            handle = RPUHandle(layer: metalLayer, size: bounds.size, scale: scale)
        } else {
            handle?.resize(size: bounds.size, scale: scale)
        }
    }

    private func drawFrame() {
        handle?.render()
    }

    @discardableResult
    override func becomeFirstResponder() -> Bool {
        true
    }

    override func mouseDown(with event: NSEvent) {
        window?.makeFirstResponder(self)
        super.mouseDown(with: event)
    }

    private func keyIdentifier(for event: NSEvent) -> String? {
        switch event.keyCode {
        case 123: return "ArrowLeft"
        case 124: return "ArrowRight"
        case 125: return "ArrowDown"
        case 126: return "ArrowUp"
        case 36: return "Enter"
        case 76: return "Enter"
        case 49: return "Space"
        default:
            guard let chars = event.charactersIgnoringModifiers, !chars.isEmpty else {
                return nil
            }
            if chars == "\r" || chars == "\n" {
                return "Enter"
            }
            return chars.uppercased()
        }
    }

    override func keyDown(with event: NSEvent) {
        if event.modifierFlags.contains(.command) {
            super.keyDown(with: event)
            return
        }
        if let key = keyIdentifier(for: event) {
            handle?.keyDown(key)
            drawFrame()
        } else {
            super.keyDown(with: event)
        }
    }

    override func keyUp(with event: NSEvent) {
        if event.modifierFlags.contains(.command) {
            super.keyUp(with: event)
            return
        }
        if let key = keyIdentifier(for: event) {
            handle?.keyUp(key)
            drawFrame()
        } else {
            super.keyUp(with: event)
        }
    }
}
"#
    .to_string()
}

fn generated_xcode_workspace_data() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<Workspace
   version = "1.0">
   <FileRef
      location = "self:">
   </FileRef>
</Workspace>
"#
    .to_string()
}

fn generated_xcode_pbxproj(app_display_name: &str, bundle_id: &str) -> String {
    let escaped_name = app_display_name.replace('"', "\\\"");
    let escaped_bundle = bundle_id.replace('"', "\\\"");
    format!(
        r#"// !$*UTF8*$!
{{
	archiveVersion = 1;
	classes = {{
	}};
	objectVersion = 77;
	objects = {{

/* Begin PBXBuildFile section */
		AA0000010000000000000001 /* librpu_apple_export.a in Frameworks */ = {{isa = PBXBuildFile; fileRef = AA0001010000000000000001 /* librpu_apple_export.a */; }};
		AA0000010000000000000002 /* Project in Resources */ = {{isa = PBXBuildFile; fileRef = AA0001010000000000000002 /* Project */; }};
		AA0000010000000000000003 /* AudioToolbox.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AA0001010000000000000004 /* AudioToolbox.framework */; }};
		AA0000010000000000000004 /* CoreAudio.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AA0001010000000000000005 /* CoreAudio.framework */; }};
		AA0000010000000000000005 /* AudioUnit.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AA0001010000000000000006 /* AudioUnit.framework */; }};
		AA0000010000000000000006 /* Carbon.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AA0001010000000000000007 /* Carbon.framework */; }};
/* End PBXBuildFile section */

/* Begin PBXFileReference section */
		AA0001010000000000000001 /* librpu_apple_export.a */ = {{isa = PBXFileReference; lastKnownFileType = archive.ar; path = RustBridge/build/librpu_apple_export.a; sourceTree = "<group>"; }};
		AA0001010000000000000002 /* Project */ = {{isa = PBXFileReference; lastKnownFileType = folder; path = Project; sourceTree = "<group>"; }};
		AA0001010000000000000003 /* {escaped_name}.app */ = {{isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = "{escaped_name}.app"; sourceTree = BUILT_PRODUCTS_DIR; }};
		AA0001010000000000000004 /* AudioToolbox.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = AudioToolbox.framework; path = System/Library/Frameworks/AudioToolbox.framework; sourceTree = SDKROOT; }};
		AA0001010000000000000005 /* CoreAudio.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = CoreAudio.framework; path = System/Library/Frameworks/CoreAudio.framework; sourceTree = SDKROOT; }};
		AA0001010000000000000006 /* AudioUnit.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = AudioUnit.framework; path = System/Library/Frameworks/AudioUnit.framework; sourceTree = SDKROOT; }};
		AA0001010000000000000007 /* Carbon.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = Carbon.framework; path = System/Library/Frameworks/Carbon.framework; sourceTree = SDKROOT; }};
/* End PBXFileReference section */

/* Begin PBXFileSystemSynchronizedRootGroup section */
		AA0002010000000000000001 /* App */ = {{
			isa = PBXFileSystemSynchronizedRootGroup;
			path = App;
			sourceTree = "<group>";
		}};
/* End PBXFileSystemSynchronizedRootGroup section */

/* Begin PBXFrameworksBuildPhase section */
		AA0003010000000000000001 /* Frameworks */ = {{
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AA0000010000000000000003 /* AudioToolbox.framework in Frameworks */,
				AA0000010000000000000004 /* CoreAudio.framework in Frameworks */,
				AA0000010000000000000005 /* AudioUnit.framework in Frameworks */,
				AA0000010000000000000006 /* Carbon.framework in Frameworks */,
				AA0000010000000000000001 /* librpu_apple_export.a in Frameworks */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXFrameworksBuildPhase section */

/* Begin PBXGroup section */
		AA0004010000000000000001 = {{
			isa = PBXGroup;
			children = (
				AA0002010000000000000001 /* App */,
				AA0001010000000000000002 /* Project */,
				AA0001010000000000000004 /* AudioToolbox.framework */,
				AA0001010000000000000005 /* CoreAudio.framework */,
				AA0001010000000000000006 /* AudioUnit.framework */,
				AA0001010000000000000007 /* Carbon.framework */,
				AA0001010000000000000001 /* librpu_apple_export.a */,
				AA0004010000000000000002 /* Products */,
			);
			sourceTree = "<group>";
		}};
		AA0004010000000000000002 /* Products */ = {{
			isa = PBXGroup;
			children = (
				AA0001010000000000000003 /* {escaped_name}.app */,
			);
			name = Products;
			sourceTree = "<group>";
		}};
/* End PBXGroup section */

/* Begin PBXNativeTarget section */
		AA0005010000000000000001 /* RPUAppleApp */ = {{
			isa = PBXNativeTarget;
			buildConfigurationList = AA0009010000000000000001 /* Build configuration list for PBXNativeTarget "RPUAppleApp" */;
			buildPhases = (
				AA0006010000000000000001 /* Sources */,
				AA0003010000000000000001 /* Frameworks */,
				AA0006010000000000000002 /* Resources */,
			);
			buildRules = (
			);
			dependencies = (
			);
			fileSystemSynchronizedGroups = (
				AA0002010000000000000001 /* App */,
			);
			name = RPUAppleApp;
			productName = "{escaped_name}";
			productReference = AA0001010000000000000003 /* {escaped_name}.app */;
			productType = "com.apple.product-type.application";
		}};
/* End PBXNativeTarget section */

/* Begin PBXProject section */
		AA0007010000000000000001 /* Project object */ = {{
			isa = PBXProject;
			attributes = {{
				BuildIndependentTargetsInParallel = 1;
				LastSwiftUpdateCheck = 2610;
				LastUpgradeCheck = 2610;
				TargetAttributes = {{
					AA0005010000000000000001 = {{
						CreatedOnToolsVersion = 26.1.1;
					}};
				}};
			}};
			buildConfigurationList = AA0009010000000000000002 /* Build configuration list for PBXProject "RPUAppleApp" */;
			developmentRegion = en;
			hasScannedForEncodings = 0;
			knownRegions = (
				en,
				Base,
			);
			mainGroup = AA0004010000000000000001;
			minimizedProjectReferenceProxies = 1;
			preferredProjectObjectVersion = 77;
			productRefGroup = AA0004010000000000000002 /* Products */;
			projectDirPath = "";
			projectRoot = "";
			targets = (
				AA0005010000000000000001 /* RPUAppleApp */,
			);
		}};
/* End PBXProject section */

/* Begin PBXResourcesBuildPhase section */
		AA0006010000000000000002 /* Resources */ = {{
			isa = PBXResourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AA0000010000000000000002 /* Project in Resources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXResourcesBuildPhase section */

/* Begin PBXSourcesBuildPhase section */
		AA0006010000000000000001 /* Sources */ = {{
			isa = PBXSourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXSourcesBuildPhase section */

/* Begin XCBuildConfiguration section */
		AA0008010000000000000001 /* Debug */ = {{
			isa = XCBuildConfiguration;
			buildSettings = {{
				ALWAYS_SEARCH_USER_PATHS = NO;
				CODE_SIGN_STYLE = Automatic;
				GENERATE_INFOPLIST_FILE = YES;
				INFOPLIST_KEY_CFBundleDisplayName = "{escaped_name}";
				LD_RUNPATH_SEARCH_PATHS = "@executable_path/../Frameworks";
				LIBRARY_SEARCH_PATHS = (
					"$(inherited)",
					"$(SRCROOT)/RustBridge/build",
				);
				MACOSX_DEPLOYMENT_TARGET = 13.0;
				OTHER_LDFLAGS = (
					"$(inherited)",
					"-framework",
					"AudioToolbox",
					"-framework",
					"CoreAudio",
					"-framework",
					"AudioUnit",
					"-framework",
					"Carbon",
				);
				PRODUCT_BUNDLE_IDENTIFIER = "{escaped_bundle}";
				PRODUCT_NAME = "{escaped_name}";
				SDKROOT = macosx;
				SWIFT_VERSION = 5.0;
			}};
			name = Debug;
		}};
		AA0008010000000000000002 /* Release */ = {{
			isa = XCBuildConfiguration;
			buildSettings = {{
				ALWAYS_SEARCH_USER_PATHS = NO;
				CODE_SIGN_STYLE = Automatic;
				GENERATE_INFOPLIST_FILE = YES;
				INFOPLIST_KEY_CFBundleDisplayName = "{escaped_name}";
				LD_RUNPATH_SEARCH_PATHS = "@executable_path/../Frameworks";
				LIBRARY_SEARCH_PATHS = (
					"$(inherited)",
					"$(SRCROOT)/RustBridge/build",
				);
				MACOSX_DEPLOYMENT_TARGET = 13.0;
				OTHER_LDFLAGS = (
					"$(inherited)",
					"-framework",
					"AudioToolbox",
					"-framework",
					"CoreAudio",
					"-framework",
					"AudioUnit",
					"-framework",
					"Carbon",
				);
				PRODUCT_BUNDLE_IDENTIFIER = "{escaped_bundle}";
				PRODUCT_NAME = "{escaped_name}";
				SDKROOT = macosx;
				SWIFT_VERSION = 5.0;
			}};
			name = Release;
		}};
/* End XCBuildConfiguration section */

/* Begin XCConfigurationList section */
		AA0009010000000000000001 /* Build configuration list for PBXNativeTarget "RPUAppleApp" */ = {{
			isa = XCConfigurationList;
			buildConfigurations = (
				AA0008010000000000000001 /* Debug */,
				AA0008010000000000000002 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		}};
		AA0009010000000000000002 /* Build configuration list for PBXProject "RPUAppleApp" */ = {{
			isa = XCConfigurationList;
			buildConfigurations = (
				AA0008010000000000000001 /* Debug */,
				AA0008010000000000000002 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		}};
/* End XCConfigurationList section */
	}};
	rootObject = AA0007010000000000000001 /* Project object */;
}}
"#
    )
}

fn generated_web_index_html(title: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{}</title>
    <style>
      html, body {{
        margin: 0;
        padding: 0;
        background: #000000;
        width: 100%;
        height: 100%;
        min-height: 100vh;
      }}
      body {{
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
      }}
      canvas {{
        image-rendering: pixelated;
        image-rendering: crisp-edges;
      }}
    </style>
  </head>
  <body>
    <script type="module">
      import init, {{ start }} from "./rpu_web_export.js";
      init()
        .then(() => start())
        .catch((error) => {{
          console.error("RPU web init failed", error);
          const pre = document.createElement("pre");
          pre.textContent = String(error);
          pre.style.color = "#f4f8ff";
          pre.style.padding = "24px";
          document.body.appendChild(pre);
        }});
    </script>
  </body>
</html>
"##,
        title
    )
}

fn rust_raw_literal(value: &str) -> String {
    format!("r#\"{}\"#", value)
}

fn canonical_display(path: PathBuf) -> String {
    fs::canonicalize(&path)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn find_wasm_bindgen() -> Option<String> {
    let candidates = ["wasm-bindgen", "wasm-bindgen-cli"];
    for candidate in candidates {
        let Ok(status) = Command::new(candidate).arg("--version").status() else {
            continue;
        };
        if status.success() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn ensure_web_prerequisites() -> Result<()> {
    ensure_command_available(
        "cargo",
        "Cargo is required to build for the web. Install Rust from https://www.rust-lang.org/tools/install",
    )?;
    ensure_command_available(
        "rustup",
        "Rustup is required for web export setup. Install Rust from https://www.rust-lang.org/tools/install",
    )?;

    if !has_rust_target("wasm32-unknown-unknown")? {
        println!("Missing Rust target `wasm32-unknown-unknown`.");
        println!("Running: rustup target add wasm32-unknown-unknown");
        let status = Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .status()
            .context("failed to run `rustup target add wasm32-unknown-unknown`")?;
        if !status.success() {
            bail!(
                "failed to install Rust target `wasm32-unknown-unknown`; run `rustup target add wasm32-unknown-unknown` manually"
            );
        }
    }

    if find_wasm_bindgen().is_none() {
        bail!(
            "wasm-bindgen CLI is required for web export.\nInstall it with:\n  cargo install wasm-bindgen-cli"
        );
    }

    Ok(())
}

fn ensure_xcode_export_prerequisites() -> Result<()> {
    if env::consts::OS != "macos" {
        bail!(
            "Xcode export currently requires macOS. Run this command on a Mac with Xcode installed."
        );
    }

    ensure_command_available(
        "xcodebuild",
        "Xcode export requires Xcode and the command line tools. Install Xcode from the App Store, then run `xcode-select --install` if needed.",
    )?;

    ensure_command_available(
        "cargo",
        "Cargo is required to build the generated Rust bridge. Install Rust from https://www.rust-lang.org/tools/install",
    )?;

    Ok(())
}

fn ensure_command_available(command: &str, help: &str) -> Result<()> {
    let status = Command::new(command).arg("--version").status();
    match status {
        Ok(status) if status.success() => Ok(()),
        _ => bail!("{help}"),
    }
}

fn has_rust_target(target: &str) -> Result<bool> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("failed to query installed Rust targets via rustup")?;
    if !output.status.success() {
        bail!("failed to query installed Rust targets via rustup");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().any(|line| line.trim() == target))
}

fn content_type_for(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => Some("text/html; charset=utf-8"),
        Some("js") => Some("text/javascript; charset=utf-8"),
        Some("wasm") => Some("application/wasm"),
        Some("css") => Some("text/css; charset=utf-8"),
        Some("json") => Some("application/json"),
        Some("png") => Some("image/png"),
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("ttf") => Some("font/ttf"),
        _ => None,
    }
}

fn export_terrain_debug_images(
    compiled: &rpu_core::CompiledProject,
    project_root: &Path,
    build_dir: &Path,
) -> Result<()> {
    let output_dir = build_dir.join("debug/maps");
    let mut wrote_any = false;

    for document in &compiled.parsed_scenes {
        let document_stem = document
            .path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("scene");

        for scene in &document.scenes {
            for map in &scene.maps {
                let classified = map.classify_terrain();
                if classified.cells.is_empty() {
                    continue;
                }

                fs::create_dir_all(&output_dir)
                    .with_context(|| format!("failed to create {}", output_dir.display()))?;
                let filename = format!(
                    "{}__{}__{}.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let path = output_dir.join(filename);
                write_terrain_debug_png(&classified, &path)?;
                let region_filename = format!(
                    "{}__{}__{}__regions.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let region_path = output_dir.join(region_filename);
                write_terrain_regions_png(&classified, &region_path)?;
                let tangent_filename = format!(
                    "{}__{}__{}__tangents.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let tangent_path = output_dir.join(tangent_filename);
                write_terrain_tangents_png(&classified, &tangent_path)?;
                let material_filename = format!(
                    "{}__{}__{}__materials.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let material_path = output_dir.join(material_filename);
                write_terrain_materials_png(&classified, &material_path)?;
                let synth_filename = format!(
                    "{}__{}__{}__synth.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let synth_path = output_dir.join(synth_filename);
                write_terrain_synth_png(project_root, &classified, &synth_path)?;
                let synth_layers_filename = format!(
                    "{}__{}__{}__synth_layers.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let synth_layers_path = output_dir.join(synth_layers_filename);
                write_terrain_synth_layers_png(project_root, &classified, &synth_layers_path)?;
                let transition_filename = format!(
                    "{}__{}__{}__transitions.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let transition_path = output_dir.join(transition_filename);
                write_terrain_transitions_png(&classified, &transition_path)?;
                let band_filename = format!(
                    "{}__{}__{}__bands.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let band_path = output_dir.join(band_filename);
                write_terrain_bands_png(&classified, &band_path)?;
                let loop_filename = format!(
                    "{}__{}__{}__loops.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let loop_path = output_dir.join(loop_filename);
                write_terrain_loops_png(&classified, &loop_path)?;
                let contour_filename = format!(
                    "{}__{}__{}__contours.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let contour_path = output_dir.join(contour_filename);
                write_terrain_contours_png(&classified, &contour_path)?;
                let influence_filename = format!(
                    "{}__{}__{}__influences.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let influence_path = output_dir.join(influence_filename);
                write_terrain_influences_png(&classified, &influence_path)?;
                let heightfield_filename = format!(
                    "{}__{}__{}__heightfield.png",
                    sanitize_debug_name(document_stem),
                    sanitize_debug_name(&scene.name),
                    sanitize_debug_name(&map.name)
                );
                let heightfield_path = output_dir.join(heightfield_filename);
                write_terrain_heightfield_png(&classified, &heightfield_path)?;
                wrote_any = true;
            }
        }
    }

    if wrote_any {
        fs::write(output_dir.join("README.txt"), terrain_debug_readme())
            .with_context(|| format!("failed to write {}", output_dir.join("README.txt").display()))?;
        println!("Wrote terrain debug images to {}", output_dir.display());
    }

    Ok(())
}

fn write_terrain_debug_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));

    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(&mut image, x, y, tile, tile, terrain_shape_rgba(cell.shape));
        draw_shape_accent(&mut image, x, y, tile, cell.shape, material_accent_rgba(&cell.material));
        draw_style_marker(&mut image, x, y, tile, cell.style);
        draw_exposed_sides(&mut image, x, y, tile, cell.exposed);
        draw_normal_marker(&mut image, x, y, tile, cell.normal);
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_regions_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height =
        border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));

    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for region in &classified.regions {
        let fill = region_color_rgba(region.id);
        let border_color = region_outline_rgba(region.id);
        for &(row, col) in &region.cells {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, fill);
            draw_region_outline(&mut image, x, y, tile, border_color);
        }
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_tangents_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(&mut image, x, y, tile, tile, terrain_shape_rgba(cell.shape));
        draw_shape_accent(&mut image, x, y, tile, cell.shape, material_accent_rgba(&cell.material));
        draw_tangent_marker(&mut image, x, y, tile, cell.tangent);
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_materials_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(&mut image, x, y, tile, tile, material_fill_rgba(&cell.material));
        draw_shape_accent(&mut image, x, y, tile, cell.shape, rgba([20, 24, 30, 255]));
        draw_band_distance_label(&mut image, x, y, tile, cell.boundary_distance);
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_synth_png(
    project_root: &Path,
    classified: &rpu_core::ClassifiedAsciiMap,
    path: &Path,
) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let material_fields = build_material_fields(project_root, classified);
    let region_lookup: std::collections::HashMap<usize, &rpu_core::TerrainRegion> =
        classified.regions.iter().map(|region| (region.id, region)).collect();
    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([10, 12, 16, 255]));
    for cell in &classified.cells {
        let Some(region) = region_lookup.get(&cell.region_id).copied() else {
            continue;
        };
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        for py in 0..tile {
            for px in 0..tile {
                let color = synthesize_terrain_pixel(&material_fields, cell, region, px, py, tile);
                image.put_pixel(x + px, y + py, color);
            }
        }
        draw_exposed_sides(&mut image, x, y, tile, cell.exposed);
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_synth_layers_png(
    project_root: &Path,
    classified: &rpu_core::ClassifiedAsciiMap,
    path: &Path,
) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width =
        border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2
        + classified.height as u32 * tile
        + classified.height.saturating_sub(1) as u32 * gap;

    let material_fields = build_material_fields(project_root, classified);
    let region_lookup: std::collections::HashMap<usize, &rpu_core::TerrainRegion> =
        classified.regions.iter().map(|region| (region.id, region)).collect();
    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([10, 12, 16, 255]));
    for cell in &classified.cells {
        let Some(region) = region_lookup.get(&cell.region_id).copied() else {
            continue;
        };
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        for py in 0..tile {
            for px in 0..tile {
                let (winner, differs) =
                    synthesize_terrain_pixel_layers(&material_fields, cell, region, px, py, tile);
                let mut color = material_fill_rgba(winner);
                if differs {
                    color = lighten_rgba(color, 38);
                }
                image.put_pixel(x + px, y + py, color);
            }
        }
        draw_exposed_sides(&mut image, x, y, tile, cell.exposed);
        fill_rect(&mut image, x, y, tile, 5, material_fill_rgba(&cell.material));
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_transitions_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let region_loop_lengths: std::collections::HashMap<usize, usize> = classified
        .regions
        .iter()
        .map(|region| (region.id, region.boundary_loop.len()))
        .collect();

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(&mut image, x, y, tile, tile, material_fill_rgba(&cell.material));
        let loop_len = *region_loop_lengths.get(&cell.region_id).unwrap_or(&1);
        let stripe = surface_u_rgba(cell.surface_u, loop_len);
        fill_rect(&mut image, x, y, tile, 6, stripe);
        draw_band_distance_label(&mut image, x, y, tile, cell.boundary_distance);
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_bands_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(&mut image, x, y, tile, tile, terrain_band_rgba(cell.depth_band));
        draw_band_distance_label(&mut image, x, y, tile, cell.boundary_distance);
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_loops_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for row in 0..classified.height {
        for col in 0..classified.width {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, rgba([26, 32, 40, 255]));
        }
    }

    for region in &classified.regions {
        let fill = region_color_rgba(region.id);
        let outline = region_outline_rgba(region.id);
        let path_color = rgba([250, 250, 250, 235]);
        for &(row, col) in &region.cells {
            let x = border + col as u32 * (tile + gap);
            let y = border + row as u32 * (tile + gap);
            fill_rect(&mut image, x, y, tile, tile, fill);
            draw_region_outline(&mut image, x, y, tile, outline);
        }
        for window in region.boundary_loop.windows(2) {
            let (row0, col0) = window[0];
            let (row1, col1) = window[1];
            let x0 = border as i32 + col0 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            let y0 = border as i32 + row0 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            let x1 = border as i32 + col1 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            let y1 = border as i32 + row1 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            draw_line(&mut image, x0, y0, x1, y1, path_color);
        }
        if region.boundary_loop.len() > 1 {
            let (row0, col0) = region.boundary_loop[0];
            let (row1, col1) = region.boundary_loop[region.boundary_loop.len() - 1];
            let x0 = border as i32 + col0 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            let y0 = border as i32 + row0 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            let x1 = border as i32 + col1 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            let y1 = border as i32 + row1 as i32 * (tile + gap) as i32 + tile as i32 / 2;
            draw_line(&mut image, x0, y0, x1, y1, path_color);
        }
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_contours_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(&mut image, x, y, tile, tile, contour_fill_rgba(cell.contour));
        draw_terrain_contour(&mut image, x, y, tile, cell.contour, rgba([255, 255, 255, 255]));
        draw_region_outline(&mut image, x, y, tile, rgba([38, 44, 56, 255]));
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_influences_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        fill_rect(
            &mut image,
            x,
            y,
            tile,
            tile,
            transition_role_rgba(cell.transition_role, cell.transition_strength),
        );
        draw_terrain_contour(&mut image, x, y, tile, cell.contour, rgba([255, 255, 255, 255]));
        draw_region_outline(&mut image, x, y, tile, rgba([38, 44, 56, 255]));
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_terrain_heightfield_png(classified: &rpu_core::ClassifiedAsciiMap, path: &Path) -> Result<()> {
    let tile = 40u32;
    let gap = 2u32;
    let border = 12u32;
    let width = border * 2 + classified.width as u32 * tile + classified.width.saturating_sub(1) as u32 * gap;
    let height = border * 2 + classified.height as u32 * tile + classified.height.saturating_sub(1) as u32 * gap;

    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));
    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        for py in 0..tile {
            for px in 0..tile {
                let surface_y = surface_height_for_cell(cell, px, tile);
                let shade = ((surface_y as f32 / tile.max(1) as f32) * 190.0 + 30.0)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                image.put_pixel(x + px, y + py, rgba([shade, shade, shade, 255]));
            }
        }
        draw_surface_profile(&mut image, x, y, tile, cell, rgba([255, 255, 255, 255]));
        draw_region_outline(&mut image, x, y, tile, rgba([38, 44, 56, 255]));
    }

    image
        .save(path)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn fill_rect(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Rgba<u8>,
) {
    for py in y..(y + height).min(image.height()) {
        for px in x..(x + width).min(image.width()) {
            image.put_pixel(px, py, color);
        }
    }
}

fn draw_shape_accent(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    shape: rpu_core::TerrainShape,
    color: Rgba<u8>,
) {
    let thickness = 4u32;
    let inset = 6u32;
    let inner = tile.saturating_sub(inset * 2);
    match shape {
        rpu_core::TerrainShape::Top => fill_rect(image, x + inset, y + inset, inner, thickness, color),
        rpu_core::TerrainShape::Bottom => {
            fill_rect(image, x + inset, y + tile - inset - thickness, inner, thickness, color)
        }
        rpu_core::TerrainShape::Left => fill_rect(image, x + inset, y + inset, thickness, inner, color),
        rpu_core::TerrainShape::Right => {
            fill_rect(image, x + tile - inset - thickness, y + inset, thickness, inner, color)
        }
        rpu_core::TerrainShape::TopLeftOuter => {
            fill_rect(image, x + inset, y + inset, inner / 2, thickness, color);
            fill_rect(image, x + inset, y + inset, thickness, inner / 2, color);
        }
        rpu_core::TerrainShape::TopRightOuter => {
            fill_rect(image, x + tile / 2, y + inset, inner / 2, thickness, color);
            fill_rect(image, x + tile - inset - thickness, y + inset, thickness, inner / 2, color);
        }
        rpu_core::TerrainShape::BottomLeftOuter => {
            fill_rect(image, x + inset, y + tile - inset - thickness, inner / 2, thickness, color);
            fill_rect(image, x + inset, y + tile / 2, thickness, inner / 2, color);
        }
        rpu_core::TerrainShape::BottomRightOuter => {
            fill_rect(
                image,
                x + tile / 2,
                y + tile - inset - thickness,
                inner / 2,
                thickness,
                color,
            );
            fill_rect(
                image,
                x + tile - inset - thickness,
                y + tile / 2,
                thickness,
                inner / 2,
                color,
            );
        }
        rpu_core::TerrainShape::TopLeftInner
        | rpu_core::TerrainShape::TopRightInner
        | rpu_core::TerrainShape::BottomLeftInner
        | rpu_core::TerrainShape::BottomRightInner
        | rpu_core::TerrainShape::Interior => {
            fill_rect(image, x + inset, y + inset, inner, inner, color);
        }
        rpu_core::TerrainShape::Isolated => {
            fill_rect(image, x + tile / 2 - 4, y + tile / 2 - 4, 8, 8, color);
        }
        rpu_core::TerrainShape::Empty => {}
    }
}

fn draw_normal_marker(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    normal: rpu_core::TerrainNormal,
) {
    let color = rgba([250, 250, 250, 235]);
    let shaft = (tile / 3).max(6) as i32;
    let tip = (tile / 10).max(2) as i32;
    let cx = x as i32 + tile as i32 / 2;
    let cy = y as i32 + tile as i32 / 2;

    let (dx, dy) = match normal {
        rpu_core::TerrainNormal::None => return,
        rpu_core::TerrainNormal::Up => (0, -1),
        rpu_core::TerrainNormal::Down => (0, 1),
        rpu_core::TerrainNormal::Left => (-1, 0),
        rpu_core::TerrainNormal::Right => (1, 0),
        rpu_core::TerrainNormal::UpLeft => (-1, -1),
        rpu_core::TerrainNormal::UpRight => (1, -1),
        rpu_core::TerrainNormal::DownLeft => (-1, 1),
        rpu_core::TerrainNormal::DownRight => (1, 1),
    };

    let ex = cx + dx * shaft;
    let ey = cy + dy * shaft;
    draw_line(image, cx, cy, ex, ey, color);

    if dx == 0 {
        draw_line(image, ex, ey, ex - tip, ey - dy * tip, color);
        draw_line(image, ex, ey, ex + tip, ey - dy * tip, color);
    } else if dy == 0 {
        draw_line(image, ex, ey, ex - dx * tip, ey - tip, color);
        draw_line(image, ex, ey, ex - dx * tip, ey + tip, color);
    } else {
        draw_line(image, ex, ey, ex - dx * tip, ey, color);
        draw_line(image, ex, ey, ex, ey - dy * tip, color);
    }
}

fn draw_tangent_marker(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    tangent: rpu_core::TerrainTangent,
) {
    let color = rgba([118, 231, 255, 235]);
    let shaft = (tile / 3).max(6) as i32;
    let tip = (tile / 10).max(2) as i32;
    let cx = x as i32 + tile as i32 / 2;
    let cy = y as i32 + tile as i32 / 2;

    let (dx, dy) = match tangent {
        rpu_core::TerrainTangent::None => return,
        rpu_core::TerrainTangent::Up => (0, -1),
        rpu_core::TerrainTangent::Down => (0, 1),
        rpu_core::TerrainTangent::Left => (-1, 0),
        rpu_core::TerrainTangent::Right => (1, 0),
        rpu_core::TerrainTangent::UpLeft => (-1, -1),
        rpu_core::TerrainTangent::UpRight => (1, -1),
        rpu_core::TerrainTangent::DownLeft => (-1, 1),
        rpu_core::TerrainTangent::DownRight => (1, 1),
    };

    let ex = cx + dx * shaft;
    let ey = cy + dy * shaft;
    draw_line(image, cx, cy, ex, ey, color);

    if dx == 0 {
        draw_line(image, ex, ey, ex - tip, ey - dy * tip, color);
        draw_line(image, ex, ey, ex + tip, ey - dy * tip, color);
    } else if dy == 0 {
        draw_line(image, ex, ey, ex - dx * tip, ey - tip, color);
        draw_line(image, ex, ey, ex - dx * tip, ey + tip, color);
    } else {
        draw_line(image, ex, ey, ex - dx * tip, ey, color);
        draw_line(image, ex, ey, ex, ey - dy * tip, color);
    }
}

fn draw_terrain_contour(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    contour: rpu_core::TerrainContour,
    color: Rgba<u8>,
) {
    let inset = (tile / 10).max(2) as i32;
    let x0 = x as i32 + inset;
    let x1 = x as i32 + tile as i32 - inset - 1;
    let y0 = y as i32 + inset;
    let y1 = y as i32 + tile as i32 - inset - 1;
    let cx = x as i32 + tile as i32 / 2;
    let cy = y as i32 + tile as i32 / 2;

    match contour {
        rpu_core::TerrainContour::None => {}
        rpu_core::TerrainContour::FlatTop => {
            draw_line(image, x0, y0, x1, y0, color);
        }
        rpu_core::TerrainContour::RampUpRight => {
            draw_line(image, x0, y1, x1, y0, color);
        }
        rpu_core::TerrainContour::RampUpLeft => {
            draw_line(image, x0, y0, x1, y1, color);
        }
        rpu_core::TerrainContour::CapLeft => {
            draw_line(image, x0, cy, x0 + (cx - x0) / 2, y0 + (cy - y0) / 2, color);
            draw_line(image, x0 + (cx - x0) / 2, y0 + (cy - y0) / 2, cx, y0, color);
        }
        rpu_core::TerrainContour::CapRight => {
            draw_line(image, cx, y0, x1 - (x1 - cx) / 2, y0 + (cy - y0) / 2, color);
            draw_line(image, x1 - (x1 - cx) / 2, y0 + (cy - y0) / 2, x1, cy, color);
        }
    }
}

fn draw_surface_profile(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    cell: &rpu_core::ClassifiedMapCell,
    color: Rgba<u8>,
) {
    if tile <= 1 {
        return;
    }
    for px in 0..tile.saturating_sub(1) {
        let y0 = y as i32 + surface_height_for_cell(cell, px, tile) as i32;
        let y1 = y as i32 + surface_height_for_cell(cell, px + 1, tile) as i32;
        draw_line(
            image,
            x as i32 + px as i32,
            y0,
            x as i32 + px as i32 + 1,
            y1,
            color,
        );
    }
}

fn draw_line(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: Rgba<u8>,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u32) < image.width() && (y0 as u32) < image.height() {
            image.put_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_region_outline(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    color: Rgba<u8>,
) {
    let thickness = 3u32;
    fill_rect(image, x, y, tile, thickness, color);
    fill_rect(image, x, y + tile.saturating_sub(thickness), tile, thickness, color);
    fill_rect(image, x, y, thickness, tile, color);
    fill_rect(image, x + tile.saturating_sub(thickness), y, thickness, tile, color);
}

fn draw_exposed_sides(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    exposed: rpu_core::TerrainExposedSides,
) {
    let thickness = 5u32;
    let color = rgba([250, 244, 214, 255]);
    if exposed.top {
        fill_rect(image, x, y, tile, thickness, color);
    }
    if exposed.bottom {
        fill_rect(image, x, y + tile.saturating_sub(thickness), tile, thickness, color);
    }
    if exposed.left {
        fill_rect(image, x, y, thickness, tile, color);
    }
    if exposed.right {
        fill_rect(image, x + tile.saturating_sub(thickness), y, thickness, tile, color);
    }
}

fn draw_style_marker(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    style: rpu_core::TerrainEdgeStyle,
) {
    let color = rgba([16, 18, 22, 255]);
    match style {
        rpu_core::TerrainEdgeStyle::Square => {
            fill_rect(image, x + tile / 2 - 4, y + tile / 2 - 4, 8, 8, color);
        }
        rpu_core::TerrainEdgeStyle::Round => {
            fill_rect(image, x + tile / 2 - 5, y + tile / 2 - 2, 10, 4, color);
            fill_rect(image, x + tile / 2 - 2, y + tile / 2 - 5, 4, 10, color);
        }
        rpu_core::TerrainEdgeStyle::Diagonal => {
            for step in 0..10 {
                let px = x + tile / 2 - 5 + step;
                let py = y + tile / 2 + 4 - step;
                fill_rect(image, px, py, 2, 2, color);
            }
        }
    }
}

fn terrain_shape_rgba(shape: rpu_core::TerrainShape) -> Rgba<u8> {
    let [r, g, b, a] = match shape {
        rpu_core::TerrainShape::Empty => [0, 0, 0, 0],
        rpu_core::TerrainShape::Isolated => [242, 82, 82, 255],
        rpu_core::TerrainShape::Interior => [36, 66, 189, 255],
        rpu_core::TerrainShape::Top => [89, 224, 107, 255],
        rpu_core::TerrainShape::Bottom => [184, 84, 214, 255],
        rpu_core::TerrainShape::Left => [250, 184, 66, 255],
        rpu_core::TerrainShape::Right => [250, 143, 46, 255],
        rpu_core::TerrainShape::TopLeftOuter => [66, 235, 235, 255],
        rpu_core::TerrainShape::TopRightOuter => [48, 209, 250, 255],
        rpu_core::TerrainShape::BottomLeftOuter => [224, 105, 207, 255],
        rpu_core::TerrainShape::BottomRightOuter => [191, 84, 242, 255],
        rpu_core::TerrainShape::TopLeftInner => [158, 240, 158, 255],
        rpu_core::TerrainShape::TopRightInner => [140, 224, 140, 255],
        rpu_core::TerrainShape::BottomLeftInner => [237, 148, 148, 255],
        rpu_core::TerrainShape::BottomRightInner => [219, 125, 125, 255],
    };
    rgba([r, g, b, a])
}

fn terrain_band_rgba(band: rpu_core::TerrainDepthBand) -> Rgba<u8> {
    match band {
        rpu_core::TerrainDepthBand::Edge => rgba([245, 122, 122, 255]),
        rpu_core::TerrainDepthBand::NearSurface => rgba([245, 196, 110, 255]),
        rpu_core::TerrainDepthBand::Interior => rgba([100, 188, 255, 255]),
        rpu_core::TerrainDepthBand::DeepInterior => rgba([74, 88, 214, 255]),
    }
}

fn contour_fill_rgba(contour: rpu_core::TerrainContour) -> Rgba<u8> {
    match contour {
        rpu_core::TerrainContour::None => rgba([34, 40, 50, 255]),
        rpu_core::TerrainContour::FlatTop => rgba([70, 120, 86, 255]),
        rpu_core::TerrainContour::RampUpRight | rpu_core::TerrainContour::RampUpLeft => {
            rgba([92, 92, 128, 255])
        }
        rpu_core::TerrainContour::CapLeft | rpu_core::TerrainContour::CapRight => {
            rgba([90, 112, 132, 255])
        }
    }
}

fn transition_role_rgba(role: rpu_core::TerrainTransitionRole, strength: u8) -> Rgba<u8> {
    let base = match role {
        rpu_core::TerrainTransitionRole::None => rgba([34, 40, 50, 255]),
        rpu_core::TerrainTransitionRole::RampUpRight => rgba([92, 92, 128, 255]),
        rpu_core::TerrainTransitionRole::RampUpLeft => rgba([92, 92, 128, 255]),
        rpu_core::TerrainTransitionRole::JoinFromLeft => rgba([98, 132, 86, 255]),
        rpu_core::TerrainTransitionRole::JoinFromRight => rgba([98, 132, 86, 255]),
        rpu_core::TerrainTransitionRole::JoinBoth => rgba([128, 142, 88, 255]),
    };
    if strength == 0 {
        base
    } else {
        let amount = (strength / 6).max(8);
        lighten_rgba(base, amount)
    }
}

fn draw_band_distance_label(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    tile: u32,
    distance: u32,
) {
    let color = rgba([20, 24, 30, 255]);
    let bar_width = (distance.min(6) + 1) * 4;
    let width = bar_width.min(tile.saturating_sub(8));
    fill_rect(image, x + 4, y + tile.saturating_sub(8), width, 4, color);
}

fn material_accent_rgba(material: &str) -> Rgba<u8> {
    let mut hash = 0u32;
    for byte in material.bytes() {
        hash = hash.wrapping_mul(16777619) ^ byte as u32;
    }
    let r = 80 + (hash & 0x7f) as u8;
    let g = 80 + ((hash >> 7) & 0x7f) as u8;
    let b = 80 + ((hash >> 14) & 0x7f) as u8;
    rgba([r, g, b, 255])
}

fn material_fill_rgba(material: &str) -> Rgba<u8> {
    match material {
        "grass" => rgba([106, 214, 116, 255]),
        "dirt" => rgba([176, 122, 78, 255]),
        "rock" => rgba([114, 124, 144, 255]),
        _ => material_accent_rgba(material),
    }
}

fn synthesize_terrain_pixel(
    material_fields: &std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    cell: &rpu_core::ClassifiedMapCell,
    region: &rpu_core::TerrainRegion,
    px: u32,
    py: u32,
    tile: u32,
) -> Rgba<u8> {
    let (u, v) = region_space_projection_for_cell(cell, region, px, py, tile);
    let surface_y = surface_height_for_cell(cell, px, tile);
    if py < surface_y {
        return rgba([0, 0, 0, 0]);
    }
    let local_inward = py - surface_y;
    let cap_base = (tile / 2).max(12);
    let cap_variation = (cell.surface_u % 5) as i32 - 2;
    let cap_depth = (cap_base as i32 + cap_variation).max(8) as u32;
    let top_material = top_material_for_stack(&cell.material_key);
    let is_surface_cap_cell = cell.material == top_material
        && matches!(
            cell.normal,
            rpu_core::TerrainNormal::Up
                | rpu_core::TerrainNormal::UpLeft
                | rpu_core::TerrainNormal::UpRight
        );
    let body_material = body_material_for_cell(cell);
    let body = sample_material_field(material_fields, body_material, u, v);
    if is_surface_cap_cell && local_inward < cap_depth {
        let source_h = material_fields
            .get(top_material)
            .map(|image| image.height().max(1))
            .unwrap_or(16);
        let sampled_h = if top_material == "grass" {
            (source_h / 2).max(1)
        } else {
            source_h
        };
        let sampled_offset = if top_material == "grass" && source_h > sampled_h {
            source_h - sampled_h
        } else {
            0
        };
        let cap_v = if cap_depth <= 1 {
            0
        } else {
            sampled_offset
                + (local_inward.saturating_mul(sampled_h.saturating_sub(1)))
                    / cap_depth.saturating_sub(1)
        };
        let top = sample_material_field(material_fields, top_material, u, cap_v);
        return alpha_over(top, body);
    }
    body
}

fn synthesize_terrain_pixel_layers<'a>(
    material_fields: &'a std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    cell: &'a rpu_core::ClassifiedMapCell,
    region: &rpu_core::TerrainRegion,
    px: u32,
    py: u32,
    tile: u32,
) -> (&'a str, bool) {
    let (u, v) = region_space_projection_for_cell(cell, region, px, py, tile);
    sample_material_stack_layers(material_fields, cell, u, v)
}

fn build_material_fields(
    project_root: &Path,
    classified: &rpu_core::ClassifiedAsciiMap,
) -> std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut fields = std::collections::HashMap::new();
    let materials: std::collections::HashSet<_> = classified
        .cells
        .iter()
        .flat_map(|cell| {
            cell.material_key
                .split('>')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
        })
        .collect();

    for material in materials {
        let image = load_material_source(project_root, material)
            .unwrap_or_else(|| builtin_material_image(material));
        fields.insert(material.to_string(), image);
    }

    fields
}

#[allow(dead_code)]
fn build_stack_fields(
    project_root: &Path,
    classified: &rpu_core::ClassifiedAsciiMap,
) -> std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut fields = std::collections::HashMap::new();
    let stacks: std::collections::HashSet<_> = classified
        .cells
        .iter()
        .map(|cell| cell.material_key.clone())
        .collect();

    for stack in stacks {
        let source = build_stack_source(project_root, &stack);
        let field = wfc_material_field(&stack, &source).unwrap_or_else(|| quilt_material_field(&stack, &source));
        fields.insert(stack, field);
    }

    fields
}

#[allow(dead_code)]
fn build_stack_source(project_root: &Path, stack_key: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let materials = stack_key
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if materials.is_empty() {
        return builtin_material_image("rock");
    }

    let sources = materials
        .iter()
        .map(|material| {
            load_material_source(project_root, material)
                .unwrap_or_else(|| builtin_material_image(material))
        })
        .collect::<Vec<_>>();

    let width = sources.iter().map(|img| img.width()).max().unwrap_or(16).max(1);
    let band_height = sources.iter().map(|img| img.height()).max().unwrap_or(16).max(1);
    let height = band_height * materials.len() as u32;
    let mut image = ImageBuffer::from_pixel(width, height.max(1), rgba([0, 0, 0, 0]));

    for (index, source) in sources.iter().enumerate() {
        let y_offset = index as u32 * band_height;
        for y in 0..band_height {
            for x in 0..width {
                let sample = *source.get_pixel(x % source.width().max(1), y % source.height().max(1));
                image.put_pixel(x, y_offset + y, sample);
            }
        }
    }

    image
}

fn load_material_source(
    project_root: &Path,
    material: &str,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let candidates = [
        project_root.join("assets").join(format!("{material}.png")),
        project_root.join("assets").join("terrain").join(format!("{material}.png")),
        project_root
            .join("assets")
            .join(format!("terrain_{material}.png")),
    ];

    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        if let Ok(image) = image::open(&candidate) {
            return Some(image.to_rgba8());
        }
    }

    None
}

#[allow(dead_code)]
fn sample_material_stack(
    material_fields: &std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    cell: &rpu_core::ClassifiedMapCell,
    u: u32,
    v: u32,
) -> Rgba<u8> {
    let stack = cell
        .material_key
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if stack.is_empty() {
        return rgba([0, 0, 0, 255]);
    }
    let start_index = stack
        .iter()
        .position(|material| *material == cell.material)
        .unwrap_or(0);
    let mut out = rgba([0, 0, 0, 0]);
    for material in stack[start_index..].iter().rev() {
        let sample = sample_material_field(material_fields, material, u, v);
        out = alpha_over(sample, out);
    }
    out
}

fn sample_material_stack_layers<'a>(
    material_fields: &'a std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    cell: &'a rpu_core::ClassifiedMapCell,
    u: u32,
    v: u32,
) -> (&'a str, bool) {
    let stack = cell
        .material_key
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if stack.is_empty() {
        return ("", false);
    }
    let start_index = stack
        .iter()
        .position(|material| *material == cell.material)
        .unwrap_or(0);
    let mut winner = cell.material.as_str();
    for material in stack[start_index..].iter() {
        let sample = sample_material_field(material_fields, material, u, v);
        if sample[3] > 0 {
            winner = material;
            break;
        }
    }
    (winner, winner != cell.material)
}

fn sample_material_field(
    material_fields: &std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    material: &str,
    u: u32,
    v: u32,
) -> Rgba<u8> {
    if let Some(image) = material_fields.get(material) {
        let x = u % image.width().max(1);
        let y = v % image.height().max(1);
        return *image.get_pixel(x, y);
    }
    sample_material_exemplar(material, u, v)
}

#[allow(dead_code)]
fn sample_stack_field(field: &ImageBuffer<Rgba<u8>, Vec<u8>>, u: u32, v: u32) -> Rgba<u8> {
    let x = u % field.width().max(1);
    let y = v % field.height().max(1);
    *field.get_pixel(x, y)
}

fn top_material_for_stack(stack_key: &str) -> &str {
    stack_key
        .split('>')
        .map(str::trim)
        .find(|part| !part.is_empty())
        .unwrap_or("rock")
}

fn body_material_for_cell<'a>(
    cell: &'a rpu_core::ClassifiedMapCell,
) -> &'a str {
    let stack = cell
        .material_key
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if stack.is_empty() {
        return "rock";
    }
    if stack.len() == 1 {
        return stack[0];
    }
    let top = stack[0];
    if cell.material == top {
        stack.get(1).copied().unwrap_or(top)
    } else {
        cell.material.as_str()
    }
}

#[allow(dead_code)]
fn alpha_over(top: Rgba<u8>, bottom: Rgba<u8>) -> Rgba<u8> {
    let ta = top[3] as f32 / 255.0;
    let ba = bottom[3] as f32 / 255.0;
    let out_a = ta + ba * (1.0 - ta);
    if out_a <= f32::EPSILON {
        return rgba([0, 0, 0, 0]);
    }
    let blend = |tc: u8, bc: u8| -> u8 {
        (((tc as f32 / 255.0) * ta + (bc as f32 / 255.0) * ba * (1.0 - ta)) / out_a * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    rgba([
        blend(top[0], bottom[0]),
        blend(top[1], bottom[1]),
        blend(top[2], bottom[2]),
        (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
    ])
}

fn lighten_rgba(color: Rgba<u8>, amount: u8) -> Rgba<u8> {
    rgba([
        color[0].saturating_add(amount),
        color[1].saturating_add(amount),
        color[2].saturating_add(amount),
        color[3],
    ])
}

#[allow(dead_code)]
fn lerp_rgba(a: Rgba<u8>, b: Rgba<u8>, t: u8) -> Rgba<u8> {
    let tf = t as f32 / 255.0;
    let blend = |av: u8, bv: u8| -> u8 {
        ((av as f32) * (1.0 - tf) + (bv as f32) * tf)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    rgba([
        blend(a[0], b[0]),
        blend(a[1], b[1]),
        blend(a[2], b[2]),
        blend(a[3], b[3]),
    ])
}

fn surface_height_for_cell(cell: &rpu_core::ClassifiedMapCell, px: u32, tile: u32) -> u32 {
    let max = tile.saturating_sub(1).max(1);
    let x = px.min(max);
    let flat = 0u32;
    let ramp = match cell.contour {
        rpu_core::TerrainContour::RampUpRight => max.saturating_sub(x),
        rpu_core::TerrainContour::RampUpLeft => x,
        rpu_core::TerrainContour::CapLeft => x / 2,
        rpu_core::TerrainContour::CapRight => max.saturating_sub(x) / 2,
        rpu_core::TerrainContour::FlatTop | rpu_core::TerrainContour::None => 0,
    };

    match cell.transition_role {
        rpu_core::TerrainTransitionRole::RampUpRight | rpu_core::TerrainTransitionRole::RampUpLeft => ramp,
        rpu_core::TerrainTransitionRole::JoinFromLeft => {
            let strength = cell.transition_strength as u32;
            let edge = max.saturating_sub(x);
            (strength * edge * max / (255 * max.max(1))).min(max)
        }
        rpu_core::TerrainTransitionRole::JoinFromRight => {
            let strength = cell.transition_strength as u32;
            let edge = x;
            (strength * edge * max / (255 * max.max(1))).min(max)
        }
        rpu_core::TerrainTransitionRole::JoinBoth => {
            let strength = cell.transition_strength as u32;
            let edge = x.min(max.saturating_sub(x));
            (strength * (max.saturating_sub(edge)) * (max / 2).max(1) / (255 * max.max(1)))
                .min(max / 2)
        }
        rpu_core::TerrainTransitionRole::None => flat,
    }
}

#[allow(dead_code)]
fn inward_from_heightfield(cell: &rpu_core::ClassifiedMapCell, px: u32, py: u32, tile: u32) -> u32 {
    let surface_y = surface_height_for_cell(cell, px, tile);
    py.saturating_sub(surface_y)
}

#[allow(dead_code)]
fn along_surface_projection(
    tangent: rpu_core::TerrainTangent,
    px: u32,
    py: u32,
    tile: u32,
) -> u32 {
    match tangent {
        rpu_core::TerrainTangent::None => px,
        rpu_core::TerrainTangent::Right => px,
        rpu_core::TerrainTangent::Left => tile.saturating_sub(1).saturating_sub(px),
        rpu_core::TerrainTangent::Down => py,
        rpu_core::TerrainTangent::Up => tile.saturating_sub(1).saturating_sub(py),
        rpu_core::TerrainTangent::UpLeft => (tile.saturating_sub(1).saturating_sub(px) + tile.saturating_sub(1).saturating_sub(py)) / 2,
        rpu_core::TerrainTangent::UpRight => (px + tile.saturating_sub(1).saturating_sub(py)) / 2,
        rpu_core::TerrainTangent::DownLeft => (tile.saturating_sub(1).saturating_sub(px) + py) / 2,
        rpu_core::TerrainTangent::DownRight => (px + py) / 2,
    }
}

fn region_space_projection_for_cell(
    cell: &rpu_core::ClassifiedMapCell,
    region: &rpu_core::TerrainRegion,
    px: u32,
    py: u32,
    tile: u32,
) -> (u32, u32) {
    let region_x = (cell.col.saturating_sub(region.min_col) as u32) * tile + px;
    let local_inward = py.saturating_sub(surface_height_for_cell(cell, px, tile));
    let inward = cell.boundary_distance * tile + local_inward;
    (region_x, inward)
}

fn sample_material_exemplar(material: &str, u: u32, v: u32) -> Rgba<u8> {
    let (pattern, palette) = material_exemplar(material);
    let w = pattern[0].len() as u32;
    let h = pattern.len() as u32;
    let ix = (u % w) as usize;
    let iy = (v % h) as usize;
    rgba(palette[pattern[iy][ix] as usize])
}

fn builtin_material_image(material: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (pattern, palette) = material_exemplar(material);
    let width = pattern[0].len() as u32;
    let height = pattern.len() as u32;
    let mut image = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    for y in 0..height {
        for x in 0..width {
            image.put_pixel(x, y, rgba(palette[pattern[y as usize][x as usize] as usize]));
        }
    }
    image
}

#[allow(dead_code)]
fn quilt_material_field(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let width = 256u32;
    let height = 256u32;
    let patch = 8u32.min(source.width().max(1)).min(source.height().max(1));
    let overlap = 3u32.min(patch.saturating_sub(1));
    let step = patch.saturating_sub(overlap).max(1);
    let mut field = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    let mut filled = vec![false; (width * height) as usize];

    let max_x = if width > patch { width - patch } else { 0 };
    let max_y = if height > patch { height - patch } else { 0 };
    let mut by = 0;
    while by <= max_y {
        let mut bx = 0;
        while bx <= max_x {
            let (sx, sy) = choose_patch_origin(material, source, &field, &filled, bx, by, patch, overlap);
            blit_patch(source, &mut field, &mut filled, sx, sy, bx, by, patch);
            if bx == max_x {
                break;
            }
            bx = (bx + step).min(max_x);
        }
        if by == max_y {
            break;
        }
        by = (by + step).min(max_y);
    }

    field
}

#[derive(Clone)]
#[allow(dead_code)]
struct WfcPattern {
    pixels: Vec<[u8; 4]>,
    band: usize,
}

#[allow(dead_code)]
fn wfc_material_field(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let n = 3usize.min(source.width().max(1) as usize).min(source.height().max(1) as usize);
    if n == 0 {
        return None;
    }
    let band_count = material
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .count()
        .max(1);
    let band_height = (source.height().max(1) as usize / band_count.max(1)).max(1);
    let patterns = extract_wfc_patterns(source, n, band_height, band_count);
    if patterns.is_empty() {
        return None;
    }
    let compat = build_wfc_compatibility(&patterns, n);
    for salt in 0..2u32 {
        if let Some(field) = solve_wfc_field(material, &patterns, &compat, n, 48, 48, band_count, salt) {
            return Some(field);
        }
    }
    None
}

#[allow(dead_code)]
fn extract_wfc_patterns(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    n: usize,
    band_height: usize,
    band_count: usize,
) -> Vec<WfcPattern> {
    use std::collections::HashSet;
    let mut seen: HashSet<(usize, Vec<[u8; 4]>)> = HashSet::new();
    let mut patterns = Vec::new();
    let sw = source.width().max(1) as usize;
    let sh = source.height().max(1) as usize;
    for sy in 0..sh {
        for sx in 0..sw {
            let band = (sy / band_height).min(band_count.saturating_sub(1));
            let mut pixels = Vec::with_capacity(n * n);
            for py in 0..n {
                for px in 0..n {
                    let p = source.get_pixel(((sx + px) % sw) as u32, ((sy + py) % sh) as u32);
                    pixels.push(p.0);
                }
            }
            if seen.insert((band, pixels.clone())) {
                patterns.push(WfcPattern { pixels, band });
            }
        }
    }
    patterns
}

#[allow(dead_code)]
fn build_wfc_compatibility(patterns: &[WfcPattern], n: usize) -> [Vec<Vec<usize>>; 4] {
    let mut right = vec![Vec::new(); patterns.len()];
    let mut left = vec![Vec::new(); patterns.len()];
    let mut down = vec![Vec::new(); patterns.len()];
    let mut up = vec![Vec::new(); patterns.len()];
    for (i, a) in patterns.iter().enumerate() {
        for (j, b) in patterns.iter().enumerate() {
            if patterns_compatible_right(a, b, n) {
                right[i].push(j);
                left[j].push(i);
            }
            if patterns_compatible_down(a, b, n) {
                down[i].push(j);
                up[j].push(i);
            }
        }
    }
    [right, left, down, up]
}

#[allow(dead_code)]
fn patterns_compatible_right(a: &WfcPattern, b: &WfcPattern, n: usize) -> bool {
    for y in 0..n {
        for x in 1..n {
            if a.pixels[y * n + x] != b.pixels[y * n + (x - 1)] {
                return false;
            }
        }
    }
    true
}

#[allow(dead_code)]
fn patterns_compatible_down(a: &WfcPattern, b: &WfcPattern, n: usize) -> bool {
    for y in 1..n {
        for x in 0..n {
            if a.pixels[y * n + x] != b.pixels[(y - 1) * n + x] {
                return false;
            }
        }
    }
    true
}

#[allow(dead_code)]
fn solve_wfc_field(
    material: &str,
    patterns: &[WfcPattern],
    compat: &[Vec<Vec<usize>>; 4],
    n: usize,
    width: usize,
    height: usize,
    band_count: usize,
    salt: u32,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let cells = width * height;
    let pcount = patterns.len();
    let mut wave = vec![true; cells * pcount];
    let mut counts = vec![0usize; cells];

    let band_rows = (height / band_count.max(1)).max(1);
    for y in 0..height {
        let target_band = (y / band_rows).min(band_count.saturating_sub(1));
        for x in 0..width {
            let idx = y * width + x;
            let mut count = 0usize;
            for p in 0..pcount {
                let allowed = patterns[p].band.abs_diff(target_band) <= 1;
                wave[idx * pcount + p] = allowed;
                if allowed {
                    count += 1;
                }
            }
            if count == 0 {
                for p in 0..pcount {
                    let allowed = patterns[p].band == target_band;
                    wave[idx * pcount + p] = allowed;
                    if allowed {
                        count += 1;
                    }
                }
            }
            if count == 0 {
                return None;
            }
            counts[idx] = count;
        }
    }

    loop {
        let mut best = None;
        let mut best_count = usize::MAX;
        for idx in 0..cells {
            let c = counts[idx];
            if c > 1 && c < best_count {
                best_count = c;
                best = Some(idx);
            }
        }
        let Some(cell_idx) = best else { break };
        let allowed: Vec<usize> = (0..pcount).filter(|&p| wave[cell_idx * pcount + p]).collect();
        if allowed.is_empty() {
            return None;
        }
        let choice = allowed[hash_material_seed(material, cell_idx as u32, salt) as usize % allowed.len()];
        for p in 0..pcount {
            wave[cell_idx * pcount + p] = p == choice;
        }
        counts[cell_idx] = 1;
        if !propagate_wfc(&mut wave, &mut counts, compat, width, height, pcount, cell_idx) {
            return None;
        }
    }

    reconstruct_wfc_image(patterns, &wave, width, height, n, pcount)
}

#[allow(dead_code)]
fn propagate_wfc(
    wave: &mut [bool],
    counts: &mut [usize],
    compat: &[Vec<Vec<usize>>; 4],
    width: usize,
    height: usize,
    pcount: usize,
    start_idx: usize,
) -> bool {
    use std::collections::VecDeque;
    let mut queue = VecDeque::new();
    queue.push_back(start_idx);
    while let Some(idx) = queue.pop_front() {
        let x = idx % width;
        let y = idx / width;
        let neighbors = [
            if x + 1 < width { Some((idx + 1, 0usize)) } else { None },
            if x > 0 { Some((idx - 1, 1usize)) } else { None },
            if y + 1 < height { Some((idx + width, 2usize)) } else { None },
            if y > 0 { Some((idx - width, 3usize)) } else { None },
        ];
        for neighbor in neighbors.into_iter().flatten() {
            let (nidx, dir) = neighbor;
            let mut changed = false;
            for np in 0..pcount {
                if !wave[nidx * pcount + np] {
                    continue;
                }
                let mut supported = false;
                for p in 0..pcount {
                    if wave[idx * pcount + p] && compat[dir][p].contains(&np) {
                        supported = true;
                        break;
                    }
                }
                if !supported {
                    wave[nidx * pcount + np] = false;
                    counts[nidx] = counts[nidx].saturating_sub(1);
                    changed = true;
                }
            }
            if counts[nidx] == 0 {
                return false;
            }
            if changed {
                queue.push_back(nidx);
            }
        }
    }
    true
}

#[allow(dead_code)]
fn reconstruct_wfc_image(
    patterns: &[WfcPattern],
    wave: &[bool],
    width: usize,
    height: usize,
    n: usize,
    pcount: usize,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut chosen = vec![0usize; width * height];
    for idx in 0..chosen.len() {
        let Some(pattern_idx) = (0..pcount).find(|&p| wave[idx * pcount + p]) else {
            return None;
        };
        chosen[idx] = pattern_idx;
    }

    let out_w = width + n.saturating_sub(1);
    let out_h = height + n.saturating_sub(1);
    let mut sums = vec![[0u32; 4]; out_w * out_h];
    let mut counts = vec![0u32; out_w * out_h];

    for y in 0..height {
        for x in 0..width {
            let pattern = &patterns[chosen[y * width + x]];
            for py in 0..n {
                for px in 0..n {
                    let ox = x + px;
                    let oy = y + py;
                    let idx = oy * out_w + ox;
                    let pixel = pattern.pixels[py * n + px];
                    for c in 0..4 {
                        sums[idx][c] += pixel[c] as u32;
                    }
                    counts[idx] += 1;
                }
            }
        }
    }

    let mut image = ImageBuffer::from_pixel(out_w as u32, out_h as u32, rgba([0, 0, 0, 0]));
    for y in 0..out_h {
        for x in 0..out_w {
            let idx = y * out_w + x;
            let count = counts[idx].max(1);
            image.put_pixel(
                x as u32,
                y as u32,
                rgba([
                    (sums[idx][0] / count) as u8,
                    (sums[idx][1] / count) as u8,
                    (sums[idx][2] / count) as u8,
                    (sums[idx][3] / count) as u8,
                ]),
            );
        }
    }
    Some(image)
}

#[allow(dead_code)]
fn choose_patch_origin(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    field: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filled: &[bool],
    bx: u32,
    by: u32,
    patch: u32,
    overlap: u32,
) -> (u32, u32) {
    let mut best = Vec::new();
    let mut best_score = u64::MAX;
    for sy in 0..source.height().max(1) {
        for sx in 0..source.width().max(1) {
            let score = patch_overlap_score(source, field, filled, sx, sy, bx, by, patch, overlap);
            if score < best_score {
                best_score = score;
                best.clear();
                best.push((sx, sy));
            } else if score == best_score {
                best.push((sx, sy));
            }
        }
    }
    let choice = hash_material_seed(material, bx, by) as usize % best.len().max(1);
    best.get(choice).copied().unwrap_or((0, 0))
}

#[allow(dead_code)]
fn patch_overlap_score(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    field: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filled: &[bool],
    sx: u32,
    sy: u32,
    bx: u32,
    by: u32,
    patch: u32,
    overlap: u32,
) -> u64 {
    let mut score = 0u64;
    for py in 0..patch {
        for px in 0..patch {
            let in_overlap = (px < overlap && bx > 0) || (py < overlap && by > 0);
            if !in_overlap {
                continue;
            }
            let fx = bx + px;
            let fy = by + py;
            let idx = (fy * field.width() + fx) as usize;
            if !filled.get(idx).copied().unwrap_or(false) {
                continue;
            }
            let src = *source.get_pixel(sx.wrapping_add(px) % source.width().max(1), sy.wrapping_add(py) % source.height().max(1));
            let dst = *field.get_pixel(fx, fy);
            score += pixel_distance(src, dst);
        }
    }
    score
}

#[allow(dead_code)]
fn blit_patch(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    field: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    filled: &mut [bool],
    sx: u32,
    sy: u32,
    bx: u32,
    by: u32,
    patch: u32,
) {
    for py in 0..patch {
        for px in 0..patch {
            let fx = bx + px;
            let fy = by + py;
            if fx >= field.width() || fy >= field.height() {
                continue;
            }
            let src = *source.get_pixel(sx.wrapping_add(px) % source.width().max(1), sy.wrapping_add(py) % source.height().max(1));
            field.put_pixel(fx, fy, src);
            let idx = (fy * field.width() + fx) as usize;
            if let Some(slot) = filled.get_mut(idx) {
                *slot = true;
            }
        }
    }
}

#[allow(dead_code)]
fn pixel_distance(a: Rgba<u8>, b: Rgba<u8>) -> u64 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    let da = a[3] as i32 - b[3] as i32;
    (dr * dr + dg * dg + db * db + da * da) as u64
}

#[allow(dead_code)]
fn hash_material_seed(material: &str, x: u32, y: u32) -> u32 {
    let mut hash = 2166136261u32;
    for byte in material.bytes() {
        hash = hash.wrapping_mul(16777619) ^ byte as u32;
    }
    hash ^ x.wrapping_mul(0x9e3779b1) ^ y.wrapping_mul(0x85ebca6b)
}

fn material_exemplar(material: &str) -> (&'static [[u8; 8]; 8], &'static [[u8; 4]; 4]) {
    match material {
        "grass" => (&GRASS_PATTERN, &GRASS_PALETTE),
        "dirt" => (&DIRT_PATTERN, &DIRT_PALETTE),
        "rock" => (&ROCK_PATTERN, &ROCK_PALETTE),
        _ => (&ROCK_PATTERN, &ROCK_PALETTE),
    }
}

const GRASS_PATTERN: [[u8; 8]; 8] = [
    [0, 1, 0, 2, 0, 1, 0, 2],
    [1, 2, 1, 3, 1, 2, 1, 3],
    [0, 1, 0, 2, 0, 1, 0, 2],
    [1, 3, 1, 2, 1, 3, 1, 2],
    [0, 1, 0, 2, 0, 1, 0, 2],
    [1, 2, 1, 3, 1, 2, 1, 3],
    [0, 1, 0, 2, 0, 1, 0, 2],
    [1, 3, 1, 2, 1, 3, 1, 2],
];

const DIRT_PATTERN: [[u8; 8]; 8] = [
    [0, 1, 1, 0, 2, 1, 0, 1],
    [1, 2, 0, 1, 1, 0, 2, 1],
    [0, 1, 3, 1, 0, 1, 1, 0],
    [1, 0, 1, 2, 1, 3, 0, 1],
    [2, 1, 0, 1, 0, 1, 2, 0],
    [1, 0, 1, 3, 1, 0, 1, 2],
    [0, 2, 1, 0, 1, 2, 0, 1],
    [1, 1, 0, 2, 0, 1, 3, 0],
];

const ROCK_PATTERN: [[u8; 8]; 8] = [
    [1, 0, 1, 0, 2, 0, 1, 0],
    [0, 2, 0, 1, 0, 1, 0, 2],
    [1, 0, 3, 0, 1, 0, 2, 0],
    [0, 1, 0, 2, 0, 3, 0, 1],
    [2, 0, 1, 0, 2, 0, 1, 0],
    [0, 1, 0, 3, 0, 2, 0, 1],
    [1, 0, 2, 0, 1, 0, 3, 0],
    [0, 2, 0, 1, 0, 1, 0, 2],
];

const GRASS_PALETTE: [[u8; 4]; 4] = [
    [66, 134, 58, 255],
    [88, 178, 82, 255],
    [118, 212, 103, 255],
    [174, 238, 144, 255],
];

const DIRT_PALETTE: [[u8; 4]; 4] = [
    [88, 54, 32, 255],
    [122, 78, 48, 255],
    [158, 106, 64, 255],
    [194, 142, 86, 255],
];

const ROCK_PALETTE: [[u8; 4]; 4] = [
    [62, 68, 86, 255],
    [92, 102, 120, 255],
    [124, 134, 152, 255],
    [168, 176, 188, 255],
];

fn surface_u_rgba(surface_u: u32, loop_len: usize) -> Rgba<u8> {
    if loop_len <= 1 {
        return rgba([245, 245, 245, 255]);
    }
    let t = surface_u as f32 / (loop_len.saturating_sub(1) as f32).max(1.0);
    let r = (100.0 + 155.0 * (t * std::f32::consts::TAU).cos().abs()) as u8;
    let g = (80.0 + 175.0 * ((t + 0.33) * std::f32::consts::TAU).sin().abs()) as u8;
    let b = (80.0 + 175.0 * ((t + 0.66) * std::f32::consts::TAU).sin().abs()) as u8;
    rgba([r, g, b, 255])
}

fn region_color_rgba(region_id: usize) -> Rgba<u8> {
    let hash = (region_id as u32).wrapping_mul(0x9e3779b1);
    let r = 60 + (hash & 0x9f) as u8;
    let g = 60 + ((hash >> 8) & 0x9f) as u8;
    let b = 60 + ((hash >> 16) & 0x9f) as u8;
    rgba([r, g, b, 255])
}

fn region_outline_rgba(region_id: usize) -> Rgba<u8> {
    let hash = (region_id as u32).wrapping_mul(0x85ebca6b);
    let r = 140 + (hash & 0x5f) as u8;
    let g = 140 + ((hash >> 8) & 0x5f) as u8;
    let b = 140 + ((hash >> 16) & 0x5f) as u8;
    rgba([r, g, b, 255])
}

fn rgba(bytes: [u8; 4]) -> Rgba<u8> {
    Rgba(bytes)
}

fn sanitize_debug_name(name: &str) -> String {
    let mut output = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
        } else if !output.ends_with('_') {
            output.push('_');
        }
    }
    output.trim_matches('_').to_string()
}

fn terrain_debug_readme() -> String {
    r#"RPU Terrain Debug Output

Files:

- *_terrain.png
  Per-cell shape classification debug image.
- *__tangents.png
  Per-cell tangent debug image.
- *__materials.png
  Resolved material-layer debug image.
- *__synth.png
  First build-time synthesized terrain preview.
- *__synth_layers.png
  Per-pixel winning material layer in the synth preview.
- *__transitions.png
  Surface-coordinate transition debug image.
- *__bands.png
  Boundary-distance and depth-band debug image.
- *__regions.png
  Connected terrain-region debug image.
- *__loops.png
  Ordered region-boundary loop debug image.
- *__contours.png
  Per-cell interpreted surface contour debug image.
- *__influences.png
  Ramp and plateau-join influence debug image.
- *__heightfield.png
  Per-cell local contour heightfield debug image.

In *_terrain.png:

- fill color = derived terrain shape
- small inner accent = material identity
- center marker = edge style
- bright edge strokes = exposed sides
- white arrow = coarse surface normal

Shape colors:

- bright cyan = TopLeftOuter
- blue-cyan = TopRightOuter
- green = Top
- orange = Left
- darker orange = Right
- magenta = Bottom
- pink-purple = BottomLeftOuter
- violet = BottomRightOuter
- dark blue = Interior
- red = Isolated
- pale green / pale red shades = inner-corner classes

Exposed side strokes:

- bright top edge = exposed top
- bright bottom edge = exposed bottom
- bright left edge = exposed left
- bright right edge = exposed right

Center style markers:

- square dot = Square
- rounded cross = Round
- diagonal slash = Diagonal

Normal arrows:

- up / down / left / right = single exposed side
- diagonal arrows = exposed corner direction
- normals are coarse, discrete debug directions for now

In *__regions.png:

- each connected terrain region gets its own color
- outline color distinguishes region borders
- grouping is currently based on:
  - same material
  - 4-neighbor connectivity

In *__tangents.png:

- fill color = derived terrain shape
- cyan arrow = coarse tangent direction

In *__materials.png:

- fill color = resolved effective material layer
- dark inner accent = shape
- bottom bar length = boundary distance
- top-facing and diagonal outer surfaces keep the top material
- side and bottom outer surfaces fall back to the next material

In *__synth.png:

- coherent preview generated from source textures when available
- sampling follows:
  - `surface_u` along the region boundary loop
  - `boundary_distance` inward from the surface
- source texture lookup order:
  - `assets/<material>.png`
  - `assets/terrain/<material>.png`
  - `assets/terrain_<material>.png`
- if no source texture is found, built-in material exemplars are used
- this is the first synthesis prototype, not final WFC

In *__synth_layers.png:

- fill color = material layer that actually wins the per-pixel composite
- top stripe = resolved material for that cell
- brightened pixels = winner differs from the cell's resolved material
- use this to inspect where a cap texture is transparent and deeper layers show through

In *__transitions.png:

- fill color = resolved effective material layer
- top stripe color = `surface_u` position along the region boundary loop
- bottom bar length = `boundary_distance`

In *__bands.png:

- red = Edge
- amber = NearSurface
- blue = Interior
- deep blue = DeepInterior
- bottom bar length = boundary distance

In *__loops.png:

- region fill colors match *__regions.png
- white lines connect each region's ordered boundary loop
- this is the current deterministic perimeter traversal order

In *__contours.png:

- fill color = derived terrain shape
- bright edge strokes = exposed sides
- white contour line = interpreted local surface profile
- use this to check whether flats and diagonals form a coherent top contour

In *__influences.png:

- dark fill = no special transition influence
- purple fill = ramp body
- green fill = plateau cell influenced by a neighboring ramp
- olive fill = influenced from both sides
- white contour line = local contour
- use this to inspect which plateau cells participate in a ramp connection

In *__heightfield.png:

- grayscale fill = local surface height inside the tile
- white profile line = sampled contour height across the tile width
- this is the current field used to derive inward depth for synthesis
"#
    .to_string()
}
