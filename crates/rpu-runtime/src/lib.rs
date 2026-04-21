use anyhow::Result;
use rpu_core::{
    AnimationMode, AsciiMapNode, BinaryOp, BytecodeOp, CompareOp, CompiledProject, Condition,
    DestroyTarget, DrawCommand, Expr, MapLegendMeaning, OpCode, RectNode, ResizeMode,
    RpuProject, SceneCamera, SceneRect, ScriptProperty, ScriptTarget, SpriteNode, TextNode,
    WindowConfig,
};
use rpu_scenevm::{run_app, RpuSceneApp, RuntimeContext, SceneFrame};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn run(project: RpuProject) -> Result<()> {
    run_app(RpuRuntimeApp::new(project)?)
}

struct RpuRuntimeApp {
    session: RuntimeSession,
}

impl RpuRuntimeApp {
    fn new(project: RpuProject) -> Result<Self> {
        Ok(Self {
            session: RuntimeSession::new(project)?,
        })
    }
}

impl RpuSceneApp for RpuRuntimeApp {
    fn window_title(&self) -> Option<String> {
        Some(format!("RPU - {}", self.session.compiled.name))
    }

    fn initial_window_size(&self) -> Option<(u32, u32)> {
        let window = &self.session.compiled.window;
        Some((
            (window.width as f32 * window.default_scale.max(0.1)).round() as u32,
            (window.height as f32 * window.default_scale.max(0.1)).round() as u32,
        ))
    }

    fn init(&mut self, _ctx: &mut RuntimeContext) {
        self.session.mark_initialized(_ctx);
    }

    fn update(&mut self, _ctx: &mut RuntimeContext) {
        self.session.tick(_ctx);
    }

    fn render(&mut self, _ctx: &mut RuntimeContext, frame: &mut SceneFrame) {
        frame.clear_color(self.current_clear_color());
        let camera = self.session.compiled.active_camera();
        let viewport = frame.size();
        let view = RenderView::new(&self.session.compiled.window, viewport);

        for (index, command) in self.session.world.static_draw_commands.iter().enumerate() {
            submit_draw_command(
                frame,
                &camera,
                &view,
                index as i32,
                command,
                &self.session.project,
                self.session.query_state.elapsed_time,
            );
        }

        let base_order = self.session.world.static_draw_commands.len() as i32;
        for (index, entity) in self.session.world.entities.iter().enumerate() {
            if !entity.visible {
                continue;
            }
            let stable_order = base_order + index as i32;
            let (x, y, width, height) = world_to_screen(
                &camera,
                &view,
                entity.pos[0],
                entity.pos[1],
                entity.size[0],
                entity.size[1],
            );
            match &entity.kind {
                RuntimeEntityKind::Rect => {
                    frame.push_rect(
                        entity.layer,
                        entity.z * 1000 + stable_order,
                        x,
                        y,
                        width,
                        height,
                        entity.color,
                    );
                }
                RuntimeEntityKind::Sprite {
                    texture,
                    frames,
                    animation_fps,
                    animation_mode,
                    ..
                } => {
                    let texture_name = current_texture_frame(
                        texture.as_deref(),
                        frames,
                        *animation_fps,
                        *animation_mode,
                        self.session.query_state.elapsed_time,
                        entity.spawn_time,
                    );
                    let texture_path = texture_name
                        .as_deref()
                        .map(|texture| self.session.project.root().join("assets").join(texture))
                        .map(pathbuf_to_string);
                    submit_sprite(
                        frame,
                        &camera,
                        &view,
                        entity.layer,
                        entity.z * 1000 + stable_order,
                        entity.pos[0],
                        entity.pos[1],
                        entity.size[0],
                        entity.size[1],
                        entity.color,
                        entity.scroll,
                        entity.repeat_x,
                        entity.repeat_y,
                        entity.flip_x,
                        entity.flip_y,
                        texture_path.as_deref(),
                        self.session.query_state.elapsed_time,
                    );
                }
                RuntimeEntityKind::Text {
                    value,
                    font,
                    font_size,
                } => {
                    let font_path = self
                        .session
                        .project
                        .root()
                        .join("assets")
                        .join(font);
                    let (x, y, _, _) = world_to_screen(
                        &camera,
                        &view,
                        entity.pos[0],
                        entity.pos[1],
                        0.0,
                        0.0,
                    );
                    frame.push_text(
                        entity.layer,
                        entity.z * 1000 + stable_order,
                        x,
                        y,
                        value,
                        &pathbuf_to_string(font_path),
                        *font_size,
                        entity.color,
                    );
                }
            }
        }
    }
}

struct RuntimeSession {
    project: RpuProject,
    compiled: CompiledProject,
    world: RuntimeWorld,
    initialized: bool,
    ticks: u64,
    last_reload_poll: Instant,
    last_tick_instant: Instant,
    start_instant: Instant,
    query_state: RuntimeQueryState,
    spawn_serial: u64,
    every_state: HashMap<String, f32>,
    rand_state: HashMap<String, f32>,
    rng_state: u64,
}

struct RuntimeWorld {
    static_draw_commands: Vec<DrawCommand>,
    templates: Vec<RuntimeEntity>,
    entities: Vec<RuntimeEntity>,
}

#[derive(Clone)]
struct RuntimeEntity {
    name: String,
    template: bool,
    visible: bool,
    spawn_time: f32,
    group: Option<String>,
    layer: i32,
    z: i32,
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    scroll: [f32; 2],
    repeat_x: bool,
    repeat_y: bool,
    flip_x: bool,
    flip_y: bool,
    script: Option<String>,
    script_state: HashMap<String, Value>,
    kind: RuntimeEntityKind,
}

#[derive(Clone)]
enum RuntimeEntityKind {
    Rect,
    Sprite {
        texture: Option<String>,
        frames: Vec<String>,
        animation_fps: f32,
        animation_mode: AnimationMode,
        destroy_on_animation_end: bool,
    },
    Text {
        value: String,
        font: String,
        font_size: f32,
    },
}

#[derive(Clone)]
enum Value {
    Scalar(f32),
    Vec2([f32; 2]),
    Color([f32; 4]),
    String(String),
}

enum ExecSignal {
    Continue,
    Stop,
    Return(Value),
}

#[derive(Default)]
struct RuntimeQueryState {
    window_size: (u32, u32),
    input_left: bool,
    input_right: bool,
    input_up: bool,
    input_down: bool,
    input_action: bool,
    pressed_keys: HashMap<String, bool>,
    elapsed_time: f32,
}

struct RenderView {
    virtual_size: (f32, f32),
    actual_size: (f32, f32),
    scale: f32,
    offset: (f32, f32),
    resize: ResizeMode,
}

