use anyhow::{Context, Result, bail};
use rpu_core::{
    BinaryOp, BuildBackend, BuiltCartridge, BuiltCartridgeManifest, BytecodeOp, CartridgeEntry,
    CartridgeFormatInfo, CartridgeProjectInfo, CompareOp, Condition, Diagnostic, Expr,
    ModuleBackend, OpCode, ProjectKind, RpuProject, SourceLanguage, wasm_abi,
};
use std::env;
use std::fs;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use tiny_http::{Header, Response, Server, StatusCode};

const WASM_BINDGEN_VERSION: &str = "0.2.126";

pub fn new_project(name: &str, path: Option<&Path>) -> Result<()> {
    let root = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(name));
    RpuProject::create(&root, name)?;
    println!("Created RPU cartridge at {}", root.display());
    Ok(())
}

pub fn run_project(project_root: &Path, args: &[String]) -> Result<i32> {
    if BuiltCartridge::is_bundle_path(project_root) {
        return run_built_cartridge(project_root, args);
    }
    let project = RpuProject::load(project_root)?;
    if project.build().backend == BuildBackend::Wasm {
        let cartridge = build_cartridge(&project)?;
        return run_built_cartridge(&cartridge, args);
    }
    match project.kind() {
        ProjectKind::App => {
            rpu_runtime::run(project)?;
            Ok(0)
        }
        ProjectKind::Cli => run_cli_project(project, args),
        ProjectKind::Module => bail!("module cartridges cannot be run directly"),
    }
}

pub fn build_project(project_root: &Path) -> Result<()> {
    let project = RpuProject::load(project_root)?;
    if project.build().backend == BuildBackend::Wasm {
        let cartridge = build_cartridge(&project)?;
        println!("Built cartridge at {}", cartridge.display());
        return Ok(());
    }
    let compiled = project.compile()?;
    let build_dir = project.root().join("build");
    fs::create_dir_all(&build_dir)
        .with_context(|| format!("failed to create {}", build_dir.display()))?;

    let summary = format!(
        "RPU build placeholder\nproject = {}\nversion = {}\nkind = {:?}\nbuild_language = {:?}\nbuild_backend = {:?}\nroot = {}\nmodules = {}\nscene_defs = {}\nscene_files = {}\nscripts = {}\ncameras = {}\nrects = {}\nsprites = {}\nhandlers = {}\nops = {}\nassets = {}\nwarnings = {}\nerrors = {}\n\n{}",
        compiled.name,
        compiled.version,
        compiled.kind,
        compiled.build.language,
        compiled.build.backend,
        project.root().display(),
        compiled.modules.len(),
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

fn run_built_cartridge(cartridge_root: &Path, args: &[String]) -> Result<i32> {
    let cartridge = BuiltCartridge::load(cartridge_root)?;
    let manifest = cartridge.manifest();
    match manifest.project.kind {
        ProjectKind::Cli => {}
        ProjectKind::Module => bail!("module cartridges cannot be run directly"),
        ProjectKind::App => bail!("WASM app cartridge execution is not implemented yet"),
    }
    if manifest.entry.backend != BuildBackend::Wasm {
        bail!(
            "cartridge entry backend {:?} is not executable yet",
            manifest.entry.backend
        );
    }
    let mut loaded_modules = Vec::with_capacity(manifest.modules.len());
    for module in &manifest.modules {
        if module.backend != ModuleBackend::Wasm {
            bail!(
                "module `{}` backend {:?} is not executable yet",
                module.name,
                module.backend
            );
        }
        let path = cartridge.module_path(module);
        let bytes = fs::read(&path).with_context(|| {
            format!(
                "failed to read module `{}` at {}",
                module.name,
                path.display()
            )
        })?;
        let loaded = rpu_wasm::load_module(&bytes, &manifest.requires, args)
            .with_context(|| format!("failed to initialize module `{}`", module.name))?;
        loaded_modules.push(loaded);
    }
    let entry = cartridge.entry_path();
    let bytes = fs::read(&entry).with_context(|| format!("failed to read {}", entry.display()))?;
    rpu_wasm::run_cli(&bytes, &manifest.requires, args)
        .with_context(|| format!("failed to run cartridge {}", cartridge.root().display()))
}

fn build_cartridge(project: &RpuProject) -> Result<PathBuf> {
    let entry_artifact = build_wasm_project(project)?;
    package_wasm_cartridge(project, &entry_artifact)
}

fn build_wasm_project(project: &RpuProject) -> Result<PathBuf> {
    match project.build().language {
        SourceLanguage::C => build_c_wasm_project(project),
        SourceLanguage::Rpu => bail!("RPU-to-WASM compilation is not implemented yet"),
        language => bail!("{language:?}-to-WASM compilation is not implemented yet"),
    }
}

fn build_c_wasm_project(project: &RpuProject) -> Result<PathBuf> {
    let sources_dir = project.root().join("src");
    let mut sources = Vec::new();
    collect_files_with_extension(&sources_dir, "c", &mut sources)?;
    sources.sort();
    if sources.is_empty() {
        bail!(
            "C cartridge does not contain any `.c` files under {}",
            sources_dir.display()
        );
    }

    let clang = find_wasm_clang().context(
        "no WebAssembly-capable Clang found; install upstream LLVM and LLD (for example `brew install llvm lld`) or set RPU_CLANG",
    )?;
    let sdk_root = c_sdk_root();
    let sdk_include = sdk_root.join("include");
    let sdk_source = sdk_root.join("src/rpu.c");
    if !sdk_include.join("rpu.h").is_file() || !sdk_source.is_file() {
        bail!(
            "RPU C SDK not found at {}; set RPU_C_SDK to its directory",
            sdk_root.display()
        );
    }

    let build_dir = project.root().join("build");
    fs::create_dir_all(&build_dir)
        .with_context(|| format!("failed to create {}", build_dir.display()))?;
    let artifact = build_dir.join("main.wasm");

    let mut command = Command::new(&clang);
    command
        .arg("--target=wasm32-unknown-unknown")
        .arg("-std=c11")
        .arg("-O2")
        .arg("-ffreestanding")
        .arg("-fno-builtin")
        .arg("-fvisibility=hidden")
        .arg("-nostdlib")
        .arg("-I")
        .arg(&sdk_include)
        .arg(&sdk_source);
    if project.kind() == ProjectKind::Module {
        command.arg("-DRPU_CARTRIDGE_MODULE=1");
    }
    for source in &sources {
        command.arg(source);
    }
    let output = command
        .arg("-Wl,--no-entry")
        .arg("-Wl,--allow-undefined")
        .arg("-Wl,--export-memory")
        .arg("-Wl,--initial-memory=131072")
        .arg("-Wl,--max-memory=16777216")
        .arg("-Wl,--strip-all")
        .arg("-o")
        .arg(&artifact)
        .output()
        .with_context(|| format!("failed to launch {}", clang.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("C-to-WASM compilation failed:\n{}", stderr.trim());
    }

    Ok(artifact)
}

fn package_wasm_cartridge(project: &RpuProject, entry_artifact: &Path) -> Result<PathBuf> {
    let build_dir = project.root().join("build");
    let artifact_name = cartridge_artifact_name(project.name());
    let cartridge_dir = build_dir.join(format!("{artifact_name}.cart"));
    let staging_dir = build_dir.join(format!(".{artifact_name}.cart.tmp"));

    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)
            .with_context(|| format!("failed to clean {}", staging_dir.display()))?;
    }
    fs::create_dir_all(&staging_dir)
        .with_context(|| format!("failed to create {}", staging_dir.display()))?;

    fs::copy(entry_artifact, staging_dir.join("main.wasm")).with_context(|| {
        format!(
            "failed to copy WASM entry {} into cartridge",
            entry_artifact.display()
        )
    })?;
    for directory in ["assets", "shaders", "modules"] {
        let source = project.root().join(directory);
        if source.exists() {
            copy_cartridge_directory(&source, &staging_dir.join(directory))?;
        }
    }

    let manifest = BuiltCartridgeManifest {
        cartridge: CartridgeFormatInfo {
            format_version: rpu_core::CARTRIDGE_FORMAT_VERSION,
            abi_version: wasm_abi::ABI_VERSION,
        },
        project: CartridgeProjectInfo {
            name: project.name().to_string(),
            version: project.version().to_string(),
            kind: project.kind(),
        },
        entry: CartridgeEntry {
            backend: BuildBackend::Wasm,
            path: PathBuf::from("main.wasm"),
        },
        requires: project.requires().clone(),
        modules: project.modules().to_vec(),
    };
    let manifest_text =
        toml::to_string_pretty(&manifest).context("failed to serialize cartridge manifest")?;
    fs::write(staging_dir.join("manifest.toml"), manifest_text).with_context(|| {
        format!(
            "failed to write {}",
            staging_dir.join("manifest.toml").display()
        )
    })?;

    BuiltCartridge::load(&staging_dir).context("generated cartridge failed validation")?;
    if cartridge_dir.exists() {
        fs::remove_dir_all(&cartridge_dir)
            .with_context(|| format!("failed to replace {}", cartridge_dir.display()))?;
    }
    fs::rename(&staging_dir, &cartridge_dir).with_context(|| {
        format!(
            "failed to move {} to {}",
            staging_dir.display(),
            cartridge_dir.display()
        )
    })?;
    Ok(cartridge_dir)
}

fn cartridge_artifact_name(name: &str) -> String {
    let name = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if name.is_empty() {
        "cartridge".to_string()
    } else {
        name
    }
}

fn copy_cartridge_directory(source: &Path, destination: &Path) -> Result<()> {
    let source_metadata = fs::symlink_metadata(source)
        .with_context(|| format!("failed to inspect {}", source.display()))?;
    if source_metadata.file_type().is_symlink() {
        bail!(
            "cartridge resources may not be symbolic links: {}",
            source.display()
        );
    }
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;
    for entry in fs::read_dir(source)
        .with_context(|| format!("failed to read cartridge resources {}", source.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_symlink() {
            bail!(
                "cartridge resources may not be symbolic links: {}",
                entry.path().display()
            );
        }
        if file_type.is_dir() {
            copy_cartridge_directory(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy cartridge resource {}",
                    entry.path().display()
                )
            })?;
        }
    }
    Ok(())
}

