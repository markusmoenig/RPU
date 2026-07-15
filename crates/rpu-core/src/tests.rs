use super::*;

fn source_file(path: &str, contents: &str) -> SourceFile {
    SourceFile {
        relative_path: PathBuf::from(path),
        contents: contents.to_string(),
        modified: None,
    }
}

#[test]
fn manifest_parses_capability_requirements() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "cart_demo"

[requires]
system = true
graphics = true
audio = false
network = false
"#,
    )
    .expect("manifest should parse");

    assert!(manifest.requires.system);
    assert!(manifest.requires.graphics);
    assert!(!manifest.requires.audio);
    assert!(!manifest.requires.network);
}

#[test]
fn manifest_ignores_legacy_fine_grained_capability_fields() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "legacy_cli"

[requires]
graphics = false
audio = false
input = false
resources = true
time = true
window = false
ui = false
storage = false
network = false
"#,
    )
    .expect("manifest should parse");

    assert!(manifest.requires.system);
    assert!(!manifest.requires.graphics);
    assert!(!manifest.requires.audio);
    assert!(!manifest.requires.network);
}

#[test]
fn manifest_uses_legacy_default_capabilities_when_requires_is_missing() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "legacy_project"
"#,
    )
    .expect("manifest should parse");

    assert_eq!(manifest.requires, CapabilityConfig::default());
    assert_eq!(manifest.build, BuildConfig::default());
}

#[test]
fn manifest_parses_rpu_wasm_build_target() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "shared_abi_demo"
kind = "cli"

[build]
language = "rpu"
backend = "wasm"

[requires]
system = true
graphics = false
audio = false
network = false
"#,
    )
    .expect("manifest should parse");

    assert_eq!(manifest.build.language, SourceLanguage::Rpu);
    assert_eq!(manifest.build.backend, BuildBackend::Wasm);
}

#[test]
fn manifest_parses_cartridge_kind_and_modules() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "mesh_tools"
kind = "cli"

[[modules]]
name = "simplify"
backend = "wasm"
path = "modules/simplify.wasm"

[[modules]]
name = "quick_script"
backend = "bytecode"
path = "bytecode/quick_script.rpubc"
"#,
    )
    .expect("manifest should parse");

    assert_eq!(manifest.project.kind, ProjectKind::Cli);
    assert_eq!(manifest.modules.len(), 2);
    assert_eq!(manifest.modules[0].backend, ModuleBackend::Wasm);
    assert_eq!(
        manifest.modules[0].path,
        PathBuf::from("modules/simplify.wasm")
    );
    assert_eq!(manifest.modules[1].backend, ModuleBackend::Bytecode);
}

#[test]
fn built_cartridge_manifest_is_language_neutral() {
    let manifest: BuiltCartridgeManifest = toml::from_str(
        r#"
[cartridge]
format_version = 1
abi_version = 1

[project]
name = "hello_c"
version = "0.1.0"
kind = "cli"

[entry]
backend = "wasm"
path = "main.wasm"

[requires]
system = true
graphics = false
audio = false
network = false
"#,
    )
    .expect("built cartridge manifest should parse");

    assert_eq!(manifest.cartridge.format_version, CARTRIDGE_FORMAT_VERSION);
    assert_eq!(manifest.cartridge.abi_version, wasm_abi::ABI_VERSION);
    assert_eq!(manifest.project.kind, ProjectKind::Cli);
    assert_eq!(manifest.entry.backend, BuildBackend::Wasm);
    assert_eq!(manifest.entry.path, PathBuf::from("main.wasm"));
}

#[test]
fn cartridge_paths_cannot_escape_the_bundle() {
    assert!(validate_cartridge_relative_path(Path::new("main.wasm"), "entry").is_ok());
    assert!(validate_cartridge_relative_path(Path::new("modules/tool.wasm"), "module").is_ok());
    assert!(validate_cartridge_relative_path(Path::new("../main.wasm"), "entry").is_err());
    assert!(validate_cartridge_relative_path(Path::new("./main.wasm"), "entry").is_err());
    assert!(validate_cartridge_relative_path(Path::new("/tmp/main.wasm"), "entry").is_err());
}