impl RuntimeSession {
    fn new(project: RpuProject) -> Result<Self> {
        let compiled = project.compile()?;
        log_compilation("initial compile", &compiled);
        let world = RuntimeWorld::from_compiled(&compiled);
        let mut session = Self {
            project,
            compiled,
            world,
            initialized: false,
            ticks: 0,
            last_reload_poll: Instant::now(),
            last_tick_instant: Instant::now(),
            start_instant: Instant::now(),
            query_state: RuntimeQueryState::default(),
            spawn_serial: 0,
            every_state: HashMap::new(),
            rand_state: HashMap::new(),
            rng_state: 0x9E3779B97F4A7C15,
        };
        session.initialize_script_state_all();
        Ok(session)
    }

    fn mark_initialized(&mut self, ctx: &RuntimeContext) {
        self.refresh_query_state(ctx);
        self.initialized = true;
        self.execute_event("ready", 0.0);
        self.last_tick_instant = Instant::now();
    }

    fn tick(&mut self, ctx: &RuntimeContext) {
        self.refresh_query_state(ctx);
        let now = Instant::now();
        let dt = (now - self.last_tick_instant).as_secs_f32().min(0.1);
        self.last_tick_instant = now;

        if self.initialized {
            self.ticks = self.ticks.saturating_add(1);
            self.execute_event("update", dt);
            self.world
                .remove_finished_animations(self.query_state.elapsed_time);
        }
        self.maybe_reload();
    }

    fn refresh_query_state(&mut self, ctx: &RuntimeContext) {
        self.query_state.window_size = (
            self.compiled.window.width.max(1),
            self.compiled.window.height.max(1),
        );
        self.query_state.input_left = ctx.input_left();
        self.query_state.input_right = ctx.input_right();
        self.query_state.input_up = ctx.input_up();
        self.query_state.input_down = ctx.input_down();
        self.query_state.input_action = ctx.input_action();
        self.query_state.pressed_keys = ctx
            .pressed_keys()
            .into_iter()
            .map(|key| (key, true))
            .collect();
        self.query_state.elapsed_time = self.start_instant.elapsed().as_secs_f32();
    }

    fn next_spawn_name(&mut self, template: &str) -> String {
        self.spawn_serial = self.spawn_serial.saturating_add(1);
        format!("{template}_{}", self.spawn_serial)
    }