fn collect_files_with_extension(dir: &Path, extension: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed to read source directory {}", dir.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, out)?;
        } else if path.extension() == Some(std::ffi::OsStr::new(extension)) {
            out.push(path);
        }
    }
    Ok(())
}

fn c_sdk_root() -> PathBuf {
    env::var_os("RPU_C_SDK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("sdk/c")
        })
}

fn find_wasm_clang() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("RPU_CLANG") {
        candidates.push(PathBuf::from(path));
    }
    candidates.extend([
        PathBuf::from("/opt/homebrew/opt/llvm/bin/clang"),
        PathBuf::from("/usr/local/opt/llvm/bin/clang"),
        PathBuf::from("clang"),
    ]);

    candidates
        .into_iter()
        .find(|candidate| clang_supports_wasm(candidate))
}

fn clang_supports_wasm(clang: &Path) -> bool {
    Command::new(clang)
        .arg("--print-targets")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("wasm32"))
        .unwrap_or(false)
}

fn run_cli_project(project: RpuProject, args: &[String]) -> Result<i32> {
    let compiled = project.compile()?;
    if compiled.has_errors() {
        bail!(
            "CLI cartridge has compile errors:\n{}",
            format_diagnostics(&compiled.diagnostics)
        );
    }

    let mut context = CliContext {
        args: args.to_vec(),
    };
    let mut ran = false;
    for script in &compiled.bytecode_scripts {
        for handler in &script.handlers {
            if handler.event != "run" {
                continue;
            }
            ran = true;
            if let CliSignal::Exit(code) = execute_cli_ops(&handler.ops, &mut context)
                .with_context(|| format!("failed to run {}", script.path.display()))?
            {
                return Ok(code);
            }
        }
    }

    if !ran {
        bail!("CLI cartridge does not define `on run()` in any script");
    }

    Ok(0)
}

struct CliContext {
    args: Vec<String>,
}

enum CliSignal {
    Continue,
    Exit(i32),
}

#[derive(Clone)]
enum CliValue {
    Number(f32),
    String(String),
}

fn execute_cli_ops(ops: &[BytecodeOp], context: &mut CliContext) -> Result<CliSignal> {
    for op in ops {
        match &op.op {
            OpCode::Log(message) => println!("{message}"),
            OpCode::Call(name, args) if name == "print" || name == "log" => {
                execute_cli_print(name, args, op.line, context)?
            }
            OpCode::Call(name, args) if name == "eprint" => {
                execute_cli_eprint(name, args, op.line, context)?
            }
            OpCode::Call(name, args) if name == "exit" => {
                return Ok(CliSignal::Exit(execute_cli_exit(
                    name, args, op.line, context,
                )?));
            }
            OpCode::IgnoreValue(_) => {}
            OpCode::Raw(raw) => bail!(
                "unsupported CLI script statement at line {}: {}",
                op.line,
                raw
            ),
            OpCode::If(condition, body, else_body) => {
                let branch = if eval_cli_condition(condition, context)? {
                    body
                } else {
                    else_body
                };
                if let CliSignal::Exit(code) = execute_cli_ops(branch, context)? {
                    return Ok(CliSignal::Exit(code));
                }
            }
            other => bail!(
                "unsupported CLI script operation at line {}: {:?}",
                op.line,
                other
            ),
        }
    }
    Ok(CliSignal::Continue)
}

fn execute_cli_print(name: &str, args: &[Expr], line: usize, context: &CliContext) -> Result<()> {
    let message = cli_single_arg(name, args, line, context)?.to_output_string();
    println!("{message}");
    Ok(())
}

fn execute_cli_eprint(name: &str, args: &[Expr], line: usize, context: &CliContext) -> Result<()> {
    let message = cli_single_arg(name, args, line, context)?.to_output_string();
    eprintln!("{message}");
    Ok(())
}

fn execute_cli_exit(name: &str, args: &[Expr], line: usize, context: &CliContext) -> Result<i32> {
    let code = cli_single_arg(name, args, line, context)?.to_number(name, line)?;
    Ok(code.round().clamp(0.0, 255.0) as i32)
}

fn cli_single_arg(
    name: &str,
    args: &[Expr],
    line: usize,
    context: &CliContext,
) -> Result<CliValue> {
    let Some(expr) = args.first() else {
        bail!("CLI `{name}` call expects one argument at line {line}");
    };
    if args.len() != 1 {
        bail!("CLI `{name}` call expects one argument at line {line}");
    }
    eval_cli_expr(expr, context)
}

fn eval_cli_condition(condition: &Condition, context: &CliContext) -> Result<bool> {
    match condition {
        Condition::Compare { left, op, right } => {
            let left = eval_cli_expr(left, context)?;
            let right = eval_cli_expr(right, context)?;
            compare_cli_values(&left, *op, &right)
        }
        Condition::And(left, right) => {
            Ok(eval_cli_condition(left, context)? && eval_cli_condition(right, context)?)
        }
        Condition::Or(left, right) => {
            Ok(eval_cli_condition(left, context)? || eval_cli_condition(right, context)?)
        }
        Condition::Not(condition) => Ok(!eval_cli_condition(condition, context)?),
    }
}

