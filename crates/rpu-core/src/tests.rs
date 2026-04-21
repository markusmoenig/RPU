use super::*;

fn source_file(path: &str, contents: &str) -> SourceFile {
    SourceFile {
        relative_path: PathBuf::from(path),
        contents: contents.to_string(),
        modified: None,
    }
}

#[test]
fn scene_parser_supports_maps_symbols_and_hex_colors() {
    let scene = source_file(
        "scenes/main.rpu",
        r#"
scene Main {
    map Terrain {
        origin = (64, 96)
        cell = (16, 16)

        legend {
            x = marker
            # = #ff4455
        }

        ascii {
            x#
        }
    }

    rect Hero {
        color = #00ffaa
    }

    sprite Player {
        symbol = x
        size = (32, 48)
    }
}
"#,
    );

    let mut diagnostics = Vec::new();
    let parsed = parse_scene_document(&scene, &mut diagnostics);

    assert!(diagnostics.is_empty());
    assert_eq!(parsed.scenes.len(), 1);
    let scene = &parsed.scenes[0];
    assert_eq!(scene.maps.len(), 1);
    assert_eq!(scene.rects[0].visual.color, [0.0, 1.0, 170.0 / 255.0, 1.0]);
    assert_eq!(scene.sprites[0].symbol.as_deref(), Some("x"));

    let commands = compile_scene_draw_commands(&[parsed.clone()]);
    assert_eq!(commands.len(), 3);

    match &commands[0] {
        DrawCommand::Rect(rect) => {
            assert_eq!(rect.x, 80.0);
            assert_eq!(rect.y, 96.0);
            assert_eq!(rect.width, 16.0);
            assert_eq!(rect.color, [1.0, 68.0 / 255.0, 85.0 / 255.0, 1.0]);
        }
        other => panic!("expected map rect, got {other:?}"),
    }

    match &commands[2] {
        DrawCommand::Sprite(sprite) => {
            assert_eq!(sprite.x, 64.0);
            assert_eq!(sprite.y, 96.0);
            assert_eq!(sprite.width, 32.0);
            assert_eq!(sprite.height, 48.0);
        }
        other => panic!("expected sprite, got {other:?}"),
    }
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
    assert!(hero
        .visual
        .inline_script
        .as_deref()
        .unwrap_or_default()
        .contains("on update(dt)"));

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
    resolve_sprite_texture_sizes(
        &example_root,
        &[PathBuf::from("assets/player.png")],
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
                    assert!(matches!(*mult_left, Expr::Number(value) if (value - 12.0).abs() < f32::EPSILON));
                    assert!(matches!(*mult_right, Expr::Dt));
                }
                other => panic!("unexpected right expr: {other:?}"),
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
fn condition_parser_supports_boolean_composition() {
    let condition =
        parse_condition("next_x < 120.0 || (Accent.x < 260.0 && !(self.y < 200.0))")
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
            assert!(matches!(&body[0].op, OpCode::Return(Expr::Number(value)) if (*value - 10.0).abs() < f32::EPSILON));
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