    fn next_random_unit(&mut self) -> f32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let bits = (self.rng_state >> 32) as u32;
        bits as f32 / u32::MAX as f32
    }

    fn random_between(&mut self, min: f32, max: f32) -> f32 {
        let low = min.min(max);
        let high = min.max(max);
        if (high - low).abs() < f32::EPSILON {
            return low;
        }
        low + self.next_random_unit() * (high - low)
    }

    fn random_between_spread(&mut self, key: &str, min: f32, max: f32) -> f32 {
        let low = min.min(max);
        let high = min.max(max);
        if (high - low).abs() < f32::EPSILON {
            return low;
        }

        let mut phase = self
            .rand_state
            .get(key)
            .copied()
            .unwrap_or_else(|| self.next_random_unit());
        phase = (phase + 0.618_033_95).fract();

        // Add a little true randomness so repeated calls do not look too patterned.
        let jitter = (self.next_random_unit() - 0.5) * 0.16;
        let sample = (phase + jitter).clamp(0.0, 1.0);
        self.rand_state.insert(key.to_string(), phase);
        low + sample * (high - low)
    }

    fn initialize_script_state_all(&mut self) {
        let names: Vec<String> = self
            .world
            .entities
            .iter()
            .map(|entity| entity.name.clone())
            .collect();
        for name in names {
            if let Some(index) = self.world.find_entity_index(&name) {
                self.initialize_entity_state(index);
            }
        }
    }

    fn initialize_entity_state(&mut self, entity_index: usize) {
        let Some(entity) = self.world.entities.get(entity_index) else {
            return;
        };
        let Some(script_name) = entity.script.as_deref() else {
            return;
        };
        let script_path = PathBuf::from("scripts").join(script_name);
        let Some(script) = self
            .compiled
            .bytecode_scripts
            .iter()
            .find(|script| script.path == script_path)
            .cloned()
        else {
            return;
        };
        let entity_name = entity.name.clone();
        let mut locals = HashMap::new();
        for state in script.state {
            let Some(value) = self.eval_expr(
                entity_index,
                &entity_name,
                &script_path,
                "state",
                Some(state.line),
                &state.init,
                0.0,
                &mut locals,
                0,
            ) else {
                eprintln!(
                    "rpu: script warning: failed to initialize state `{}` in {}:{} for {}",
                    state.name,
                    script_path.display(),
                    state.line,
                    entity_name,
                );
                continue;
            };
            self.write_state(entity_index, &state.name, value);
        }
    }

    fn maybe_reload(&mut self) {
        if self.last_reload_poll.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_reload_poll = Instant::now();

        match self
            .project
            .has_source_changes_since(self.compiled.fingerprint.latest_modified)
        {
            Ok(true) => match self.project.compile() {
                Ok(compiled) => {
                    log_compilation("hot reload", &compiled);
                    self.compiled = compiled;
                    self.world = RuntimeWorld::from_compiled(&self.compiled);
                    self.initialize_script_state_all();
                    if self.initialized {
                        self.execute_event("ready", 0.0);
                    }
                }
                Err(error) => eprintln!("rpu: reload failed: {error:#}"),
            },
            Ok(false) => {}
            Err(error) => eprintln!("rpu: failed to poll project changes: {error:#}"),
        }
    }

    fn execute_event(&mut self, event: &str, dt: f32) {
        let mut scheduled = Vec::new();
        for (entity_index, entity) in self.world.entities.iter().enumerate() {
            let Some(script_name) = entity.script.as_deref() else {
                continue;
            };
            let script_path = PathBuf::from("scripts").join(script_name);
            let Some(script) = self
                .compiled
                .bytecode_scripts
                .iter()
                .find(|script| script.path == script_path)
            else {
                continue;
            };

            for handler in script.handlers.iter().filter(|handler| handler.event == event) {
                scheduled.push((
                    entity_index,
                    entity.name.clone(),
                    script.path.clone(),
                    handler.ops.clone(),
                ));
            }
        }

        for (_entity_index, entity_name, script_path, ops) in scheduled {
            let Some(current_index) = self.world.find_entity_index(&entity_name) else {
                continue;
            };
            let mut locals = HashMap::new();
            self.apply_ops(
                current_index,
                &entity_name,
                &script_path,
                event,
                &ops,
                dt,
                &mut locals,
                0,
            );
        }
    }

    fn execute_entity_event(&mut self, entity_index: usize, event: &str, dt: f32) {
        let Some(entity) = self.world.entities.get(entity_index) else {
            return;
        };
        let Some(script_name) = entity.script.as_deref() else {
            return;
        };
        let script_path = PathBuf::from("scripts").join(script_name);
        let Some(script) = self
            .compiled
            .bytecode_scripts
            .iter()
            .find(|script| script.path == script_path)
        else {
            return;
        };
        let ops: Vec<BytecodeOp> = script
            .handlers
            .iter()
            .filter(|handler| handler.event == event)
            .flat_map(|handler| handler.ops.clone())
            .collect();
        if ops.is_empty() {
            return;
        }
        let entity_name = entity.name.clone();
        let mut locals = HashMap::new();
        let _ = self.apply_ops(
            entity_index,
            &entity_name,
            &script_path,
            event,
            &ops,
            dt,
            &mut locals,
            0,
        );
    }

    fn apply_ops(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        ops: &[BytecodeOp],
        dt: f32,
        locals: &mut HashMap<String, Value>,
        call_depth: usize,
    ) -> ExecSignal {
        for op in ops {
            let signal = self.apply_op(
                entity_index,
                entity_name,
                script_path,
                event,
                op,
                dt,
                locals,
                call_depth,
            );
            match signal {
                ExecSignal::Continue => {}
                ExecSignal::Stop => return ExecSignal::Stop,
                ExecSignal::Return(_) => return signal,
            }
        }
        ExecSignal::Continue
    }

    fn apply_op(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        op: &BytecodeOp,
        dt: f32,
        locals: &mut HashMap<String, Value>,
        call_depth: usize,
    ) -> ExecSignal {
        match &op.op {
            OpCode::Log(message) => {
                eprintln!("rpu: script: {entity_name}: {message}");
                ExecSignal::Continue
            }
            OpCode::IgnoreValue(_) => ExecSignal::Continue,
            OpCode::Call(name, args) => {
                let _ = self.invoke_function(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    Some(op.line),
                    name,
                    args,
                    dt,
                    locals,
                    call_depth,
                );
                ExecSignal::Continue
            }
            OpCode::Return(expr) => {
                let Some(value) = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    Some(op.line),
                    expr,
                    dt,
                    locals,
                    call_depth,
                ) else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate return value in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                ExecSignal::Return(value)
            }
            OpCode::Let(name, expr) => {
                let Some(value) = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    Some(op.line),
                    expr,
                    dt,
                    locals,
                    call_depth,
                ) else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate local binding in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                locals.insert(name.clone(), value);
                ExecSignal::Continue
            }
            OpCode::StateSet(name, expr) => {
                let Some(value) = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    Some(op.line),
                    expr,
                    dt,
                    locals,
                    call_depth,
                ) else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate state assignment in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                self.write_state(entity_index, name, value);
                ExecSignal::Continue
            }
            OpCode::Assign(target, expr) => {
                let Some(value) = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    Some(op.line),
                    expr,
                    dt,
                    locals,
                    call_depth,
                ) else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate assignment in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                if !self.assign_target(entity_index, target, value) {
                    eprintln!(
                        "rpu: script warning: invalid assignment target in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                ExecSignal::Continue
            }
            OpCode::If(condition, body, else_body) => {
                if self
                    .eval_condition(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        Some(op.line),
                        condition,
                        dt,
                        locals,
                        call_depth,
                    )
                    .unwrap_or(false)
                {
                    self.apply_ops(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        body,
                        dt,
                        locals,
                        call_depth,
                    )
                } else {
                    self.apply_ops(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        else_body,
                        dt,
                        locals,
                        call_depth,
                    )
                }
            }
            OpCode::Spawn(template, name, x, y) => {
                let Some(x) = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        Some(op.line),
                        x,
                        dt,
                        locals,
                        call_depth,
                    )
                    .and_then(|value| value.as_scalar())
                else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate spawn x in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                let Some(y) = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        Some(op.line),
                        y,
                        dt,
                        locals,
                        call_depth,
                    )
                    .and_then(|value| value.as_scalar())
                else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate spawn y in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                let instance_name = name.clone().unwrap_or_else(|| self.next_spawn_name(template));
                if let Some(spawned_index) = self.world.spawn_from_template(
                    template,
                    &instance_name,
                    [x, y],
                    self.query_state.elapsed_time,
                )
                {
                    self.initialize_entity_state(spawned_index);
                    if self.initialized {
                        self.execute_entity_event(spawned_index, "ready", 0.0);
                    }
                } else {
                    eprintln!(
                        "rpu: script warning: missing template `{}` in {}:{} for {}:{}",
                        template,
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                ExecSignal::Continue
            }
            OpCode::Destroy(target) => {
                let removed = match target {
                    DestroyTarget::SelfEntity => self.world.remove_entity_named(entity_name),
                    DestroyTarget::Named(name) => self.world.remove_entity_named(name),
                };
                if !removed {
                    eprintln!(
                        "rpu: script warning: failed to destroy target in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                match target {
                    DestroyTarget::SelfEntity => ExecSignal::Stop,
                    DestroyTarget::Named(_) => ExecSignal::Continue,
                }
            }
            OpCode::DestroyExpr(expr) => {
                let Some(name) = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        Some(op.line),
                        expr,
                        dt,
                        locals,
                        call_depth,
                    )
                    .and_then(|value| value.as_string().map(ToOwned::to_owned))
                else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate destroy target in {}:{} for {}:{}",
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                let _ = self.world.remove_entity_named(&name);
                ExecSignal::Continue
            }
            OpCode::MoveBy(delta) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                entity.pos[0] += delta[0];
                entity.pos[1] += delta[1];
                ExecSignal::Continue
            }
            OpCode::MoveByDt(velocity) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                entity.pos[0] += velocity[0] * dt;
                entity.pos[1] += velocity[1] * dt;
                ExecSignal::Continue
            }
            OpCode::SetPos(pos) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                entity.pos = *pos;
                ExecSignal::Continue
            }
            OpCode::SetColor(color) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                entity.color = *color;
                ExecSignal::Continue
            }
            OpCode::CopyPos(target) => {
                let Some(pos) = self.world.find_entity(target).map(|entity| entity.pos) else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {}:{} for {}:{}",
                        target,
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                    return ExecSignal::Continue;
                };
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                entity.pos = pos;
                ExecSignal::Continue
            }
            OpCode::ClampX(range) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                let min = range[0].min(range[1]);
                let max = range[0].max(range[1]);
                entity.pos[0] = entity.pos[0].clamp(min, max);
                ExecSignal::Continue
            }
            OpCode::ClampY(range) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return ExecSignal::Continue;
                };
                let min = range[0].min(range[1]);
                let max = range[0].max(range[1]);
                entity.pos[1] = entity.pos[1].clamp(min, max);
                ExecSignal::Continue
            }
            OpCode::MoveByTarget(target, delta) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.pos[0] += delta[0];
                    entity.pos[1] += delta[1];
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {}:{} for {}:{}",
                        target,
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                ExecSignal::Continue
            }
            OpCode::MoveByDtTarget(target, velocity) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.pos[0] += velocity[0] * dt;
                    entity.pos[1] += velocity[1] * dt;
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {}:{} for {}:{}",
                        target,
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                ExecSignal::Continue
            }
            OpCode::SetPosTarget(target, pos) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.pos = *pos;
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {}:{} for {}:{}",
                        target,
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                ExecSignal::Continue
            }
            OpCode::SetColorTarget(target, color) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.color = *color;
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {}:{} for {}:{}",
                        target,
                        script_path.display(),
                        op.line,
                        entity_name,
                        event
                    );
                }
                ExecSignal::Continue
            }
            OpCode::Raw(raw) => {
                eprintln!(
                    "rpu: script warning: unsupported opcode `{}` in {}:{} for {}:{}",
                    raw,
                    script_path.display(),
                    op.line,
                    entity_name,
                    event
                );
                ExecSignal::Continue
            }
        }
    }

    fn invoke_function(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        line: Option<usize>,
        name: &str,
        args: &[Expr],
        dt: f32,
        locals: &mut HashMap<String, Value>,
        call_depth: usize,
    ) -> Option<Option<Value>> {
        if call_depth >= 32 {
            eprintln!(
                "rpu: script warning: call depth exceeded in {}{} for {}:{}",
                script_path.display(),
                format_line_suffix(line),
                entity_name,
                event
            );
            return None;
        }
        let Some(script) = self
            .compiled
            .bytecode_scripts
            .iter()
            .find(|script| script.path == script_path)
        else {
            eprintln!(
                "rpu: script warning: missing script context {}{} for {}:{}",
                script_path.display(),
                format_line_suffix(line),
                entity_name,
                event
            );
            return None;
        };
        let Some(function) = script.functions.iter().find(|function| function.name == name) else {
            eprintln!(
                "rpu: script warning: missing function `{}` in {}{} for {}:{}",
                name,
                script_path.display(),
                format_line_suffix(line),
                entity_name,
                event
            );
            return None;
        };
        let function_params = function.params.clone();
        let function_ops = function.ops.clone();

        if function_params.len() != args.len() {
            eprintln!(
                "rpu: script warning: function `{}` expected {} args but got {} in {}{} for {}:{}",
                name,
                function_params.len(),
                args.len(),
                script_path.display(),
                format_line_suffix(line),
                entity_name,
                event
            );
            return None;
        }

        let mut arg_values = Vec::with_capacity(args.len());
        for arg in args {
            let Some(value) = self.eval_expr(
                entity_index,
                entity_name,
                script_path,
                event,
                line,
                arg,
                dt,
                locals,
                call_depth,
            ) else {
                eprintln!(
                    "rpu: script warning: failed to evaluate function arg for `{}` in {}{} for {}:{}",
                    name,
                    script_path.display(),
                    format_line_suffix(line),
                    entity_name,
                    event
                );
                return None;
            };
            arg_values.push(value);
        }

        let mut saved = Vec::with_capacity(function_params.len());
        for (param, value) in function_params.iter().zip(arg_values.into_iter()) {
            saved.push((param.clone(), locals.insert(param.clone(), value)));
        }

        let signal = self.apply_ops(
            entity_index,
            entity_name,
            script_path,
            event,
            &function_ops,
            dt,
            locals,
            call_depth + 1,
        );

        for (param, old_value) in saved.into_iter().rev() {
            if let Some(old_value) = old_value {
                locals.insert(param, old_value);
            } else {
                locals.remove(&param);
            }
        }

        Some(match signal {
            ExecSignal::Continue => None,
            ExecSignal::Stop => None,
            ExecSignal::Return(value) => Some(value),
        })
    }

    fn eval_condition(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        line: Option<usize>,
        condition: &Condition,
        dt: f32,
        locals: &mut HashMap<String, Value>,
        call_depth: usize,
    ) -> Option<bool> {
        Some(match condition {
            Condition::Compare { left, op, right } => {
                let left = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, left, dt, locals, call_depth)?
                    .as_scalar()?;
                let right = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, right, dt, locals, call_depth)?
                    .as_scalar()?;
                match op {
                    CompareOp::Less => left < right,
                    CompareOp::LessEqual => left <= right,
                    CompareOp::Greater => left > right,
                    CompareOp::GreaterEqual => left >= right,
                    CompareOp::Equal => (left - right).abs() < f32::EPSILON,
                    CompareOp::NotEqual => (left - right).abs() >= f32::EPSILON,
                }
            }
            Condition::And(left, right) => {
                self.eval_condition(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    left,
                    dt,
                    locals,
                    call_depth,
                )? && self.eval_condition(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    right,
                    dt,
                    locals,
                    call_depth,
                )?
            }
            Condition::Or(left, right) => {
                self.eval_condition(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    left,
                    dt,
                    locals,
                    call_depth,
                )? || self.eval_condition(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    right,
                    dt,
                    locals,
                    call_depth,
                )?
            }
            Condition::Not(inner) => !self.eval_condition(
                entity_index,
                entity_name,
                script_path,
                event,
                line,
                inner,
                dt,
                locals,
                call_depth,
            )?,
        })
    }

    fn eval_expr(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        line: Option<usize>,
        expr: &Expr,
        dt: f32,
        locals: &mut HashMap<String, Value>,
        call_depth: usize,
    ) -> Option<Value> {
        match expr {
            Expr::Number(value) => Some(Value::Scalar(*value)),
            Expr::Dt => Some(Value::Scalar(dt)),
            Expr::String(value) => Some(Value::String(value.clone())),
            Expr::Variable(name) => locals
                .get(name)
                .cloned()
                .or_else(|| self.read_state(entity_index, name)),
            Expr::Call(name, args) => self
                .eval_builtin_query(entity_index, entity_name, script_path, event, line, name, args, dt, locals, call_depth)
                .or_else(|| {
                    self.invoke_function(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        name,
                        args,
                        dt,
                        locals,
                        call_depth,
                    )
                    .and_then(|value| value)
                }),
            Expr::Target(target) => self.read_target(entity_index, target),
            Expr::Color(color) => Some(Value::Color(*color)),
            Expr::Binary(left, op, right) => {
                let left = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, left, dt, locals, call_depth)?
                    .as_scalar()?;
                let right = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, right, dt, locals, call_depth)?
                    .as_scalar()?;
                let value = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    BinaryOp::Div => left / right,
                };
                Some(Value::Scalar(value))
            }
            Expr::Clamp(value, min, max) => {
                let value = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, value, dt, locals, call_depth)?
                    .as_scalar()?;
                let min = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, min, dt, locals, call_depth)?
                    .as_scalar()?;
                let max = self
                    .eval_expr(entity_index, entity_name, script_path, event, line, max, dt, locals, call_depth)?
                    .as_scalar()?;
                Some(Value::Scalar(value.clamp(min.min(max), min.max(max))))
            }
        }
    }

    fn eval_builtin_query(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        line: Option<usize>,
        name: &str,
        args: &[Expr],
        dt: f32,
        locals: &mut HashMap<String, Value>,
        call_depth: usize,
    ) -> Option<Value> {
        match name {
            "input_left" if args.is_empty() => Some(Value::Scalar(self.query_state.input_left as i32 as f32)),
            "input_right" if args.is_empty() => Some(Value::Scalar(self.query_state.input_right as i32 as f32)),
            "input_up" if args.is_empty() => Some(Value::Scalar(self.query_state.input_up as i32 as f32)),
            "input_down" if args.is_empty() => Some(Value::Scalar(self.query_state.input_down as i32 as f32)),
            "input_action" if args.is_empty() => Some(Value::Scalar(self.query_state.input_action as i32 as f32)),
            "exists" if args.len() == 1 => {
                let name = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?;
                let name = name.as_string()?;
                Some(Value::Scalar(self.world.find_entity(name).is_some() as i32 as f32))
            }
            "first_overlap" if args.len() == 1 => {
                let group = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?;
                let group = group.as_string()?;
                let Some(source) = self.world.entities.get(entity_index) else {
                    return None;
                };
                let hit = self
                    .world
                    .entities
                    .iter()
                    .enumerate()
                    .find(|(index, entity)| {
                        *index != entity_index
                            && entity.visible
                            && entity.group.as_deref() == Some(group)
                            && intersects(source, entity)
                    })
                    .map(|(_, entity)| entity.name.clone())
                    .unwrap_or_default();
                Some(Value::String(hit))
            }
            "time" if args.is_empty() => Some(Value::Scalar(self.query_state.elapsed_time)),
            "age" if args.is_empty() => {
                let entity = self.world.entities.get(entity_index)?;
                Some(Value::Scalar((self.query_state.elapsed_time - entity.spawn_time).max(0.0)))
            }
            "lerp" if args.len() == 3 => {
                let a = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let b = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[1],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let t = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[2],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                Some(Value::Scalar(a + (b - a) * t))
            }
            "pulse" if args.len() == 1 => {
                let period = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let period = period.max(0.001);
                let phase = (self.query_state.elapsed_time / period) * std::f32::consts::TAU;
                Some(Value::Scalar(0.5 + 0.5 * phase.sin()))
            }
            "smoothstep" if args.len() == 3 => {
                let edge0 = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let edge1 = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[1],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let x = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[2],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let t = if (edge1 - edge0).abs() < f32::EPSILON {
                    0.0
                } else {
                    ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0)
                };
                Some(Value::Scalar(t * t * (3.0 - 2.0 * t)))
            }
            "alpha" if args.len() == 2 => {
                let mut color = match self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )? {
                    Value::Color(color) => color,
                    _ => return None,
                };
                let alpha = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[1],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                color[3] = alpha.clamp(0.0, 1.0);
                Some(Value::Color(color))
            }
            "format_int" if args.len() == 2 => {
                let value = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let digits = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[1],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let digits = digits.max(1.0).floor() as usize;
                let value = value.max(0.0).floor() as i32;
                Some(Value::String(format!("{value:0digits$}")))
            }
            "difficulty" if args.is_empty() => {
                Some(Value::Scalar(1.0 + (self.query_state.elapsed_time / 12.0).floor()))
            }
            "screen_width" if args.is_empty() => Some(Value::Scalar(self.query_state.window_size.0 as f32)),
            "screen_height" if args.is_empty() => Some(Value::Scalar(self.query_state.window_size.1 as f32)),
            "rand" if args.len() == 2 => {
                let min = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let max = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[1],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let key = line
                    .map(|line| format!("{}:{}:{}:{}:rand", script_path.display(), entity_name, event, line))
                    .unwrap_or_else(|| format!("{}:{}:{}:rand", script_path.display(), entity_name, event));
                Some(Value::Scalar(self.random_between_spread(&key, min, max)))
            }
            "every" if args.len() == 1 || args.len() == 2 => {
                let min_interval = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?.as_scalar()?;
                let max_interval = if args.len() == 2 {
                    self.eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?.as_scalar()?
                } else {
                    min_interval
                };
                let next_interval = min_interval.min(max_interval);
                if next_interval <= 0.0 {
                    return Some(Value::Scalar(0.0));
                }
                let line = line?;
                let key = format!("{}:{}:{}:{}", script_path.display(), entity_name, event, line);
                let now = self.query_state.elapsed_time;
                let ready = match self.every_state.get(&key).copied() {
                    Some(next_fire) if now < next_fire => false,
                    _ => true,
                };
                if ready {
                    let interval = if args.len() == 2 {
                        self.random_between(min_interval, max_interval)
                    } else {
                        min_interval
                    };
                    self.every_state.insert(key, now + interval.max(0.01));
                }
                Some(Value::Scalar(ready as i32 as f32))
            }
            "key" if args.len() == 1 => {
                let value = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[0],
                    dt,
                    locals,
                    call_depth,
                )?;
                let key = value.as_string()?;
                let pressed = self
                    .query_state
                    .pressed_keys
                    .get(&normalize_runtime_key(key))
                    .copied()
                    .unwrap_or(false);
                Some(Value::Scalar(pressed as i32 as f32))
            }
            _ => None,
        }
    }

    fn assign_target(&mut self, entity_index: usize, target: &ScriptTarget, value: Value) -> bool {
        let entity = match target {
            ScriptTarget::SelfEntity(_) => self.world.entities.get_mut(entity_index),
            ScriptTarget::NamedEntity(name, _) => self.world.find_entity_mut(name),
        };
        let Some(entity) = entity else {
            return false;
        };

        match (target_property(target), value) {
            (ScriptProperty::X, Value::Scalar(value)) => entity.pos[0] = value,
            (ScriptProperty::Y, Value::Scalar(value)) => entity.pos[1] = value,
            (ScriptProperty::Width, Value::Scalar(value)) => entity.size[0] = value,
            (ScriptProperty::Height, Value::Scalar(value)) => entity.size[1] = value,
            (ScriptProperty::Pos, Value::Vec2(value)) => entity.pos = value,
            (ScriptProperty::Size, Value::Vec2(value)) => entity.size = value,
            (ScriptProperty::Color, Value::Color(value)) => entity.color = value,
            (ScriptProperty::Texture, Value::String(value)) => match &mut entity.kind {
                RuntimeEntityKind::Sprite {
                    texture,
                    frames,
                    animation_fps,
                    animation_mode,
                    destroy_on_animation_end,
                } => {
                    *texture = Some(value.clone());
                    *frames = vec![value];
                    *animation_fps = 0.0;
                    *animation_mode = AnimationMode::Loop;
                    *destroy_on_animation_end = false;
                }
                RuntimeEntityKind::Rect | RuntimeEntityKind::Text { .. } => return false,
            },
            (ScriptProperty::Text, Value::String(value)) => match &mut entity.kind {
                RuntimeEntityKind::Text { value: current, .. } => *current = value,
                RuntimeEntityKind::Rect | RuntimeEntityKind::Sprite { .. } => return false,
            },
            (ScriptProperty::State(name), value) => {
                entity.script_state.insert(name.clone(), value);
            }
            _ => return false,
        }

        true
    }

    fn read_target(&self, entity_index: usize, target: &ScriptTarget) -> Option<Value> {
        let entity = match target {
            ScriptTarget::SelfEntity(_) => self.world.entities.get(entity_index),
            ScriptTarget::NamedEntity(name, _) => self.world.find_entity(name),
        }?;

        Some(match target_property(target) {
            ScriptProperty::X => Value::Scalar(entity.pos[0]),
            ScriptProperty::Y => Value::Scalar(entity.pos[1]),
            ScriptProperty::Width => Value::Scalar(entity.size[0]),
            ScriptProperty::Height => Value::Scalar(entity.size[1]),
            ScriptProperty::Pos => Value::Vec2(entity.pos),
            ScriptProperty::Size => Value::Vec2(entity.size),
            ScriptProperty::Color => Value::Color(entity.color),
            ScriptProperty::Texture => match &entity.kind {
                RuntimeEntityKind::Sprite { texture, .. } => {
                    Value::String(texture.clone().unwrap_or_default())
                }
                RuntimeEntityKind::Rect | RuntimeEntityKind::Text { .. } => return None,
            },
            ScriptProperty::Text => match &entity.kind {
                RuntimeEntityKind::Text { value, .. } => Value::String(value.clone()),
                RuntimeEntityKind::Rect | RuntimeEntityKind::Sprite { .. } => return None,
            },
            ScriptProperty::State(name) => entity.script_state.get(name)?.clone(),
        })
    }

    fn read_state(&self, entity_index: usize, name: &str) -> Option<Value> {
        self.world
            .entities
            .get(entity_index)?
            .script_state
            .get(name)
            .cloned()
    }

    fn write_state(&mut self, entity_index: usize, name: &str, value: Value) {
        if let Some(entity) = self.world.entities.get_mut(entity_index) {
            entity.script_state.insert(name.to_string(), value);
        }
    }
}