fn eval_cli_expr(expr: &Expr, context: &CliContext) -> Result<CliValue> {
    match expr {
        Expr::Number(value) => Ok(CliValue::Number(*value)),
        Expr::String(value) => Ok(CliValue::String(value.clone())),
        Expr::Call(name, args) if name == "arg_count" => {
            if !args.is_empty() {
                bail!("CLI `arg_count` expects no arguments");
            }
            Ok(CliValue::Number(context.args.len() as f32))
        }
        Expr::Call(name, args) if name == "arg" => {
            let index = cli_single_arg(name, args, 0, context)?.to_number(name, 0)?;
            let index = index.round().max(0.0) as usize;
            Ok(CliValue::String(
                context.args.get(index).cloned().unwrap_or_default(),
            ))
        }
        Expr::Binary(left, op, right) => {
            let left = eval_cli_expr(left, context)?;
            let right = eval_cli_expr(right, context)?;
            eval_cli_binary(&left, *op, &right)
        }
        Expr::Clamp(value, min, max) => {
            let value = eval_cli_expr(value, context)?.to_number("clamp", 0)?;
            let min = eval_cli_expr(min, context)?.to_number("clamp", 0)?;
            let max = eval_cli_expr(max, context)?.to_number("clamp", 0)?;
            Ok(CliValue::Number(value.clamp(min, max)))
        }
        Expr::Variable(name) => bail!("CLI variable `{name}` is not supported yet"),
        Expr::Dt => Ok(CliValue::Number(0.0)),
        Expr::Target(_) | Expr::Color(_) | Expr::Call(_, _) => {
            bail!("unsupported CLI expression: {:?}", expr)
        }
    }
}

fn eval_cli_binary(left: &CliValue, op: BinaryOp, right: &CliValue) -> Result<CliValue> {
    if matches!(op, BinaryOp::Add) {
        if let (CliValue::String(left), CliValue::String(right)) = (left, right) {
            return Ok(CliValue::String(format!("{left}{right}")));
        }
    }
    let left = left.to_number("binary expression", 0)?;
    let right = right.to_number("binary expression", 0)?;
    Ok(CliValue::Number(match op {
        BinaryOp::Add => left + right,
        BinaryOp::Sub => left - right,
        BinaryOp::Mul => left * right,
        BinaryOp::Div => {
            if right.abs() < f32::EPSILON {
                0.0
            } else {
                left / right
            }
        }
    }))
}

fn compare_cli_values(left: &CliValue, op: CompareOp, right: &CliValue) -> Result<bool> {
    match (left, right) {
        (CliValue::Number(left), CliValue::Number(right)) => Ok(match op {
            CompareOp::Less => left < right,
            CompareOp::LessEqual => left <= right,
            CompareOp::Greater => left > right,
            CompareOp::GreaterEqual => left >= right,
            CompareOp::Equal => (left - right).abs() < f32::EPSILON,
            CompareOp::NotEqual => (left - right).abs() >= f32::EPSILON,
        }),
        (CliValue::String(left), CliValue::String(right)) => Ok(match op {
            CompareOp::Equal => left == right,
            CompareOp::NotEqual => left != right,
            _ => false,
        }),
        _ => Ok(match op {
            CompareOp::Equal => false,
            CompareOp::NotEqual => true,
            _ => false,
        }),
    }
}

impl CliValue {
    fn to_output_string(&self) -> String {
        match self {
            CliValue::Number(value) => {
                if value.fract().abs() < f32::EPSILON {
                    format!("{}", *value as i32)
                } else {
                    value.to_string()
                }
            }
            CliValue::String(value) => value.clone(),
        }
    }

    fn to_number(&self, name: &str, line: usize) -> Result<f32> {
        match self {
            CliValue::Number(value) => Ok(*value),
            CliValue::String(value) => value.parse::<f32>().with_context(|| {
                if line == 0 {
                    format!("CLI `{name}` expected a number")
                } else {
                    format!("CLI `{name}` expected a number at line {line}")
                }
            }),
        }
    }
}

pub fn build_web_project(project_root: &Path) -> Result<()> {
    let project = RpuProject::load(project_root)?;
    if project.kind() != ProjectKind::App {
        bail!("web export currently supports only app cartridges");
    }
    ensure_web_prerequisites()?;
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

    fs::write(
        app_root.join("Cargo.toml"),
        generated_web_cargo_toml(&repo_root),
    )
    .with_context(|| format!("failed to write {}", app_root.join("Cargo.toml").display()))?;
    let _ = fs::remove_file(src_root.join("main.rs"));
    fs::write(
        src_root.join("lib.rs"),
        generated_web_main_rs(&project, &compiled),
    )
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

    let wasm_bindgen = find_wasm_bindgen().with_context(|| {
        format!(
            "wasm-bindgen CLI {WASM_BINDGEN_VERSION} is required; install it with `cargo install wasm-bindgen-cli --version {WASM_BINDGEN_VERSION} --locked --force`"
        )
    })?;
    let wasm_path = app_root.join("target/wasm32-unknown-unknown/release/rpu_web_export.wasm");
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

    fs::write(
        out_root.join("index.html"),
        generated_web_index_html(&compiled.name),
    )
    .with_context(|| format!("failed to write {}", out_root.join("index.html").display()))?;

    println!("Prepared web build at {}", out_root.display());
    Ok(())
}