#[test]
fn cartridge_module_names_must_be_unique_and_nonempty() {
    let module = |name: &str| ModuleConfig {
        name: name.to_string(),
        backend: ModuleBackend::Wasm,
        path: PathBuf::from(format!("modules/{name}.wasm")),
    };

    assert!(validate_module_names(&[module("one"), module("two")]).is_ok());
    assert!(validate_module_names(&[module("same"), module("same")]).is_err());
    assert!(validate_module_names(&[module("  ")]).is_err());
}

#[test]
fn cli_cartridges_compile_without_scene_files() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "hello_cli"
kind = "cli"

[requires]
system = true
graphics = false
audio = false
network = false
"#,
    )
    .expect("manifest should parse");
    let script = source_file(
        "scripts/main.rpu",
        r#"
on run() {
    print("Hello from CLI")

    if arg_count() > 0 {
        print(arg(0))
    }
}
"#,
    );

    let compiled = compile_project_sources(&manifest, Vec::new(), vec![script], Vec::new(), 1)
        .expect("CLI cartridge should compile");

    assert!(!compiled.has_errors());
    assert_eq!(compiled.kind, ProjectKind::Cli);
    assert!(compiled.parsed_scenes.is_empty());
    assert_eq!(compiled.bytecode_scripts[0].handlers[0].event, "run");
}

#[test]
fn rpu_wasm_build_backend_warns_until_implemented() {
    let manifest: ProjectManifest = toml::from_str(
        r#"
[project]
name = "hello_cli"
kind = "cli"

[build]
language = "rpu"
backend = "wasm"

[requires]
system = true
graphics = false
audio = false
network = false
"#,
    )
    .expect("manifest should parse");
    let script = source_file(
        "scripts/main.rpu",
        r#"
on run() {
    print("Hello from future WASM")
}
"#,
    );

    let compiled = compile_project_sources(&manifest, Vec::new(), vec![script], Vec::new(), 1)
        .expect("CLI cartridge should compile");

    assert!(!compiled.has_errors());
    assert_eq!(compiled.build.backend, BuildBackend::Wasm);
    assert!(compiled.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::Warning
            && diagnostic.message.contains("WASM build backend")
    }));
}

#[test]
fn wasm_abi_defines_cli_lifecycle_exports() {
    let exports = wasm_abi::required_exports_for_kind(ProjectKind::Cli);
    let names = exports.iter().map(|export| export.name).collect::<Vec<_>>();

    assert_eq!(wasm_abi::ABI_VERSION, 1);
    assert!(names.contains(&wasm_abi::EXPORT_ABI_VERSION));
    assert!(names.contains(&wasm_abi::EXPORT_ALLOC));
    assert!(names.contains(&wasm_abi::EXPORT_DEALLOC));
    assert!(names.contains(&wasm_abi::EXPORT_RUN));
    assert!(!names.contains(&wasm_abi::EXPORT_UPDATE));

    let run = exports
        .iter()
        .find(|export| export.name == wasm_abi::EXPORT_RUN)
        .expect("CLI WASM cartridges require rpu_run");
    assert_eq!(run.params, &[]);
    assert_eq!(run.results, &[wasm_abi::WasmValueType::I32]);
}

#[test]
fn wasm_abi_defines_app_lifecycle_exports() {
    let exports = wasm_abi::required_exports_for_kind(ProjectKind::App);
    let names = exports.iter().map(|export| export.name).collect::<Vec<_>>();

    assert!(names.contains(&wasm_abi::EXPORT_START));
    assert!(names.contains(&wasm_abi::EXPORT_UPDATE));
    assert!(names.contains(&wasm_abi::EXPORT_STOP));
    assert!(!names.contains(&wasm_abi::EXPORT_RUN));

    let update = exports
        .iter()
        .find(|export| export.name == wasm_abi::EXPORT_UPDATE)
        .expect("app WASM cartridges require rpu_update");
    assert_eq!(update.params, &[wasm_abi::WasmValueType::F32]);
    assert_eq!(update.results, &[wasm_abi::WasmValueType::I32]);
}