impl RuntimeWorld {
    fn from_compiled(compiled: &CompiledProject) -> Self {
        let mut static_draw_commands = Vec::new();
        let mut templates = Vec::new();
        let mut entities = Vec::new();

        for document in &compiled.parsed_scenes {
            for scene in &document.scenes {
                static_draw_commands.extend(compile_static_map_commands(&scene.maps));
                let markers = compile_map_markers(&scene.maps);
                for rect in &scene.rects {
                    let entity = runtime_rect_entity(rect);
                    if entity.template {
                        templates.push(entity);
                    } else {
                        entities.push(entity);
                    }
                }
                for sprite in &scene.sprites {
                    let entity = runtime_sprite_entity(sprite, &markers);
                    if entity.template {
                        templates.push(entity);
                    } else {
                        entities.push(entity);
                    }
                }
                for text in &scene.texts {
                    let entity = runtime_text_entity(text);
                    if entity.template {
                        templates.push(entity);
                    } else {
                        entities.push(entity);
                    }
                }
            }
        }

        Self {
            static_draw_commands,
            templates,
            entities,
        }
    }

    fn find_entity_mut(&mut self, name: &str) -> Option<&mut RuntimeEntity> {
        self.entities.iter_mut().find(|entity| entity.name == name)
    }

    fn find_entity(&self, name: &str) -> Option<&RuntimeEntity> {
        self.entities.iter().find(|entity| entity.name == name)
    }