pub fn serve_web_project(project_root: &Path, port: u16) -> Result<()> {
    build_web_project(project_root)?;
    let web_root = project_root.join("build/web");
    let addr = format!("127.0.0.1:{port}");
    let probe =
        TcpListener::bind(&addr).with_context(|| format!("port {} is not available", port))?;
    drop(probe);
    let server = Server::http(&addr)
        .map_err(|error| anyhow::anyhow!("failed to start server at {addr}: {error}"))?;
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
            let _ = request
                .respond(Response::from_string("Not Found").with_status_code(StatusCode(404)));
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
    if project.kind() != ProjectKind::App {
        bail!("Xcode export currently supports only app cartridges");
    }
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
    let tvos_xcodeproj_dir = output_root.join("RPUAppleTVApp.xcodeproj");
    let tvos_workspace_dir = tvos_xcodeproj_dir.join("project.xcworkspace");

    fs::create_dir_all(&app_dir)
        .with_context(|| format!("failed to create {}", app_dir.display()))?;
    fs::create_dir_all(&rust_src_dir)
        .with_context(|| format!("failed to create {}", rust_src_dir.display()))?;
    fs::create_dir_all(&bundle_project_dir)
        .with_context(|| format!("failed to create {}", bundle_project_dir.display()))?;
    fs::create_dir_all(&workspace_dir)
        .with_context(|| format!("failed to create {}", workspace_dir.display()))?;
    fs::create_dir_all(&tvos_workspace_dir)
        .with_context(|| format!("failed to create {}", tvos_workspace_dir.display()))?;

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
    let development_team = project
        .development_team()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string);
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
    fs::write(rust_src_dir.join("lib.rs"), generated_xcode_lib_rs())
        .with_context(|| format!("failed to write {}", rust_src_dir.join("lib.rs").display()))?;
    fs::write(
        rust_dir.join("build-rust.sh"),
        generated_xcode_rust_build_script(),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            rust_dir.join("build-rust.sh").display()
        )
    })?;
    build_generated_xcode_bridge(&rust_dir)?;

    fs::write(
        app_dir.join("RPUAppleApp.swift"),
        generated_xcode_app_swift(&app_display_name, default_window_size),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            app_dir.join("RPUAppleApp.swift").display()
        )
    })?;
    fs::write(app_dir.join("main.swift"), generated_xcode_main_swift())
        .with_context(|| format!("failed to write {}", app_dir.join("main.swift").display()))?;
    fs::write(
        app_dir.join("ContentView.swift"),
        generated_xcode_content_view_swift(),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            app_dir.join("ContentView.swift").display()
        )
    })?;
    fs::write(
        app_dir.join("MetalView.swift"),
        generated_xcode_metal_view_swift(),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            app_dir.join("MetalView.swift").display()
        )
    })?;
    fs::write(app_dir.join("RPUFFI.swift"), generated_xcode_ffi_swift())
        .with_context(|| format!("failed to write {}", app_dir.join("RPUFFI.swift").display()))?;
    fs::write(
        output_root.join("tvOS-Info.plist"),
        generated_xcode_tvos_info_plist(),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            output_root.join("tvOS-Info.plist").display()
        )
    })?;

    fs::write(
        xcodeproj_dir.join("project.pbxproj"),
        generated_xcode_pbxproj(
            &app_display_name,
            &app_identifier,
            development_team.as_deref(),
        ),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            xcodeproj_dir.join("project.pbxproj").display()
        )
    })?;
    fs::write(
        tvos_xcodeproj_dir.join("project.pbxproj"),
        generated_xcode_tvos_pbxproj(
            &app_display_name,
            &app_identifier,
            development_team.as_deref(),
        ),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            tvos_xcodeproj_dir.join("project.pbxproj").display()
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
    fs::write(
        tvos_workspace_dir.join("contents.xcworkspacedata"),
        generated_xcode_workspace_data(),
    )
    .with_context(|| {
        format!(
            "failed to write {}",
            tvos_workspace_dir
                .join("contents.xcworkspacedata")
                .display()
        )
    })?;

    let readme = format!(
        "# Xcode Export\n\nProject: {}\nVersion: {}\nBundle Identifier: {}\n\nThis export uses a native Apple view/surface created by Xcode and renders into that `CAMetalLayer` from Rust via FFI. It does **not** use a second renderer.\n\n## Generated Layout\n\n- `App/` shared Swift host app sources for macOS/tvOS\n- `RustBridge/` Rust static library crate used by Xcode\n- `Project/` bundled RPU scenes, scripts, and assets\n- `RPUAppleApp.xcodeproj/` macOS Xcode project\n- `RPUAppleTVApp.xcodeproj/` tvOS Xcode project\n- `tvOS-Info.plist` tvOS scene lifecycle metadata\n\n## Build Notes\n\n- Open `RPUAppleApp.xcodeproj` for macOS\n- Open `RPUAppleTVApp.xcodeproj` for Apple TV\n- The macOS export includes a prebuilt Rust static library in `RustBridge/build/`\n- The tvOS project builds the Rust archive during the Xcode build into `RustBridge/build/$(PLATFORM_NAME)/`\n- Rust render output is presented directly into a `CAMetalLayer`\n- App display name, bundle id, and Apple development team come from `[meta]` when present\n- tvOS directional input maps to movement keys; action input maps to `Space`\n- Apple audio is bridged from Rust to the native host for sound effects and music\n- If Xcode reports a missing Rust target, run the command printed by the build log, for example:\n  - `rustup target add aarch64-apple-tvos`\n  - `rustup target add aarch64-apple-tvos-sim`\n\n## Diagnostics\n\n{}\n",
        compiled.name,
        compiled.version,
        app_identifier,
        format_diagnostics(&compiled.diagnostics)
    );
    fs::write(output_root.join("README.md"), readme).with_context(|| {
        format!(
            "failed to write {}",
            output_root.join("README.md").display()
        )
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
    let macos_build_dir = build_dir.join("macosx");
    fs::create_dir_all(&build_dir)
        .with_context(|| format!("failed to create {}", build_dir.display()))?;
    fs::create_dir_all(&macos_build_dir)
        .with_context(|| format!("failed to create {}", macos_build_dir.display()))?;
    fs::copy(&built_lib, build_dir.join("librpu_apple_export.a")).with_context(|| {
        format!(
            "failed to copy generated Xcode bridge archive from {}",
            built_lib.display()
        )
    })?;
    fs::copy(&built_lib, macos_build_dir.join("librpu_apple_export.a")).with_context(|| {
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
                    diagnostic.severity, diagnostic.message, line
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
wasm-bindgen = "={WASM_BINDGEN_VERSION}"
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
            let absolute = rust_raw_literal(&canonical_display(
                project.root().join(&scene.relative_path),
            ));
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
            let absolute = rust_raw_literal(&canonical_display(
                project.root().join(&script.relative_path),
            ));
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

fn xcode_development_team_setting(development_team: Option<&str>) -> String {
    development_team
        .map(str::trim)
        .filter(|team| !team.is_empty())
        .map(|team| format!("DEVELOPMENT_TEAM = {};", team.replace('"', "\\\"")))
        .unwrap_or_default()
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

[patch.crates-io]
objc = {{ path = "{}" }}
"#,
        repo_root.join("crates/rpu-core").display(),
        repo_root.join("crates/rpu-runtime").display(),
        repo_root.join("crates/rpu-scenevm").display(),
        repo_root.join("crates/vendor/objc-0.2.7-tvos").display(),
    )
}

fn generated_xcode_rust_build_script() -> String {
    r#"#!/bin/sh
set -eu

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

cd "$(dirname "$0")"

PLATFORM="${PLATFORM_NAME:-macosx}"
BUILD_DIR="build/${PLATFORM}"
mkdir -p "${BUILD_DIR}"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Missing cargo in Xcode build environment." >&2
    echo "Install Rust from https://rustup.rs/ or make sure cargo is available at $HOME/.cargo/bin/cargo." >&2
    exit 1
fi

case "${PLATFORM}" in
    appletvos)
        RUST_TARGET="aarch64-apple-tvos"
        ;;
    appletvsimulator)
        RUST_TARGET="aarch64-apple-tvos-sim"
        ;;
    macosx)
        RUST_TARGET=""
        ;;
    *)
        echo "Unsupported Apple platform: ${PLATFORM}" >&2
        exit 1
        ;;
esac

if [ -n "${RUST_TARGET}" ]; then
    if ! command -v rustup >/dev/null 2>&1; then
        echo "Missing rustup in Xcode build environment." >&2
        echo "Install Rust from https://rustup.rs/ or make sure rustup is available at $HOME/.cargo/bin/rustup." >&2
        exit 1
    fi
    if ! rustup toolchain list | grep -q '^nightly-'; then
        echo "tvOS Rust builds currently require nightly build-std."
        echo "Run: rustup toolchain install nightly --component rust-src"
        exit 1
    fi
    if ! rustup component list --toolchain nightly | grep -q 'rust-src.*installed'; then
        echo "Missing nightly rust-src component."
        echo "Running: rustup component add rust-src --toolchain nightly"
        rustup component add rust-src --toolchain nightly
    fi
    cargo +nightly build -Z build-std=std,panic_abort --target "${RUST_TARGET}"
    cp "target/${RUST_TARGET}/debug/librpu_apple_export.a" "${BUILD_DIR}/librpu_apple_export.a"
else
    cargo build
    cp "target/debug/librpu_apple_export.a" "${BUILD_DIR}/librpu_apple_export.a"
fi
"#
    .to_string()
}

fn generated_xcode_lib_rs() -> String {
    r#"use std::ffi::{CStr, c_char, c_void};
use std::path::Path;

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
use rpu_core::RpuProject;
#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
use rpu_runtime::RuntimeApp;
#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
use rpu_scenevm::MetalLayerRunner;

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
struct RpuAppleRunner {
    runner: MetalLayerRunner<RuntimeApp>,
}

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().ok().map(ToString::to_string)
}

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
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

#[cfg(not(any(target_os = "macos", target_os = "tvos", target_os = "ios")))]
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

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_destroy(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr.cast::<RpuAppleRunner>())); }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "tvos", target_os = "ios")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_destroy(_ptr: *mut c_void) {}

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
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