#[test]
fn wasm_abi_system_capability_imports_first_cli_surface() {
    let requires = CapabilityConfig {
        system: true,
        graphics: false,
        audio: false,
        network: false,
    };
    let imports = wasm_abi::required_imports_for_capabilities(&requires);
    let names = imports.iter().map(|import| import.name).collect::<Vec<_>>();

    assert!(
        imports
            .iter()
            .all(|import| import.module == Some(wasm_abi::SYSTEM_IMPORT_MODULE))
    );
    assert!(names.contains(&wasm_abi::IMPORT_ARG_COUNT));
    assert!(names.contains(&wasm_abi::IMPORT_ARG_LEN));
    assert!(names.contains(&wasm_abi::IMPORT_ARG_READ));
    assert!(names.contains(&wasm_abi::IMPORT_PRINT));
    assert!(names.contains(&wasm_abi::IMPORT_EPRINT));
    assert!(names.contains(&wasm_abi::IMPORT_EXIT));
    assert!(names.contains(&wasm_abi::IMPORT_NOW_MS));

    let arg_read = imports
        .iter()
        .find(|import| import.name == wasm_abi::IMPORT_ARG_READ)
        .expect("system imports include arg_read");
    assert_eq!(
        arg_read.params,
        &[
            wasm_abi::WasmValueType::I32,
            wasm_abi::WasmValueType::I32,
            wasm_abi::WasmValueType::I32
        ]
    );
    assert_eq!(arg_read.results, &[wasm_abi::WasmValueType::I32]);
}

#[test]
fn wasm_abi_respects_missing_system_capability() {
    let requires = CapabilityConfig {
        system: false,
        graphics: false,
        audio: false,
        network: false,
    };

    assert!(wasm_abi::required_imports_for_capabilities(&requires).is_empty());
}

#[test]
fn wasm_abi_graphics_capability_imports_first_frame_surface() {
    let requires = CapabilityConfig {
        system: false,
        graphics: true,
        audio: false,
        network: false,
    };
    let imports = wasm_abi::required_imports_for_capabilities(&requires);
    let names = imports.iter().map(|import| import.name).collect::<Vec<_>>();

    assert!(
        imports
            .iter()
            .all(|import| import.module == Some(wasm_abi::GRAPHICS_IMPORT_MODULE))
    );
    assert!(names.contains(&wasm_abi::IMPORT_GRAPHICS_BEGIN_FRAME));
    assert!(names.contains(&wasm_abi::IMPORT_GRAPHICS_CLEAR));
    assert!(names.contains(&wasm_abi::IMPORT_GRAPHICS_DRAW_RECT));
    assert!(names.contains(&wasm_abi::IMPORT_GRAPHICS_END_FRAME));

    let draw_rect = imports
        .iter()
        .find(|import| import.name == wasm_abi::IMPORT_GRAPHICS_DRAW_RECT)
        .expect("graphics imports include draw_rect");
    assert_eq!(draw_rect.params, &[wasm_abi::WasmValueType::F32; 8]);
    assert!(draw_rect.results.is_empty());
}

#[test]
fn scene_parser_supports_inline_visual_scripts() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    rect Hero {
        color = #ff4455

        fn wrap_x(next_x) {
            return next_x
        }

        on update(dt) {
            self.x = wrap_x(self.x - 12.0 * dt)
        }
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let hero = &parsed.scenes[0].rects[0];
    assert_eq!(
        hero.visual.script_binding.as_deref(),
        Some("__inline__/scenes_main_rpu__Main__Hero.rpu")
    );
    assert!(
        hero.visual
            .inline_script
            .as_deref()
            .unwrap_or_default()
            .contains("on update(dt)")
    );

    let generated = collect_inline_script_sources(&[parsed], &[scene], &[]);
    assert_eq!(generated.len(), 1);
    assert_eq!(
        generated[0].relative_path,
        PathBuf::from("scripts/__inline__/scenes_main_rpu__Main__Hero.rpu")
    );
    assert!(generated[0].contents.contains("fn wrap_x"));
}