    fn find_entity_index(&self, name: &str) -> Option<usize> {
        self.entities.iter().position(|entity| entity.name == name)
    }

    fn find_template(&self, name: &str) -> Option<&RuntimeEntity> {
        self.templates.iter().find(|entity| entity.name == name)
    }

    fn spawn_from_template(
        &mut self,
        template: &str,
        name: &str,
        pos: [f32; 2],
        spawn_time: f32,
    ) -> Option<usize> {
        let template = self.find_template(template)?.clone();
        self.remove_entity_named(name);
        let mut entity = template;
        entity.name = name.to_string();
        entity.template = false;
        entity.visible = true;
        entity.pos = pos;
        entity.spawn_time = spawn_time;
        self.entities.push(entity);
        Some(self.entities.len() - 1)
    }

    fn remove_entity_named(&mut self, name: &str) -> bool {
        let original = self.entities.len();
        self.entities.retain(|entity| entity.name != name);
        self.entities.len() != original
    }

    fn remove_finished_animations(&mut self, elapsed_time: f32) {
        self.entities.retain(|entity| match &entity.kind {
            RuntimeEntityKind::Rect => true,
            RuntimeEntityKind::Text { .. } => true,
            RuntimeEntityKind::Sprite {
                frames,
                animation_fps,
                animation_mode,
                destroy_on_animation_end,
                ..
            } => {
                if !*destroy_on_animation_end
                    || *animation_mode != AnimationMode::Once
                    || *animation_fps <= 0.0
                    || frames.len() <= 1
                {
                    return true;
                }
                let duration = frames.len() as f32 / *animation_fps;
                elapsed_time - entity.spawn_time < duration
            }
        });
    }

}