#[cfg(not(any(target_os = "macos", target_os = "tvos", target_os = "ios")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_resize(
    _ptr: *mut c_void,
    _width: u32,
    _height: u32,
    _scale: f32,
) {}

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_render(ptr: *mut c_void) -> i32 {
    if let Some(runner) = unsafe { ptr.cast::<RpuAppleRunner>().as_mut() } {
        return if runner.runner.render().is_ok() { 0 } else { -1 };
    }
    -1
}

#[cfg(not(any(target_os = "macos", target_os = "tvos", target_os = "ios")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_render(_ptr: *mut c_void) -> i32 {
    -1
}

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_down(ptr: *mut c_void, key: *const c_char) {
    if let (Some(runner), Some(key)) = (unsafe { ptr.cast::<RpuAppleRunner>().as_mut() }, cstr_to_string(key)) {
        runner.runner.key_down(&key);
    }
}

#[cfg(not(any(target_os = "macos", target_os = "tvos", target_os = "ios")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_down(_ptr: *mut c_void, _key: *const c_char) {}

#[cfg(any(target_os = "macos", target_os = "tvos", target_os = "ios"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_up(ptr: *mut c_void, key: *const c_char) {
    if let (Some(runner), Some(key)) = (unsafe { ptr.cast::<RpuAppleRunner>().as_mut() }, cstr_to_string(key)) {
        runner.runner.key_up(&key);
    }
}

#[cfg(not(any(target_os = "macos", target_os = "tvos", target_os = "ios")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rpu_runner_key_up(_ptr: *mut c_void, _key: *const c_char) {}
"#
    .to_string()
}

fn generated_xcode_app_swift(app_name: &str, size: (u32, u32)) -> String {
    let escaped_name = app_name.replace('"', "\\\"");
    format!(
        r#"#if os(tvOS)
import UIKit

enum RPUWindowConfig {{
    static let contentWidth: CGFloat = {width}
    static let contentHeight: CGFloat = {height}
}}

final class AppDelegate: UIResponder, UIApplicationDelegate {{
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {{
        true
    }}

    func application(
        _ application: UIApplication,
        configurationForConnecting connectingSceneSession: UISceneSession,
        options: UIScene.ConnectionOptions
    ) -> UISceneConfiguration {{
        let configuration = UISceneConfiguration(name: "Default Configuration", sessionRole: connectingSceneSession.role)
        configuration.delegateClass = SceneDelegate.self
        return configuration
    }}
}}

final class SceneDelegate: UIResponder, UIWindowSceneDelegate {{
    var window: UIWindow?

    func scene(
        _ scene: UIScene,
        willConnectTo session: UISceneSession,
        options connectionOptions: UIScene.ConnectionOptions
    ) {{
        guard let windowScene = scene as? UIWindowScene else {{
            return
        }}
        let window = UIWindow(windowScene: windowScene)
        let controller = RPUViewController()
        window.rootViewController = controller
        window.backgroundColor = .black
        window.makeKeyAndVisible()
        self.window = window
    }}
}}
#else
import AppKit

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
#endif
"#,
        app_name = escaped_name,
        width = size.0,
        height = size.1
    )
}

fn generated_xcode_main_swift() -> String {
    r#"#if os(tvOS)
import UIKit

UIApplicationMain(
    CommandLine.argc,
    CommandLine.unsafeArgv,
    nil,
    NSStringFromClass(AppDelegate.self)
)
#else
import AppKit

let app = NSApplication.shared
enum RPUAppBootstrap {
    static let delegate = AppDelegate()
}
app.delegate = RPUAppBootstrap.delegate
app.setActivationPolicy(.regular)
app.activate(ignoringOtherApps: true)
app.run()
#endif
"#
    .to_string()
}

fn generated_xcode_content_view_swift() -> String {
    "import Foundation\n".to_string()
}

fn generated_xcode_tvos_info_plist() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>$(DEVELOPMENT_LANGUAGE)</string>
    <key>CFBundleDisplayName</key>
    <string>$(INFOPLIST_KEY_CFBundleDisplayName)</string>
    <key>CFBundleExecutable</key>
    <string>$(EXECUTABLE_NAME)</string>
    <key>CFBundleIdentifier</key>
    <string>$(PRODUCT_BUNDLE_IDENTIFIER)</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$(PRODUCT_NAME)</string>
    <key>CFBundlePackageType</key>
    <string>$(PRODUCT_BUNDLE_PACKAGE_TYPE)</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSRequiresIPhoneOS</key>
    <true/>
    <key>UIApplicationSceneManifest</key>
    <dict>
        <key>UIApplicationSupportsMultipleScenes</key>
        <false/>
        <key>UISceneConfigurations</key>
        <dict>
            <key>UIWindowSceneSessionRoleApplication</key>
            <array>
                <dict>
                    <key>UISceneConfigurationName</key>
                    <string>Default Configuration</string>
                    <key>UISceneDelegateClassName</key>
                    <string>$(PRODUCT_MODULE_NAME).SceneDelegate</string>
                </dict>
            </array>
        </dict>
    </dict>
    <key>UILaunchScreen</key>
    <dict/>
    <key>UIRequiresFullScreen</key>
    <true/>
</dict>
</plist>
"#
    .to_string()
}

fn generated_xcode_ffi_swift() -> String {
    r#"import Foundation
import QuartzCore

#if os(tvOS)
import AVFoundation

final class RPUAppleAudio {
    static let shared = RPUAppleAudio()

    private var musicPlayer: AVAudioPlayer?
    private var currentMusicPath: String?
    private var soundPlayers: [AVAudioPlayer] = []

    func playSound(_ path: String) {
        let url = URL(fileURLWithPath: path)
        do {
            let player = try AVAudioPlayer(contentsOf: url)
            player.numberOfLoops = 0
            player.prepareToPlay()
            player.play()
            soundPlayers.removeAll { !$0.isPlaying }
            soundPlayers.append(player)
        } catch {
        }
    }

    func playMusic(_ path: String) {
        if currentMusicPath == path, musicPlayer?.isPlaying == true {
            return
        }
        let url = URL(fileURLWithPath: path)
        do {
            let player = try AVAudioPlayer(contentsOf: url)
            player.numberOfLoops = -1
            player.prepareToPlay()
            player.play()
            musicPlayer?.stop()
            musicPlayer = player
            currentMusicPath = path
        } catch {
        }
    }

    func stopMusic() {
        musicPlayer?.stop()
        musicPlayer = nil
        currentMusicPath = nil
    }
}

@_cdecl("rpu_apple_play_sound")
func rpu_apple_play_sound(_ path: UnsafePointer<CChar>?) {
    guard let path else { return }
    RPUAppleAudio.shared.playSound(String(cString: path))
}

@_cdecl("rpu_apple_play_music")
func rpu_apple_play_music(_ path: UnsafePointer<CChar>?) {
    guard let path else { return }
    RPUAppleAudio.shared.playMusic(String(cString: path))
}

@_cdecl("rpu_apple_stop_music")
func rpu_apple_stop_music() {
    RPUAppleAudio.shared.stopMusic()
}
#endif

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
    r#"#if os(tvOS)
import UIKit
import QuartzCore
import Metal
import GameController

final class RPUViewController: UIViewController {
    private let metalView = MetalContainer(frame: .zero)

    override func loadView() {
        let root = UIView(frame: UIScreen.main.bounds)
        root.backgroundColor = .black
        root.isOpaque = true
        metalView.backgroundColor = .black
        root.addSubview(metalView)
        view = root
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        let ratio = RPUWindowConfig.contentWidth / max(RPUWindowConfig.contentHeight, 1)
        let bounds = view.bounds
        var width = bounds.width
        var height = width / ratio
        if height > bounds.height {
            height = bounds.height
            width = height * ratio
        }
        metalView.frame = CGRect(
            x: (bounds.width - width) * 0.5,
            y: (bounds.height - height) * 0.5,
            width: width,
            height: height
        ).integral
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        setNeedsFocusUpdate()
        updateFocusIfNeeded()
    }

    override var preferredFocusEnvironments: [UIFocusEnvironment] {
        [metalView]
    }

    override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if !metalView.handlePresses(presses, pressed: true) {
            super.pressesBegan(presses, with: event)
        }
    }

    override func pressesEnded(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if !metalView.handlePresses(presses, pressed: false) {
            super.pressesEnded(presses, with: event)
        }
    }
}