#[test]
fn scene_parser_supports_template_visuals() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite EnemyTemplate {
        visible = false
        template = true
        texture = "enemy.png"
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    assert!(!sprite.visual.visible);
    assert!(sprite.visual.template);
}

#[test]
fn scene_parser_supports_visual_groups() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite EnemyTemplate {
        template = true
        group = "hostile"
        texture = "enemy.png"
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    assert_eq!(sprite.visual.group.as_deref(), Some("hostile"));
}

#[test]
fn scene_parser_supports_sprite_scroll_and_repeat() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite Stars {
        texture = "bg-stars.png"
        scroll = (-8.0, 0.0)
        repeat_x = true
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    assert_eq!(sprite.scroll, [-8.0, 0.0]);
    assert!(sprite.repeat_x);
    assert!(!sprite.repeat_y);
}

#[test]
fn scene_parser_supports_sprite_texture_animation() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite Bullet {
        texture = ["shoot1.png", "shoot2.png"]
        animation_fps = 18.0
        animation_mode = "once"
        destroy_on_animation_end = true
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    assert_eq!(sprite.textures, vec!["shoot1.png", "shoot2.png"]);
    assert_eq!(sprite.animation_fps, 18.0);
    assert_eq!(sprite.animation_mode, AnimationMode::Once);
    assert!(sprite.destroy_on_animation_end);
}

#[test]
fn scene_parser_supports_sprite_animation_blocks() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite Player {
        texture = "idle1.png"

        animation idle {
            frames = ["idle1.png", "idle2.png"]
            fps = 2.0
            loop = true
        }

        animation hurt {
            frames = "hurt.png"
            fps = 1.0
            mode = once
        }
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    let idle = sprite.animations.get("idle").expect("idle animation");
    assert_eq!(idle.textures, vec!["idle1.png", "idle2.png"]);
    assert_eq!(idle.fps, 2.0);
    assert_eq!(idle.mode, AnimationMode::Loop);
    let hurt = sprite.animations.get("hurt").expect("hurt animation");
    assert_eq!(hurt.textures, vec!["hurt.png"]);
    assert_eq!(hurt.fps, 1.0);
    assert_eq!(hurt.mode, AnimationMode::Once);
}

#[test]
fn scene_parser_supports_text_nodes() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    text Score {
        pos = (12, 8)
        value = "SCORE 000000"
        font = "BetterPixels.ttf"
        font_size = 16.0
        color = #f4f8ff
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let text = &parsed.scenes[0].texts[0];
    assert_eq!(text.value, "SCORE 000000");
    assert_eq!(text.font, "BetterPixels.ttf");
    assert_eq!(text.font_size, 16.0);
}

#[test]
fn scene_parser_supports_anchor_and_text_align() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite Logo {
        anchor = top
        pos = (0, 12)
        texture = "logo.png"
    }

    text Title {
        anchor = top_right
        align = right
        pos = (-8, 8)
        value = "RPU"
        font = "BetterPixels.ttf"
        font_size = 20.0
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    let text = &parsed.scenes[0].texts[0];
    assert_eq!(sprite.visual.anchor, Anchor::Top);
    assert_eq!(text.visual.anchor, Anchor::TopRight);
    assert_eq!(text.align, TextAlign::Right);
}

#[test]
fn sprite_size_defaults_from_texture_when_omitted() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    sprite Player {
        texture = "player.png"
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let mut parsed = parse_scene_document(&scene, &mut diagnostics);
    let example_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/warped_space_shooter");
    let asset_path = example_root.join("assets/player.png");
    let asset_bytes = std::fs::read(&asset_path).expect("player asset should exist");
    let assets = vec![BundledAsset {
        relative_path: PathBuf::from("assets/player.png"),
        bytes: asset_bytes,
    }];
    resolve_sprite_texture_sizes_from_assets(
        &assets,
        std::slice::from_mut(&mut parsed),
        &mut diagnostics,
    );

    assert!(diagnostics.is_empty());
    let sprite = &parsed.scenes[0].sprites[0];
    assert_eq!(sprite.visual.size, [26.0, 21.0]);
    assert!(!sprite.visual.size_explicit);
}