impl Value {
    fn as_scalar(&self) -> Option<f32> {
        match self {
            Value::Scalar(value) => Some(*value),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(value) => Some(value.as_str()),
            _ => None,
        }
    }
}

impl RenderView {
    fn new(window: &WindowConfig, actual_size: (u32, u32)) -> Self {
        let virtual_size = (window.width.max(1) as f32, window.height.max(1) as f32);
        let actual = (actual_size.0.max(1) as f32, actual_size.1.max(1) as f32);
        let scale_x = actual.0 / virtual_size.0;
        let scale_y = actual.1 / virtual_size.1;
        let scale = match window.resize {
            ResizeMode::Letterbox => scale_x.min(scale_y),
            ResizeMode::Stretch => 1.0,
        };
        let offset = match window.resize {
            ResizeMode::Letterbox => (
                (actual.0 - virtual_size.0 * scale) * 0.5,
                (actual.1 - virtual_size.1 * scale) * 0.5,
            ),
            ResizeMode::Stretch => (0.0, 0.0),
        };

        Self {
            virtual_size,
            actual_size: actual,
            scale,
            offset,
            resize: window.resize,
        }
    }

    fn map_rect(&self, x: f32, y: f32, width: f32, height: f32) -> (f32, f32, f32, f32) {
        match self.resize {
            ResizeMode::Letterbox => (
                self.offset.0 + x * self.scale,
                self.offset.1 + y * self.scale,
                width * self.scale,
                height * self.scale,
            ),
            ResizeMode::Stretch => (
                x * (self.actual_size.0 / self.virtual_size.0),
                y * (self.actual_size.1 / self.virtual_size.1),
                width * (self.actual_size.0 / self.virtual_size.0),
                height * (self.actual_size.1 / self.virtual_size.1),
            ),
        }
    }
}

fn format_line_suffix(line: Option<usize>) -> String {
    line.map(|line| format!(":{line}")).unwrap_or_default()
}