final class MetalContainer: UIView {
    override class var layerClass: AnyClass { CAMetalLayer.self }
    override var canBecomeFocused: Bool { true }

    private var metalLayer: CAMetalLayer { layer as! CAMetalLayer }
    private var handle: RPUHandle?
    private var displayLink: CADisplayLink?
    private var previousControllerKeys: Set<String> = []
    private var activeDirectionalPressKeys: Set<String> = []
    private var suppressActionUntil: CFTimeInterval = 0

    override init(frame: CGRect) {
        super.init(frame: frame)
        backgroundColor = .black
        isOpaque = true
        metalLayer.device = MTLCreateSystemDefaultDevice()
        metalLayer.pixelFormat = .bgra8Unorm
        metalLayer.framebufferOnly = false
        metalLayer.backgroundColor = UIColor.black.cgColor
        displayLink = CADisplayLink(target: self, selector: #selector(drawFrame))
        displayLink?.preferredFramesPerSecond = 60
        displayLink?.add(to: .main, forMode: .common)
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(controllerDidConnect),
            name: .GCControllerDidConnect,
            object: nil
        )
        GCController.startWirelessControllerDiscovery(completionHandler: nil)
        configureControllers()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    deinit {
        displayLink?.invalidate()
        NotificationCenter.default.removeObserver(self)
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        let scale = window?.screen.scale ?? UIScreen.main.scale
        metalLayer.contentsScale = scale
        metalLayer.drawableSize = CGSize(width: bounds.width * scale, height: bounds.height * scale)

        if handle == nil && bounds.width > 0 && bounds.height > 0 {
            handle = RPUHandle(layer: metalLayer, size: bounds.size, scale: scale)
        } else {
            handle?.resize(size: bounds.size, scale: scale)
        }
    }

    @objc private func controllerDidConnect(_ notification: Notification) {
        configureControllers()
    }

    @objc private func drawFrame() {
        updateControllerInput()
        handle?.render()
    }

    private func configureControllers() {
        for controller in GCController.controllers() {
            controller.microGamepad?.reportsAbsoluteDpadValues = false
            controller.microGamepad?.allowsRotation = true
        }
    }

    private func updateControllerInput() {
        guard let controller = GCController.controllers().first else {
            releasePreviousControllerKeys()
            return
        }

        var pressedKeys = Set<String>()

        if let gamepad = controller.extendedGamepad {
            let hasDirectionalInput =
                gamepad.leftThumbstick.xAxis.value < -0.25 ||
                gamepad.leftThumbstick.xAxis.value > 0.25 ||
                gamepad.leftThumbstick.yAxis.value > 0.25 ||
                gamepad.leftThumbstick.yAxis.value < -0.25 ||
                gamepad.dpad.left.isPressed ||
                gamepad.dpad.right.isPressed ||
                gamepad.dpad.up.isPressed ||
                gamepad.dpad.down.isPressed
            if gamepad.leftThumbstick.xAxis.value < -0.25 || gamepad.dpad.left.isPressed { pressedKeys.insert("ArrowLeft") }
            if gamepad.leftThumbstick.xAxis.value > 0.25 || gamepad.dpad.right.isPressed { pressedKeys.insert("ArrowRight") }
            if gamepad.leftThumbstick.yAxis.value > 0.25 || gamepad.dpad.up.isPressed { pressedKeys.insert("ArrowUp") }
            if gamepad.leftThumbstick.yAxis.value < -0.25 || gamepad.dpad.down.isPressed { pressedKeys.insert("ArrowDown") }
            if hasDirectionalInput { noteDirectionalInput() }
            if !isActionSuppressed() && (gamepad.buttonA.isPressed || gamepad.buttonX.isPressed) { pressedKeys.insert("Space") }
            syncControllerKeys(pressedKeys)
            return
        }

        if let gamepad = controller.microGamepad {
            let hasDirectionalInput = abs(gamepad.dpad.xAxis.value) > 0.25 || abs(gamepad.dpad.yAxis.value) > 0.25
            if gamepad.dpad.xAxis.value < -0.25 { pressedKeys.insert("ArrowLeft") }
            if gamepad.dpad.xAxis.value > 0.25 { pressedKeys.insert("ArrowRight") }
            if gamepad.dpad.yAxis.value > 0.25 { pressedKeys.insert("ArrowUp") }
            if gamepad.dpad.yAxis.value < -0.25 { pressedKeys.insert("ArrowDown") }
            if hasDirectionalInput { noteDirectionalInput() }
            if !isActionSuppressed() && (gamepad.buttonA.isPressed || gamepad.buttonX.isPressed) {
                pressedKeys.insert("Space")
            }
        }
        syncControllerKeys(pressedKeys)
    }

    private func noteDirectionalInput() {
        suppressActionUntil = CACurrentMediaTime() + 0.22
        handle?.keyUp("Space")
    }

    private func isActionSuppressed() -> Bool {
        !activeDirectionalPressKeys.isEmpty || CACurrentMediaTime() < suppressActionUntil
    }

    private func syncControllerKeys(_ pressedKeys: Set<String>) {
        for key in pressedKeys.subtracting(previousControllerKeys) {
            handle?.keyDown(key)
        }
        for key in previousControllerKeys.subtracting(pressedKeys) {
            handle?.keyUp(key)
        }
        previousControllerKeys = pressedKeys
    }

    private func releasePreviousControllerKeys() {
        syncControllerKeys([])
    }

    func handlePresses(_ presses: Set<UIPress>, pressed: Bool) -> Bool {
        var handled = false
        let directionalPressKeys = Set(presses.compactMap { press -> String? in
            switch press.type {
            case .upArrow:
                return "ArrowUp"
            case .downArrow:
                return "ArrowDown"
            case .leftArrow:
                return "ArrowLeft"
            case .rightArrow:
                return "ArrowRight"
            default:
                return nil
            }
        })
        if !directionalPressKeys.isEmpty {
            if pressed {
                activeDirectionalPressKeys.formUnion(directionalPressKeys)
                noteDirectionalInput()
            } else {
                activeDirectionalPressKeys.subtract(directionalPressKeys)
            }
        }
        for press in presses {
            switch press.type {
            case .select, .playPause:
                if !isActionSuppressed() {
                    pressed ? handle?.keyDown("Space") : handle?.keyUp("Space")
                } else {
                    handle?.keyUp("Space")
                }
                handled = true
            case .upArrow:
                pressed ? handle?.keyDown("ArrowUp") : handle?.keyUp("ArrowUp")
                handled = true
            case .downArrow:
                pressed ? handle?.keyDown("ArrowDown") : handle?.keyUp("ArrowDown")
                handled = true
            case .leftArrow:
                pressed ? handle?.keyDown("ArrowLeft") : handle?.keyUp("ArrowLeft")
                handled = true
            case .rightArrow:
                pressed ? handle?.keyDown("ArrowRight") : handle?.keyUp("ArrowRight")
                handled = true
            default:
                break
            }
        }
        return handled
    }

    override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if !handlePresses(presses, pressed: true) {
            super.pressesBegan(presses, with: event)
        }
    }