#[test]
fn expression_parser_respects_operator_precedence() {
    let expr = parse_expr("Mascot.x - 12.0 * dt").expect("expression should parse");

    match expr {
        Expr::Binary(left, BinaryOp::Sub, right) => {
            match *left {
                Expr::Target(ScriptTarget::NamedEntity(name, ScriptProperty::X)) => {
                    assert_eq!(name, "Mascot");
                }
                other => panic!("unexpected left expr: {other:?}"),
            }
            match *right {
                Expr::Binary(mult_left, BinaryOp::Mul, mult_right) => {
                    assert!(
                        matches!(*mult_left, Expr::Number(value) if (value - 12.0).abs() < f32::EPSILON)
                    );
                    assert!(matches!(*mult_right, Expr::Dt));
                }
                other => panic!("unexpected right expr: {other:?}"),
            }
        }
        other => panic!("unexpected expr shape: {other:?}"),
    }
}

#[test]
fn expression_parser_supports_unary_minus_on_targets() {
    let expr = parse_expr("-self.width - 14.0").expect("expression should parse");

    match expr {
        Expr::Binary(left, BinaryOp::Sub, right) => {
            assert!(matches!(*right, Expr::Number(value) if (value - 14.0).abs() < f32::EPSILON));
            match *left {
                Expr::Binary(inner_left, BinaryOp::Sub, inner_right) => {
                    assert!(
                        matches!(*inner_left, Expr::Number(value) if value.abs() < f32::EPSILON)
                    );
                    assert!(matches!(
                        *inner_right,
                        Expr::Target(ScriptTarget::SelfEntity(ScriptProperty::Width))
                    ));
                }
                other => panic!("unexpected unary-minus left expr: {other:?}"),
            }
        }
        other => panic!("unexpected expr shape: {other:?}"),
    }
}