fn normalize_runtime_key(key: &str) -> String {
    match key.trim() {
        "Left" => "ArrowLeft".to_string(),
        "Right" => "ArrowRight".to_string(),
        "Up" => "ArrowUp".to_string(),
        "Down" => "ArrowDown".to_string(),
        "Space" => "Space".to_string(),
        "Enter" => "Enter".to_string(),
        "Shift" => "Shift".to_string(),
        "Control" => "Control".to_string(),
        "Alt" => "Alt".to_string(),
        "Escape" => "Escape".to_string(),
        other => other.to_uppercase(),
    }
}

fn target_property(target: &ScriptTarget) -> &ScriptProperty {
    match target {
        ScriptTarget::SelfEntity(property) => property,
        ScriptTarget::NamedEntity(_, property) => property,
    }
}

impl RpuRuntimeApp {
    fn current_clear_color(&self) -> [f32; 4] {
        let camera = self.session.compiled.active_camera();
        let mut base = camera.background;
        if camera.background == SceneCamera::default().background {
            let named = color_from_name(&self.session.compiled.name);
            base = [named[0], named[1], named[2], 1.0];
        }
        let pulse = if self.session.compiled.has_errors() {
            0.30
        } else if self.session.compiled.warning_count() > 0 {
            0.18
        } else {
            0.08
        };
        let phase = ((self.session.ticks % 180) as f32 / 180.0) * std::f32::consts::TAU;
        let lift = phase.sin().abs() * pulse;
        [
            (base[0] + lift).min(1.0),
            (base[1] + lift * 0.5).min(1.0),
            (base[2] + lift * 0.25).min(1.0),
            base[3],
        ]
    }
}

fn runtime_rect_entity(rect: &RectNode) -> RuntimeEntity {
    RuntimeEntity {
        name: rect.name.clone(),
        template: rect.visual.template,
        visible: rect.visual.visible,
        spawn_time: 0.0,
        layer: rect.visual.layer,
        z: rect.visual.z,
        pos: rect.visual.pos,
        size: rect.visual.size,
        color: rect.visual.color,
        group: rect.visual.group.clone(),
        scroll: [0.0, 0.0],
        repeat_x: false,
        repeat_y: false,
        flip_x: false,
        flip_y: false,
        script: rect.visual.script_binding.clone(),
        script_state: HashMap::new(),
        kind: RuntimeEntityKind::Rect,
    }
}

fn runtime_sprite_entity(
    sprite: &SpriteNode,
    markers: &HashMap<String, [f32; 2]>,
) -> RuntimeEntity {
    let pos = sprite
        .symbol
        .as_deref()
        .and_then(|symbol| markers.get(symbol))
        .copied()
        .unwrap_or(sprite.visual.pos);

    RuntimeEntity {
        name: sprite.name.clone(),
        template: sprite.visual.template,
        visible: sprite.visual.visible,
        spawn_time: 0.0,
        layer: sprite.visual.layer,
        z: sprite.visual.z,
        pos,
        size: sprite.visual.size,
        color: sprite.visual.color,
        group: sprite.visual.group.clone(),
        scroll: sprite.scroll,
        repeat_x: sprite.repeat_x,
        repeat_y: sprite.repeat_y,
        flip_x: sprite.flip_x,
        flip_y: sprite.flip_y,
        script: sprite.visual.script_binding.clone(),
        script_state: HashMap::new(),
        kind: RuntimeEntityKind::Sprite {
            texture: sprite.textures.first().cloned(),
            frames: sprite.textures.clone(),
            animation_fps: sprite.animation_fps,
            animation_mode: sprite.animation_mode,
            destroy_on_animation_end: sprite.destroy_on_animation_end,
        },
    }
}

fn runtime_text_entity(text: &TextNode) -> RuntimeEntity {
    RuntimeEntity {
        name: text.name.clone(),
        template: text.visual.template,
        visible: text.visual.visible,
        spawn_time: 0.0,
        layer: text.visual.layer,
        z: text.visual.z,
        pos: text.visual.pos,
        size: [0.0, 0.0],
        color: text.visual.color,
        group: text.visual.group.clone(),
        scroll: [0.0, 0.0],
        repeat_x: false,
        repeat_y: false,
        flip_x: false,
        flip_y: false,
        script: text.visual.script_binding.clone(),
        script_state: HashMap::new(),
        kind: RuntimeEntityKind::Text {
            value: text.value.clone(),
            font: text.font.clone(),
            font_size: text.font_size,
        },
    }
}

fn compile_static_map_commands(maps: &[AsciiMapNode]) -> Vec<DrawCommand> {
    maps.iter()
        .flat_map(|map| {
            let legend: HashMap<char, &MapLegendMeaning> = map
                .legend
                .iter()
                .map(|entry| (entry.symbol, &entry.meaning))
                .collect();
            let mut commands = Vec::new();
            for (row, line) in map.rows.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    if matches!(ch, ' ' | '.') {
                        continue;
                    }
                    if let Some(MapLegendMeaning::Color(color)) = legend.get(&ch) {
                        commands.push(DrawCommand::Rect(SceneRect {
                            layer: -10,
                            z: (row as i32) * 100 + col as i32,
                            x: map.origin[0] + col as f32 * map.cell[0],
                            y: map.origin[1] + row as f32 * map.cell[1],
                            width: map.cell[0],
                            height: map.cell[1],
                            color: *color,
                            visible: true,
                        }));
                    }
                }
            }
            commands
        })
        .collect()
}

fn compile_map_markers(maps: &[AsciiMapNode]) -> HashMap<String, [f32; 2]> {
    let mut markers = HashMap::new();
    for map in maps {
        let legend: HashMap<char, &MapLegendMeaning> = map
            .legend
            .iter()
            .map(|entry| (entry.symbol, &entry.meaning))
            .collect();
        for (row, line) in map.rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if let Some(MapLegendMeaning::Marker) = legend.get(&ch) {
                    markers.entry(ch.to_string()).or_insert([
                        map.origin[0] + col as f32 * map.cell[0],
                        map.origin[1] + row as f32 * map.cell[1],
                    ]);
                }
            }
        }
    }
    markers
}

fn submit_draw_command(
    frame: &mut SceneFrame,
    camera: &SceneCamera,
    view: &RenderView,
    stable_order: i32,
    command: &DrawCommand,
    project: &RpuProject,
    elapsed_time: f32,
) {
    match command {
        DrawCommand::Rect(rect) => {
            let (x, y, width, height) =
                world_to_screen(camera, view, rect.x, rect.y, rect.width, rect.height);
            frame.push_rect(rect.layer, rect.z * 1000 + stable_order, x, y, width, height, rect.color);
        }
        DrawCommand::Sprite(sprite) => {
            let texture_name = current_texture_frame(
                sprite.textures.first().map(String::as_str),
                &sprite.textures,
                sprite.animation_fps,
                sprite.animation_mode,
                elapsed_time,
                0.0,
            );
            let texture_path = texture_name
                .as_deref()
                .map(|texture| project.root().join("assets").join(texture))
                .map(pathbuf_to_string);
            submit_sprite(
                frame,
                camera,
                view,
                sprite.layer,
                sprite.z * 1000 + stable_order,
                sprite.x,
                sprite.y,
                sprite.width,
                sprite.height,
                sprite.color,
                sprite.scroll,
                sprite.repeat_x,
                sprite.repeat_y,
                sprite.flip_x,
                sprite.flip_y,
                texture_path.as_deref(),
                0.0,
            );
        }
        DrawCommand::Text(text) => {
            let font_path = project.root().join("assets").join(&text.font);
            let (x, y, _, _) = world_to_screen(camera, view, text.x, text.y, 0.0, 0.0);
            frame.push_text(
                text.layer,
                text.z * 1000 + stable_order,
                x,
                y,
                &text.value,
                &pathbuf_to_string(font_path),
                text.font_size,
                text.color,
            );
        }
    }
}

