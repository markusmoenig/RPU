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

    let material_sources = load_material_sources(project_root, classified);
    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([10, 12, 16, 255]));
    for cell in &classified.cells {
        let x = border + cell.col as u32 * (tile + gap);
        let y = border + cell.row as u32 * (tile + gap);
        for py in 0..tile {
            for px in 0..tile {
                let color = synthesize_terrain_pixel(&material_sources, cell, px, py, tile);
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
    material_sources: &std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    cell: &rpu_core::ClassifiedMapCell,
    px: u32,
    py: u32,
    tile: u32,
) -> Rgba<u8> {
    let along = along_surface_projection(cell.tangent, px, py, tile);
    let inward = inward_projection(cell.normal, px, py, tile);
    let u = cell.surface_u * tile + along;
    let v = cell.boundary_distance * tile + inward;
    sample_material_source(material_sources, &cell.material, u, v)
}

fn load_material_sources(
    project_root: &Path,
    classified: &rpu_core::ClassifiedAsciiMap,
) -> std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut sources = std::collections::HashMap::new();
    let materials: std::collections::HashSet<_> = classified
        .cells
        .iter()
        .map(|cell| cell.material.as_str())
        .collect();

    for material in materials {
        if let Some(image) = load_material_source(project_root, material) {
            sources.insert(material.to_string(), image);
        }
    }

    sources
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

fn sample_material_source(
    material_sources: &std::collections::HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    material: &str,
    u: u32,
    v: u32,
) -> Rgba<u8> {
    if let Some(image) = material_sources.get(material) {
        let x = u % image.width().max(1);
        let y = v % image.height().max(1);
        return *image.get_pixel(x, y);
    }
    sample_material_exemplar(material, u, v)
}

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

fn inward_projection(
    normal: rpu_core::TerrainNormal,
    px: u32,
    py: u32,
    tile: u32,
) -> u32 {
    match normal {
        rpu_core::TerrainNormal::None => py,
        rpu_core::TerrainNormal::Up => py,
        rpu_core::TerrainNormal::Down => tile.saturating_sub(1).saturating_sub(py),
        rpu_core::TerrainNormal::Left => px,
        rpu_core::TerrainNormal::Right => tile.saturating_sub(1).saturating_sub(px),
        rpu_core::TerrainNormal::UpLeft => (px + py) / 2,
        rpu_core::TerrainNormal::UpRight => (tile.saturating_sub(1).saturating_sub(px) + py) / 2,
        rpu_core::TerrainNormal::DownLeft => (px + tile.saturating_sub(1).saturating_sub(py)) / 2,
        rpu_core::TerrainNormal::DownRight => (
            tile.saturating_sub(1).saturating_sub(px) + tile.saturating_sub(1).saturating_sub(py)
        ) / 2,
    }
}

fn sample_material_exemplar(material: &str, u: u32, v: u32) -> Rgba<u8> {
    let (pattern, palette) = material_exemplar(material);
    let w = pattern[0].len() as u32;
    let h = pattern.len() as u32;
    let ix = (u % w) as usize;
    let iy = (v % h) as usize;
    rgba(palette[pattern[iy][ix] as usize])
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
- *__transitions.png
  Surface-coordinate transition debug image.
- *__bands.png
  Boundary-distance and depth-band debug image.
- *__regions.png
  Connected terrain-region debug image.
- *__loops.png
  Ordered region-boundary loop debug image.

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
"#
    .to_string()
}
