use anyhow::{Context, Result, bail};
use rpu_core::{Diagnostic, RpuProject};
use std::fs;
use std::io::Read;
use std::net::TcpListener;
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

    println!("Wrote build placeholder to {}", build_dir.display());
    Ok(())
}

pub fn build_web_project(project_root: &Path) -> Result<()> {
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