    override func pressesEnded(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        if !handlePresses(presses, pressed: false) {
            super.pressesEnded(presses, with: event)
        }
    }
}
#else
import AppKit
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
#endif
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

fn generated_xcode_pbxproj(
    app_display_name: &str,
    bundle_id: &str,
    development_team: Option<&str>,
) -> String {
    let escaped_name = app_display_name.replace('"', "\\\"");
    let escaped_bundle = bundle_id.replace('"', "\\\"");
    let development_team_setting = xcode_development_team_setting(development_team);
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
				{development_team_setting}
				EXTRACT_APP_INTENTS_METADATA = NO;
				GENERATE_INFOPLIST_FILE = YES;
				INFOPLIST_KEY_CFBundleDisplayName = "{escaped_name}";
				LD_RUNPATH_SEARCH_PATHS = "@executable_path/../Frameworks";
				LIBRARY_SEARCH_PATHS = (
					"$(inherited)",
					"$(SRCROOT)/RustBridge/build",
				);
				MACOSX_DEPLOYMENT_TARGET = 13.0;
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
				{development_team_setting}
				EXTRACT_APP_INTENTS_METADATA = NO;
				GENERATE_INFOPLIST_FILE = YES;
				INFOPLIST_KEY_CFBundleDisplayName = "{escaped_name}";
				LD_RUNPATH_SEARCH_PATHS = "@executable_path/../Frameworks";
				LIBRARY_SEARCH_PATHS = (
					"$(inherited)",
					"$(SRCROOT)/RustBridge/build",
				);
				MACOSX_DEPLOYMENT_TARGET = 13.0;
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

fn generated_xcode_tvos_pbxproj(
    app_display_name: &str,
    bundle_id: &str,
    development_team: Option<&str>,
) -> String {
    let escaped_name = app_display_name.replace('"', "\\\"");
    let escaped_bundle = bundle_id.replace('"', "\\\"");
    let development_team_setting = xcode_development_team_setting(development_team);
    let tvos_bundle = if escaped_bundle.ends_with(".tvos") {
        escaped_bundle.clone()
    } else {
        format!("{escaped_bundle}.tvos")
    };
    format!(
        r#"// !$*UTF8*$!
{{
	archiveVersion = 1;
	classes = {{
	}};
	objectVersion = 77;
	objects = {{

/* Begin PBXBuildFile section */
		AT0000010000000000000001 /* librpu_apple_export.a in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000001 /* librpu_apple_export.a */; }};
		AT0000010000000000000002 /* Project in Resources */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000002 /* Project */; }};
		AT0000010000000000000003 /* UIKit.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000004 /* UIKit.framework */; }};
		AT0000010000000000000004 /* Metal.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000005 /* Metal.framework */; }};
		AT0000010000000000000005 /* QuartzCore.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000006 /* QuartzCore.framework */; }};
		AT0000010000000000000006 /* Foundation.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000007 /* Foundation.framework */; }};
		AT0000010000000000000007 /* GameController.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000008 /* GameController.framework */; }};
		AT0000010000000000000008 /* AVFoundation.framework in Frameworks */ = {{isa = PBXBuildFile; fileRef = AT0001010000000000000009 /* AVFoundation.framework */; }};
/* End PBXBuildFile section */

/* Begin PBXFileReference section */
		AT0001010000000000000001 /* librpu_apple_export.a */ = {{isa = PBXFileReference; lastKnownFileType = archive.ar; path = "RustBridge/build/$(PLATFORM_NAME)/librpu_apple_export.a"; sourceTree = "<group>"; }};
		AT0001010000000000000002 /* Project */ = {{isa = PBXFileReference; lastKnownFileType = folder; path = Project; sourceTree = "<group>"; }};
		AT0001010000000000000003 /* {escaped_name}.app */ = {{isa = PBXFileReference; explicitFileType = wrapper.application; includeInIndex = 0; path = "{escaped_name}.app"; sourceTree = BUILT_PRODUCTS_DIR; }};
		AT0001010000000000000004 /* UIKit.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = UIKit.framework; path = System/Library/Frameworks/UIKit.framework; sourceTree = SDKROOT; }};
		AT0001010000000000000005 /* Metal.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = Metal.framework; path = System/Library/Frameworks/Metal.framework; sourceTree = SDKROOT; }};
		AT0001010000000000000006 /* QuartzCore.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = QuartzCore.framework; path = System/Library/Frameworks/QuartzCore.framework; sourceTree = SDKROOT; }};
		AT0001010000000000000007 /* Foundation.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = Foundation.framework; path = System/Library/Frameworks/Foundation.framework; sourceTree = SDKROOT; }};
		AT0001010000000000000008 /* GameController.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = GameController.framework; path = System/Library/Frameworks/GameController.framework; sourceTree = SDKROOT; }};
		AT0001010000000000000009 /* AVFoundation.framework */ = {{isa = PBXFileReference; lastKnownFileType = wrapper.framework; name = AVFoundation.framework; path = System/Library/Frameworks/AVFoundation.framework; sourceTree = SDKROOT; }};
/* End PBXFileReference section */

/* Begin PBXFileSystemSynchronizedRootGroup section */
		AT0002010000000000000001 /* App */ = {{
			isa = PBXFileSystemSynchronizedRootGroup;
			path = App;
			sourceTree = "<group>";
		}};
/* End PBXFileSystemSynchronizedRootGroup section */