fn log_compilation(label: &str, compiled: &CompiledProject) {
    eprintln!(
        "rpu: {}: {} scene(s), {} script(s), {} camera(s), {} rect(s), {} sprite(s), {} handler(s), {} op(s), {} asset(s), {} warning(s), {} error(s)",
        label,
        compiled.scene_count(),
        compiled.bytecode_scripts.len(),
        compiled.camera_count(),
        compiled.rect_count(),
        compiled.sprite_count(),
        compiled.handler_count(),
        compiled.op_count(),
        compiled.assets.len(),
        compiled.warning_count(),
        compiled.error_count()
    );
    for diagnostic in &compiled.diagnostics {
        match (&diagnostic.path, diagnostic.line) {
            (Some(path), Some(line)) => eprintln!(
                "rpu: {:?}: {} ({}:{})",
                diagnostic.severity,
                diagnostic.message,
                path.display(),
                line
            ),
            (Some(path), None) => eprintln!(
                "rpu: {:?}: {} ({})",
                diagnostic.severity,
                diagnostic.message,
                path.display()
            ),
            (None, Some(line)) => eprintln!(
                "rpu: {:?}: {} (line {})",
                diagnostic.severity,
                diagnostic.message,
                line
            ),
            (None, None) => eprintln!("rpu: {:?}: {}", diagnostic.severity, diagnostic.message),
        }
    }
}

fn color_from_name(name: &str) -> [f32; 3] {
    let mut hash = 0u32;
    for byte in name.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(u32::from(byte));
    }

    let r = 0.10 + ((hash & 0xFF) as f32 / 255.0) * 0.35;
    let g = 0.12 + (((hash >> 8) & 0xFF) as f32 / 255.0) * 0.35;
    let b = 0.16 + (((hash >> 16) & 0xFF) as f32 / 255.0) * 0.40;
    [r, g, b]
}

fn world_to_screen(
    camera: &SceneCamera,
    view: &RenderView,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> (f32, f32, f32, f32) {
    let zoom = camera.zoom.max(0.01);
    let virtual_x = (x - camera.x) * zoom + view.virtual_size.0 * 0.5;
    let virtual_y = (y - camera.y) * zoom + view.virtual_size.1 * 0.5;
    view.map_rect(virtual_x, virtual_y, width * zoom, height * zoom)
}

fn world_to_virtual(
    camera: &SceneCamera,
    view: &RenderView,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> (f32, f32, f32, f32) {
    let zoom = camera.zoom.max(0.01);
    let virtual_x = (x - camera.x) * zoom + view.virtual_size.0 * 0.5;
    let virtual_y = (y - camera.y) * zoom + view.virtual_size.1 * 0.5;
    (virtual_x, virtual_y, width * zoom, height * zoom)
}

fn intersects(a: &RuntimeEntity, b: &RuntimeEntity) -> bool {
    let ax0 = a.pos[0];
    let ay0 = a.pos[1];
    let ax1 = ax0 + a.size[0];
    let ay1 = ay0 + a.size[1];
    let bx0 = b.pos[0];
    let by0 = b.pos[1];
    let bx1 = bx0 + b.size[0];
    let by1 = by0 + b.size[1];

    ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
}

fn current_texture_frame(
    current_texture: Option<&str>,
    frames: &[String],
    animation_fps: f32,
    animation_mode: AnimationMode,
    elapsed_time: f32,
    spawn_time: f32,
) -> Option<String> {
    if frames.is_empty() {
        return current_texture.map(ToOwned::to_owned);
    }
    if frames.len() == 1 || animation_fps <= 0.0 {
        return current_texture
            .map(ToOwned::to_owned)
            .or_else(|| frames.first().cloned());
    }

    let age = (elapsed_time - spawn_time).max(0.0);
    let raw_index = (age * animation_fps).floor() as usize;
    let index = match animation_mode {
        AnimationMode::Loop => raw_index % frames.len(),
        AnimationMode::Once => raw_index.min(frames.len().saturating_sub(1)),
    };
    frames.get(index).cloned()
}

fn submit_sprite(
    frame: &mut SceneFrame,
    camera: &SceneCamera,
    view: &RenderView,
    layer: i32,
    order: i32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
    scroll: [f32; 2],
    repeat_x: bool,
    repeat_y: bool,
    flip_x: bool,
    flip_y: bool,
    texture_path: Option<&str>,
    elapsed_time: f32,
) {
    let scrolled_x = x + scroll[0] * elapsed_time;
    let scrolled_y = y + scroll[1] * elapsed_time;

    if !repeat_x && !repeat_y {
        let (sx, sy, sw, sh) = world_to_screen(camera, view, scrolled_x, scrolled_y, width, height);
        frame.push_sprite(layer, order, sx, sy, sw, sh, color, flip_x, flip_y, texture_path);
        return;
    }

    let (base_x, base_y, virtual_w, virtual_h) =
        world_to_virtual(camera, view, scrolled_x, scrolled_y, width, height);
    if virtual_w <= 0.0 || virtual_h <= 0.0 {
        return;
    }

    let start_x = if repeat_x {
        base_x.rem_euclid(virtual_w) - virtual_w
    } else {
        base_x
    };
    let start_y = if repeat_y {
        base_y.rem_euclid(virtual_h) - virtual_h
    } else {
        base_y
    };
    let max_x = if repeat_x {
        view.virtual_size.0 + virtual_w
    } else {
        start_x + 0.1
    };
    let max_y = if repeat_y {
        view.virtual_size.1 + virtual_h
    } else {
        start_y + 0.1
    };

    let mut tile_y = start_y;
    while tile_y < max_y {
        let mut tile_x = start_x;
        while tile_x < max_x {
            let (sx, sy, sw, sh) = view.map_rect(tile_x, tile_y, virtual_w, virtual_h);
            frame.push_sprite(layer, order, sx, sy, sw, sh, color, flip_x, flip_y, texture_path);
            if !repeat_x {
                break;
            }
            tile_x += virtual_w;
        }
        if !repeat_y {
            break;
        }
        tile_y += virtual_h;
    }
}

fn pathbuf_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