#[test]
fn expression_parser_supports_string_call_args() {
    let expr = parse_expr(r#"key("Space")"#).expect("expression should parse");

    match expr {
        Expr::Call(name, args) => {
            assert_eq!(name, "key");
            assert!(matches!(&args[0], Expr::String(value) if value == "Space"));
        }
        other => panic!("unexpected expr shape: {other:?}"),
    }
}

#[test]
fn script_target_supports_sprite_texture_property() {
    let target = parse_script_target("self.texture").expect("texture target should parse");
    assert!(matches!(
        target,
        ScriptTarget::SelfEntity(ScriptProperty::Texture)
    ));
}

#[test]
fn script_target_supports_text_property() {
    let target = parse_script_target("self.text").expect("text target should parse");
    assert!(matches!(
        target,
        ScriptTarget::SelfEntity(ScriptProperty::Text)
    ));
}

#[test]
fn script_target_supports_named_state_property() {
    let target = parse_script_target("HudState.score").expect("state target should parse");
    assert!(matches!(
        target,
        ScriptTarget::NamedEntity(name, ScriptProperty::State(property))
            if name == "HudState" && property == "score"
    ));
}

#[test]
fn condition_parser_supports_boolean_composition() {
    let condition = parse_condition("next_x < 120.0 || (Accent.x < 260.0 && !(self.y < 200.0))")
        .expect("condition should parse");

    match condition {
        Condition::Or(left, right) => {
            assert!(matches!(
                *left,
                Condition::Compare {
                    left: Expr::Variable(ref name),
                    op: CompareOp::Less,
                    ..
                } if name == "next_x"
            ));
            match *right {
                Condition::And(_, negated) => {
                    assert!(matches!(*negated, Condition::Not(_)));
                }
                other => panic!("unexpected right condition: {other:?}"),
            }
        }
        other => panic!("unexpected condition shape: {other:?}"),
    }
}

#[test]
fn condition_parser_supports_truthy_calls() {
    let condition = parse_condition(r#"key("Space")"#).expect("condition should parse");

    match condition {
        Condition::Compare {
            left: Expr::Call(name, args),
            op: CompareOp::NotEqual,
            right: Expr::Number(value),
        } => {
            assert_eq!(name, "key");
            assert!(matches!(&args[0], Expr::String(key) if key == "Space"));
            assert_eq!(value, 0.0);
        }
        other => panic!("unexpected condition shape: {other:?}"),
    }
}

#[test]
fn script_compiler_supports_functions_params_locals_returns_and_calls() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
fn sync(limit) {
    if limit < 10.0 {
        return 10.0
    } else {
        return limit
    }
}

on update(dt) {
    let next_x = self.x - 12.0 * dt
    self.x = sync(next_x)
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);

    assert!(diagnostics.is_empty());
    assert_eq!(compiled.functions.len(), 1);
    assert_eq!(compiled.functions[0].name, "sync");
    assert_eq!(compiled.functions[0].params, vec!["limit"]);
    assert_eq!(compiled.handlers.len(), 1);

    let handler = &compiled.handlers[0];
    assert_eq!(handler.event, "update");
    assert_eq!(handler.params, vec!["dt"]);
    assert!(matches!(
        &handler.ops[0].op,
        OpCode::Let(name, Expr::Binary(_, BinaryOp::Sub, _)) if name == "next_x"
    ));
    assert_eq!(handler.ops[0].line, 11);
    assert!(matches!(
        &handler.ops[1].op,
        OpCode::Assign(
            ScriptTarget::SelfEntity(ScriptProperty::X),
            Expr::Call(name, args)
        ) if name == "sync" && matches!(&args[0], Expr::Variable(var) if var == "next_x")
    ));
    assert_eq!(handler.ops[1].line, 12);

    let function = &compiled.functions[0];
    assert_eq!(function.ops[0].line, 3);
    match &function.ops[0].op {
        OpCode::If(condition, body, _) => {
            assert!(!body.is_empty());
            assert_eq!(body[0].line, 4);
            assert!(
                matches!(&body[0].op, OpCode::Return(Expr::Number(value)) if (*value - 10.0).abs() < f32::EPSILON)
            );
            match condition {
                Condition::Compare {
                    left: Expr::Variable(name),
                    op: CompareOp::Less,
                    right: Expr::Number(value),
                } => {
                    assert_eq!(name, "limit");
                    assert!((*value - 10.0).abs() < f32::EPSILON);
                }
                other => panic!("unexpected function condition: {other:?}"),
            }
        }
        other => panic!("unexpected function op: {other:?}"),
    }
}

#[test]
fn script_compiler_supports_event_handler_params_and_direct_calls() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
on event(event, value) {
    if event == "motion" && value == "idle" {
        self.texture = "idle.png"
    }
}

on update(dt) {
    emit("motion", "idle")
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);

    assert!(diagnostics.is_empty());
    assert_eq!(compiled.handlers.len(), 2);
    assert_eq!(compiled.handlers[0].event, "event");
    assert_eq!(compiled.handlers[0].params, vec!["event", "value"]);
    assert!(matches!(
        &compiled.handlers[1].ops[0].op,
        OpCode::Call(name, args)
            if name == "emit"
                && matches!(&args[0], Expr::String(value) if value == "motion")
                && matches!(&args[1], Expr::String(value) if value == "idle")
    ));
}

#[test]
fn script_compiler_supports_state_declarations_and_assignments() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
state score = 0

on update(dt) {
    let _ = dt
    score = score + 10.0
    self.score = score
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);

    assert!(diagnostics.is_empty());
    assert_eq!(compiled.state.len(), 1);
    assert_eq!(compiled.state[0].name, "score");
    assert_eq!(compiled.state[0].line, 2);
    assert!(matches!(&compiled.state[0].init, Expr::Number(value) if *value == 0.0));
    assert!(matches!(
        &compiled.handlers[0].ops[1].op,
        OpCode::StateSet(name, Expr::Binary(_, BinaryOp::Add, _)) if name == "score"
    ));
    assert!(matches!(
        &compiled.handlers[0].ops[2].op,
        OpCode::Assign(
            ScriptTarget::SelfEntity(ScriptProperty::State(property)),
            Expr::Variable(name)
        ) if property == "score" && name == "score"
    ));
}