/* Begin PBXFrameworksBuildPhase section */
		AT0003010000000000000001 /* Frameworks */ = {{
			isa = PBXFrameworksBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AT0000010000000000000003 /* UIKit.framework in Frameworks */,
				AT0000010000000000000004 /* Metal.framework in Frameworks */,
				AT0000010000000000000005 /* QuartzCore.framework in Frameworks */,
				AT0000010000000000000006 /* Foundation.framework in Frameworks */,
				AT0000010000000000000007 /* GameController.framework in Frameworks */,
				AT0000010000000000000008 /* AVFoundation.framework in Frameworks */,
				AT0000010000000000000001 /* librpu_apple_export.a in Frameworks */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXFrameworksBuildPhase section */

/* Begin PBXGroup section */
		AT0004010000000000000001 = {{
			isa = PBXGroup;
			children = (
				AT0002010000000000000001 /* App */,
				AT0001010000000000000002 /* Project */,
				AT0001010000000000000004 /* UIKit.framework */,
				AT0001010000000000000005 /* Metal.framework */,
				AT0001010000000000000006 /* QuartzCore.framework */,
				AT0001010000000000000007 /* Foundation.framework */,
				AT0001010000000000000008 /* GameController.framework */,
				AT0001010000000000000009 /* AVFoundation.framework */,
				AT0001010000000000000001 /* librpu_apple_export.a */,
				AT0004010000000000000002 /* Products */,
			);
			sourceTree = "<group>";
		}};
		AT0004010000000000000002 /* Products */ = {{
			isa = PBXGroup;
			children = (
				AT0001010000000000000003 /* {escaped_name}.app */,
			);
			name = Products;
			sourceTree = "<group>";
		}};
/* End PBXGroup section */

/* Begin PBXNativeTarget section */
		AT0005010000000000000001 /* RPUAppleTVApp */ = {{
			isa = PBXNativeTarget;
			buildConfigurationList = AT0009010000000000000001 /* Build configuration list for PBXNativeTarget "RPUAppleTVApp" */;
			buildPhases = (
				AT0006010000000000000003 /* Build Rust Bridge */,
				AT0006010000000000000001 /* Sources */,
				AT0003010000000000000001 /* Frameworks */,
				AT0006010000000000000002 /* Resources */,
			);
			buildRules = (
			);
			dependencies = (
			);
			fileSystemSynchronizedGroups = (
				AT0002010000000000000001 /* App */,
			);
			name = RPUAppleTVApp;
			productName = "{escaped_name}";
			productReference = AT0001010000000000000003 /* {escaped_name}.app */;
			productType = "com.apple.product-type.application";
		}};
/* End PBXNativeTarget section */

/* Begin PBXProject section */
		AT0007010000000000000001 /* Project object */ = {{
			isa = PBXProject;
			attributes = {{
				BuildIndependentTargetsInParallel = 1;
				LastSwiftUpdateCheck = 2610;
				LastUpgradeCheck = 2610;
				TargetAttributes = {{
					AT0005010000000000000001 = {{
						CreatedOnToolsVersion = 26.1.1;
					}};
				}};
			}};
			buildConfigurationList = AT0009010000000000000002 /* Build configuration list for PBXProject "RPUAppleTVApp" */;
			developmentRegion = en;
			hasScannedForEncodings = 0;
			knownRegions = (
				en,
				Base,
			);
			mainGroup = AT0004010000000000000001;
			minimizedProjectReferenceProxies = 1;
			preferredProjectObjectVersion = 77;
			productRefGroup = AT0004010000000000000002 /* Products */;
			projectDirPath = "";
			projectRoot = "";
			targets = (
				AT0005010000000000000001 /* RPUAppleTVApp */,
			);
		}};
/* End PBXProject section */

/* Begin PBXResourcesBuildPhase section */
		AT0006010000000000000002 /* Resources */ = {{
			isa = PBXResourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
				AT0000010000000000000002 /* Project in Resources */,
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXResourcesBuildPhase section */

/* Begin PBXShellScriptBuildPhase section */
		AT0006010000000000000003 /* Build Rust Bridge */ = {{
			isa = PBXShellScriptBuildPhase;
			alwaysOutOfDate = 1;
			buildActionMask = 2147483647;
			files = (
			);
			inputPaths = (
				"$(SRCROOT)/RustBridge/Cargo.toml",
				"$(SRCROOT)/RustBridge/src/lib.rs",
			);
			name = "Build Rust Bridge";
			outputPaths = (
				"$(SRCROOT)/RustBridge/build/$(PLATFORM_NAME)/librpu_apple_export.a",
			);
			runOnlyForDeploymentPostprocessing = 0;
			shellPath = /bin/sh;
			shellScript = "bash \"$SRCROOT/RustBridge/build-rust.sh\"\n";
		}};
/* End PBXShellScriptBuildPhase section */

/* Begin PBXSourcesBuildPhase section */
		AT0006010000000000000001 /* Sources */ = {{
			isa = PBXSourcesBuildPhase;
			buildActionMask = 2147483647;
			files = (
			);
			runOnlyForDeploymentPostprocessing = 0;
		}};
/* End PBXSourcesBuildPhase section */

/* Begin XCBuildConfiguration section */
		AT0008010000000000000001 /* Debug */ = {{
			isa = XCBuildConfiguration;
			buildSettings = {{
				ALWAYS_SEARCH_USER_PATHS = NO;
				ARCHS = arm64;
				CODE_SIGN_STYLE = Automatic;
				{development_team_setting}
				EXTRACT_APP_INTENTS_METADATA = NO;
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = "tvOS-Info.plist";
				INFOPLIST_KEY_CFBundleDisplayName = "{escaped_name}";
				LD_RUNPATH_SEARCH_PATHS = "@executable_path/Frameworks";
				LIBRARY_SEARCH_PATHS = (
					"$(inherited)",
					"$(SRCROOT)/RustBridge/build/$(PLATFORM_NAME)",
				);
				ONLY_ACTIVE_ARCH = YES;
				PRODUCT_BUNDLE_IDENTIFIER = "{tvos_bundle}";
				PRODUCT_NAME = "{escaped_name}";
				SDKROOT = appletvos;
				SUPPORTED_PLATFORMS = "appletvos appletvsimulator";
				SWIFT_VERSION = 5.0;
				TARGETED_DEVICE_FAMILY = 3;
				TVOS_DEPLOYMENT_TARGET = 15.0;
			}};
			name = Debug;
		}};
		AT0008010000000000000002 /* Release */ = {{
			isa = XCBuildConfiguration;
			buildSettings = {{
				ALWAYS_SEARCH_USER_PATHS = NO;
				ARCHS = arm64;
				CODE_SIGN_STYLE = Automatic;
				{development_team_setting}
				EXTRACT_APP_INTENTS_METADATA = NO;
				GENERATE_INFOPLIST_FILE = NO;
				INFOPLIST_FILE = "tvOS-Info.plist";
				INFOPLIST_KEY_CFBundleDisplayName = "{escaped_name}";
				LD_RUNPATH_SEARCH_PATHS = "@executable_path/Frameworks";
				LIBRARY_SEARCH_PATHS = (
					"$(inherited)",
					"$(SRCROOT)/RustBridge/build/$(PLATFORM_NAME)",
				);
				ONLY_ACTIVE_ARCH = YES;
				PRODUCT_BUNDLE_IDENTIFIER = "{tvos_bundle}";
				PRODUCT_NAME = "{escaped_name}";
				SDKROOT = appletvos;
				SUPPORTED_PLATFORMS = "appletvos appletvsimulator";
				SWIFT_VERSION = 5.0;
				TARGETED_DEVICE_FAMILY = 3;
				TVOS_DEPLOYMENT_TARGET = 15.0;
			}};
			name = Release;
		}};
/* End XCBuildConfiguration section */

/* Begin XCConfigurationList section */
		AT0009010000000000000001 /* Build configuration list for PBXNativeTarget "RPUAppleTVApp" */ = {{
			isa = XCConfigurationList;
			buildConfigurations = (
				AT0008010000000000000001 /* Debug */,
				AT0008010000000000000002 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		}};
		AT0009010000000000000002 /* Build configuration list for PBXProject "RPUAppleTVApp" */ = {{
			isa = XCConfigurationList;
			buildConfigurations = (
				AT0008010000000000000001 /* Debug */,
				AT0008010000000000000002 /* Release */,
			);
			defaultConfigurationIsVisible = 0;
			defaultConfigurationName = Release;
		}};
/* End XCConfigurationList section */
	}};
	rootObject = AT0007010000000000000001 /* Project object */;
}}
"#,
        escaped_name = escaped_name,
        tvos_bundle = tvos_bundle,
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
        let Ok(output) = Command::new(candidate).arg("--version").output() else {
            continue;
        };
        if output.status.success()
            && parse_wasm_bindgen_version(&output.stdout) == Some(WASM_BINDGEN_VERSION)
        {
            return Some(candidate.to_string());
        }
    }
    None
}

fn parse_wasm_bindgen_version(output: &[u8]) -> Option<&str> {
    std::str::from_utf8(output).ok()?.split_whitespace().last()
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
            "wasm-bindgen CLI {WASM_BINDGEN_VERSION} is required for web export.\nInstall or update it with:\n  cargo install wasm-bindgen-cli --version {WASM_BINDGEN_VERSION} --locked --force"
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

    ensure_xcodebuild_available()?;

    ensure_command_available(
        "cargo",
        "Cargo is required to build the generated Rust bridge. Install Rust from https://www.rust-lang.org/tools/install",
    )?;

    Ok(())
}

fn ensure_xcodebuild_available() -> Result<()> {
    let status = Command::new("xcodebuild").arg("-version").status();
    match status {
        Ok(status) if status.success() => Ok(()),
        _ => bail!(
            "Xcode export requires Xcode and the command line tools. Install Xcode from the App Store, then run `xcode-select --install` if needed."
        ),
    }
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