#[test]
fn script_compiler_supports_else_if_chains() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
on update(dt) {
    if time() < 0.75 {
        self.x = 0.0
    } else if time() < 2.0 {
        self.x = 1.0
    } else {
        self.x = 2.0
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);

    assert!(diagnostics.is_empty());
    let ops = &compiled.handlers[0].ops;
    assert_eq!(ops.len(), 1);
    match &ops[0].op {
        OpCode::If(_, then_body, else_body) => {
            assert_eq!(then_body.len(), 1);
            assert_eq!(else_body.len(), 1);
            match &else_body[0].op {
                OpCode::If(_, nested_then, nested_else) => {
                    assert_eq!(nested_then.len(), 1);
                    assert_eq!(nested_else.len(), 1);
                    assert!(matches!(
                        &nested_else[0].op,
                        OpCode::Assign(ScriptTarget::SelfEntity(ScriptProperty::X), Expr::Number(value))
                            if (*value - 2.0).abs() < f32::EPSILON
                    ));
                }
                other => panic!("unexpected else-if op: {other:?}"),
            }
        }
        other => panic!("unexpected top-level op: {other:?}"),
    }
}

#[test]
fn script_compiler_supports_spawn_and_destroy() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
on update(dt) {
    spawn("EnemyTemplate", "EnemyA", screen_width() + 40.0, 120.0)
    destroy("EnemyA")
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);
    assert!(diagnostics.is_empty());

    let ops = &compiled.handlers[0].ops;
    assert!(matches!(
        &ops[0].op,
        OpCode::Spawn(template, Some(name), _, _)
            if template == "EnemyTemplate" && name == "EnemyA"
    ));
    assert!(matches!(
        &ops[1].op,
        OpCode::Destroy(DestroyTarget::Named(name)) if name == "EnemyA"
    ));
}

#[test]
fn script_compiler_supports_dynamic_destroy_targets() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
on update(dt) {
    let hit = first_overlap("hostile")
    destroy(hit)
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);
    assert!(diagnostics.is_empty());

    let ops = &compiled.handlers[0].ops;
    assert!(matches!(
        &ops[1].op,
        OpCode::DestroyExpr(Expr::Variable(name)) if name == "hit"
    ));
}

#[test]
fn script_compiler_supports_auto_named_spawn() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
on update(dt) {
    spawn("EnemyTemplate", screen_width() + 40.0, 120.0)
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);
    assert!(diagnostics.is_empty());
    assert!(matches!(
        &compiled.handlers[0].ops[0].op,
        OpCode::Spawn(template, None, _, _) if template == "EnemyTemplate"
    ));
}

#[test]
fn compatibility_builtins_still_compile_to_specific_opcodes() {
    assert!(matches!(
        compile_op(r#"move_by_dt("Mascot", -12.0, 0.0)"#),
        OpCode::MoveByDtTarget(name, delta)
            if name == "Mascot" && delta == [-12.0, 0.0]
    ));
    assert!(matches!(
        compile_op("set_color(#ff4455)"),
        OpCode::SetColor(color)
            if color == [1.0, 68.0 / 255.0, 85.0 / 255.0, 1.0]
    ));
}

#[test]
fn compatibility_builtins_take_precedence_over_generic_calls_in_scripts() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
on ready() {
    log("ready")
    move_by(4.0, 2.0)
    set_color(#ff4455)
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);
    let ops = &compiled.handlers[0].ops;

    assert!(diagnostics.is_empty());
    assert!(matches!(&ops[0].op, OpCode::Log(message) if message == "ready"));
    assert!(matches!(&ops[1].op, OpCode::MoveBy(delta) if *delta == [4.0, 2.0]));
    assert!(matches!(&ops[2].op, OpCode::SetColor(_)));
}

#[test]
fn invalid_function_signature_emits_diagnostic() {
    let script = source_file(
        "scripts/main.rpu",
        r#"
fn broken( {
}
"#,
    );

    let mut diagnostics = Vec::new();
    let compiled = compile_script(&script, &mut diagnostics);

    assert!(compiled.handlers.is_empty());
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "invalid function signature" && diagnostic.line == Some(2)
    }));
}
