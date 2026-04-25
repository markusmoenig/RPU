use anyhow::Result;
use image::{ImageBuffer, Rgba};
use rpu_core::{
    Anchor, AnimationMode, AsciiMapNode, BinaryOp, BundledProject, BytecodeOp, CompiledProject,
    Condition, DestroyTarget, DrawCommand, Expr, MapLegendMeaning, OpCode, RectNode, ResizeMode,
    RpuProject, SceneCamera, SceneRect, ScriptProperty, ScriptTarget, SpriteNode, TextAlign,
    TextNode, WindowConfig, apply_scene_layout,
};
#[cfg(any(
    target_arch = "wasm32",
    all(
        not(target_arch = "wasm32"),
        not(target_os = "tvos"),
        not(target_os = "ios")
    )
))]
use rpu_scenevm::run_app;
use rpu_scenevm::{RpuSceneApp, RuntimeContext, SceneFrame, register_generated_rgba_texture};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(any(
    target_arch = "wasm32",
    all(
        not(target_arch = "wasm32"),
        not(target_os = "tvos"),
        not(target_os = "ios")
    )
))]
pub fn run(project: RpuProject) -> Result<()> {
    run_app(RuntimeApp::new(project)?)
}

#[cfg(any(
    target_arch = "wasm32",
    all(
        not(target_arch = "wasm32"),
        not(target_os = "tvos"),
        not(target_os = "ios")
    )
))]
pub fn run_bundled(project: BundledProject, asset_base: &str) -> Result<()> {
    run_app(RuntimeApp::new_bundled(project, asset_base)?)
}

pub struct RuntimeApp {
    session: RuntimeSession,
}

impl RuntimeApp {
    pub fn new(project: RpuProject) -> Result<Self> {
        Ok(Self {
            session: RuntimeSession::new_native(project)?,
        })
    }

    pub fn new_bundled(project: BundledProject, asset_base: &str) -> Result<Self> {
        Ok(Self {
            session: RuntimeSession::new_bundled(project, asset_base)?,
        })
    }
}

impl RpuSceneApp for RuntimeApp {
    fn window_title(&self) -> Option<String> {
        Some(format!("RPU - {}", self.session.compiled.name))
    }

    fn initial_window_size(&self) -> Option<(u32, u32)> {
        let window = &self.session.compiled.window;
        #[cfg(target_arch = "wasm32")]
        {
            return Some((window.width.max(1), window.height.max(1)));
        }
        #[cfg(not(target_arch = "wasm32"))]
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
        for command in self.session.drain_audio_commands() {
            match command {
                AudioCommand::PlaySound(path) => _ctx.play_sound(&path),
                AudioCommand::PlayMusic(path) => _ctx.play_music(&path),
                AudioCommand::StopMusic => _ctx.stop_music(),
            }
        }
    }

    fn render(&mut self, _ctx: &mut RuntimeContext, frame: &mut SceneFrame) {
        frame.clear_color(self.current_clear_color());
        let camera = self
            .session
            .compiled
            .active_camera_for(&self.session.current_scene);
        let viewport = frame.size();
        let view = RenderView::new(&self.session.compiled.window, viewport);

        for (index, command) in self.session.world.static_draw_commands.iter().enumerate() {
            submit_draw_command(
                frame,
                &camera,
                &view,
                index as i32,
                command,
                &self.session.asset_base,
                &self.session.high_scores,
                self.session.query_state.elapsed_time,
            );
        }

        let base_order = self.session.world.static_draw_commands.len() as i32;
        for (index, entity) in self.session.world.entities.iter().enumerate() {
            if !entity.visible {
                continue;
            }
            let stable_order = base_order + index as i32;
            let (x, y, width, height) = screen_rect_for_anchor(
                entity.anchor,
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
                        entity.animation_start_time,
                    );
                    let texture_path = texture_name
                        .as_deref()
                        .map(|texture| self.session.asset_path(texture));
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
                        entity.rotation,
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
                    align,
                } => {
                    let font_path = self.session.asset_path(font);
                    let (tx, ty) = screen_point_for_anchor(
                        entity.anchor,
                        &camera,
                        &view,
                        entity.pos[0],
                        entity.pos[1],
                    );
                    frame.push_text(
                        entity.layer,
                        entity.z * 1000 + stable_order,
                        tx,
                        ty,
                        value,
                        &font_path,
                        *font_size * view.scale.max(0.01),
                        entity.color,
                        Anchor::World,
                        *align,
                    );
                }
            }
        }
    }
}

struct RuntimeSession {
    project: Option<RpuProject>,
    compiled: CompiledProject,
    asset_base: String,
    hot_reload: bool,
    current_scene: String,
    pending_scene: Option<String>,
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
    persisted_entity_state: HashMap<String, HashMap<String, Value>>,
    high_scores: HighScoreTable,
    pending_audio: Vec<AudioCommand>,
}

struct RuntimeWorld {
    static_draw_commands: Vec<DrawCommand>,
    colliders: Vec<RuntimeCollider>,
    templates: Vec<RuntimeEntity>,
    entities: Vec<RuntimeEntity>,
}

#[derive(Clone, Copy)]
struct RuntimeCollider {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone)]
struct RuntimeEntity {
    name: String,
    template: bool,
    visible: bool,
    spawn_time: f32,
    animation_start_time: f32,
    group: Option<String>,
    anchor: Anchor,
    layer: i32,
    z: i32,
    pos: [f32; 2],
    size: [f32; 2],
    rotation: f32,
    color: [f32; 4],
    scroll: [f32; 2],
    repeat_x: bool,
    repeat_y: bool,
    flip_x: bool,
    flip_y: bool,
    physics: RuntimePhysics,
    script: Option<String>,
    script_state: HashMap<String, Value>,
    kind: RuntimeEntityKind,
}

#[derive(Clone)]
struct RuntimePhysics {
    mode: rpu_core::PhysicsMode,
    settings: rpu_core::PlatformerPhysicsSettings,
    velocity: [f32; 2],
    move_x: f32,
    jump_requested: bool,
    grounded: bool,
}

#[derive(Clone)]
enum RuntimeEntityKind {
    Rect,
    Sprite {
        texture: Option<String>,
        frames: Vec<String>,
        current_animation: Option<String>,
        animations: HashMap<String, rpu_core::SpriteAnimation>,
        animation_fps: f32,
        animation_mode: AnimationMode,
        destroy_on_animation_end: bool,
    },
    Text {
        value: String,
        font: String,
        font_size: f32,
        align: TextAlign,
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

#[derive(Clone, Debug)]
struct HighScoreEntry {
    name: String,
    score: i32,
}

#[derive(Clone, Debug)]
struct HighScoreTable {
    entries: Vec<HighScoreEntry>,
}

enum AudioCommand {
    PlaySound(String),
    PlayMusic(String),
    StopMusic,
}

impl RuntimeSession {
    fn new_native(project: RpuProject) -> Result<Self> {
        let compiled = project.compile()?;
        log_compilation("initial compile", &compiled);
        let current_scene = resolve_scene_name(&compiled, compiled.start_scene.as_str());
        let world = RuntimeWorld::from_compiled_scene(
            &compiled,
            &current_scene,
            &project.root().join("assets").display().to_string(),
        );
        let mut session = Self {
            asset_base: project.root().join("assets").display().to_string(),
            project: Some(project),
            compiled,
            hot_reload: true,
            current_scene,
            pending_scene: None,
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
            persisted_entity_state: HashMap::new(),
            high_scores: HighScoreTable::default(),
            pending_audio: Vec::new(),
        };
        session.initialize_script_state_all();
        Ok(session)
    }

    fn new_bundled(project: BundledProject, asset_base: &str) -> Result<Self> {
        let compiled = project.compile()?;
        log_compilation("initial compile", &compiled);
        let current_scene = resolve_scene_name(&compiled, compiled.start_scene.as_str());
        let world = RuntimeWorld::from_compiled_scene(&compiled, &current_scene, asset_base);
        let mut session = Self {
            project: None,
            compiled,
            asset_base: asset_base.trim_end_matches('/').to_string(),
            hot_reload: false,
            current_scene,
            pending_scene: None,
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
            persisted_entity_state: HashMap::new(),
            high_scores: HighScoreTable::default(),
            pending_audio: Vec::new(),
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
            self.world.apply_platformer_physics(dt);
            self.world
                .remove_finished_animations(self.query_state.elapsed_time);
            self.capture_persistent_entity_state();
        }
        self.maybe_reload();
    }

    fn drain_audio_commands(&mut self) -> Vec<AudioCommand> {
        std::mem::take(&mut self.pending_audio)
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
            if self
                .world
                .entities
                .get(entity_index)
                .and_then(|entity| entity.script_state.get(&state.name))
                .is_some()
            {
                continue;
            }
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
        if !self.hot_reload {
            return;
        }
        if self.last_reload_poll.elapsed() < Duration::from_millis(500) {
            return;
        }
        self.last_reload_poll = Instant::now();

        let Some(project) = self.project.as_ref() else {
            return;
        };

        match project.has_source_changes_since(self.compiled.fingerprint.latest_modified) {
            Ok(true) => match project.compile() {
                Ok(compiled) => {
                    log_compilation("hot reload", &compiled);
                    self.capture_persistent_entity_state();
                    self.compiled = compiled;
                    self.current_scene =
                        resolve_scene_name(&self.compiled, self.current_scene.as_str());
                    self.rebuild_world_for_scene(false);
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

    fn asset_path(&self, asset_name: &str) -> String {
        let asset_name = asset_name.trim_start_matches('/');
        if self.asset_base.is_empty() {
            asset_name.to_string()
        } else {
            format!("{}/{}", self.asset_base, asset_name)
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

            for handler in script
                .handlers
                .iter()
                .filter(|handler| handler.event == event)
            {
                scheduled.push((
                    entity_index,
                    entity.name.clone(),
                    script.path.clone(),
                    handler.params.clone(),
                    handler.ops.clone(),
                ));
            }
        }

        for (_entity_index, entity_name, script_path, params, ops) in scheduled {
            let Some(current_index) = self.world.find_entity_index(&entity_name) else {
                continue;
            };
            let mut locals = HashMap::new();
            bind_handler_args(&params, &[Value::Scalar(dt)], &mut locals);
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
            if self.apply_pending_scene_change() {
                break;
            }
        }
    }

    fn execute_entity_event(&mut self, entity_index: usize, event: &str, dt: f32) {
        self.execute_entity_event_with_args(entity_index, event, dt, &[Value::Scalar(dt)]);
    }

    fn execute_entity_event_with_args(
        &mut self,
        entity_index: usize,
        event: &str,
        dt: f32,
        args: &[Value],
    ) {
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
        let handlers: Vec<(Vec<String>, Vec<BytecodeOp>)> = script
            .handlers
            .iter()
            .filter(|handler| handler.event == event)
            .map(|handler| (handler.params.clone(), handler.ops.clone()))
            .collect();
        if handlers.is_empty() {
            return;
        }
        let entity_name = entity.name.clone();
        for (params, ops) in handlers {
            let mut locals = HashMap::new();
            bind_handler_args(&params, args, &mut locals);
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
        let _ = self.apply_pending_scene_change();
    }

    fn queue_scene_switch(&mut self, scene_name: String) {
        self.pending_scene = Some(scene_name);
    }

    fn apply_pending_scene_change(&mut self) -> bool {
        let Some(scene_name) = self.pending_scene.take() else {
            return false;
        };
        self.capture_persistent_entity_state();
        self.current_scene = resolve_scene_name(&self.compiled, &scene_name);
        self.rebuild_world_for_scene(true);
        true
    }

    fn rebuild_world_for_scene(&mut self, run_ready: bool) {
        self.world = RuntimeWorld::from_compiled_scene(
            &self.compiled,
            &self.current_scene,
            &self.asset_base,
        );
        self.restore_persistent_entity_state();
        self.initialize_script_state_all();
        if run_ready && self.initialized {
            self.execute_event("ready", 0.0);
        }
    }

    fn capture_persistent_entity_state(&mut self) {
        for entity in &self.world.entities {
            if entity.script.is_some() && !entity.script_state.is_empty() {
                self.persisted_entity_state
                    .insert(entity.name.clone(), entity.script_state.clone());
            }
        }
    }

    fn restore_persistent_entity_state(&mut self) {
        for entity in &mut self.world.entities {
            if let Some(saved) = self.persisted_entity_state.get(&entity.name) {
                entity.script_state.extend(saved.clone());
            }
        }
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
                if let Some(signal) = self.invoke_builtin_action(
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
                ) {
                    return signal;
                }
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
                let instance_name = name
                    .clone()
                    .unwrap_or_else(|| self.next_spawn_name(template));
                if let Some(spawned_index) = self.world.spawn_from_template(
                    template,
                    &instance_name,
                    [x, y],
                    self.query_state.elapsed_time,
                ) {
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

    fn invoke_builtin_action(
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
    ) -> Option<ExecSignal> {
        match name {
            "emit" if args.len() == 2 => {
                let event_name = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_string()?
                    .to_string();
                let value = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    &args[1],
                    dt,
                    locals,
                    call_depth,
                )?;
                self.execute_entity_event_with_args(
                    entity_index,
                    "event",
                    dt,
                    &[Value::String(event_name), value],
                );
                Some(ExecSignal::Continue)
            }
            "set_scene" if args.len() == 1 => {
                let scene_name = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_string()?
                    .to_string();
                self.queue_scene_switch(scene_name);
                Some(ExecSignal::Stop)
            }
            "submit_score" if args.len() == 2 => {
                let name = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_string()?
                    .to_string();
                let score = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()? as i32;
                self.high_scores.submit(&name, score);
                Some(ExecSignal::Continue)
            }
            "play_sound" if args.len() == 1 => {
                let asset_name = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_string()?
                    .to_string();
                self.pending_audio
                    .push(AudioCommand::PlaySound(self.asset_path(&asset_name)));
                Some(ExecSignal::Continue)
            }
            "play_music" if args.len() == 1 => {
                let asset_name = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_string()?
                    .to_string();
                self.pending_audio
                    .push(AudioCommand::PlayMusic(self.asset_path(&asset_name)));
                Some(ExecSignal::Continue)
            }
            "stop_music" if args.is_empty() => {
                self.pending_audio.push(AudioCommand::StopMusic);
                Some(ExecSignal::Continue)
            }
            _ => None,
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
        let Some(function) = script
            .functions
            .iter()
            .find(|function| function.name == name)
        else {
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
                let left = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    left,
                    dt,
                    locals,
                    call_depth,
                )?;
                let right = self.eval_expr(
                    entity_index,
                    entity_name,
                    script_path,
                    event,
                    line,
                    right,
                    dt,
                    locals,
                    call_depth,
                )?;
                compare_values(&left, *op, &right)?
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
                .eval_builtin_query(
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
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        left,
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let right = self
                    .eval_expr(
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
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        value,
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let min = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        min,
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let max = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        max,
                        dt,
                        locals,
                        call_depth,
                    )?
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
            "input_left" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.input_left as i32 as f32))
            }
            "input_right" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.input_right as i32 as f32))
            }
            "input_up" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.input_up as i32 as f32))
            }
            "input_down" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.input_down as i32 as f32))
            }
            "input_action" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.input_action as i32 as f32))
            }
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
                Some(Value::Scalar(
                    self.world.find_entity(name).is_some() as i32 as f32
                ))
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
                Some(Value::Scalar(
                    (self.query_state.elapsed_time - entity.spawn_time).max(0.0),
                ))
            }
            "lerp" if args.len() == 3 => {
                let a = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let b = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let t = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[2],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                Some(Value::Scalar(a + (b - a) * t))
            }
            "pulse" if args.len() == 1 => {
                let period = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let period = period.max(0.001);
                let phase = (self.query_state.elapsed_time / period) * std::f32::consts::TAU;
                Some(Value::Scalar(0.5 + 0.5 * phase.sin()))
            }
            "smoothstep" if args.len() == 3 => {
                let edge0 = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let edge1 = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let x = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[2],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
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
                let alpha = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                color[3] = alpha.clamp(0.0, 1.0);
                Some(Value::Color(color))
            }
            "format_int" if args.len() == 2 => {
                let value = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let digits = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let digits = digits.max(1.0).floor() as usize;
                let value = value.max(0.0).floor() as i32;
                Some(Value::String(format!("{value:0digits$}")))
            }
            "high_score_name" if args.len() == 1 => {
                let index = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let index = index.max(1.0).floor() as usize - 1;
                Some(Value::String(self.high_scores.name_at(index).to_string()))
            }
            "high_score_value" if args.len() == 1 => {
                let index = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let index = index.max(1.0).floor() as usize - 1;
                Some(Value::Scalar(self.high_scores.score_at(index) as f32))
            }
            "difficulty" if args.is_empty() => Some(Value::Scalar(
                1.0 + (self.query_state.elapsed_time / 12.0).floor(),
            )),
            "screen_width" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.window_size.0 as f32))
            }
            "screen_height" if args.is_empty() => {
                Some(Value::Scalar(self.query_state.window_size.1 as f32))
            }
            "rand" if args.len() == 2 => {
                let min = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let max = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[1],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
                let key = line
                    .map(|line| {
                        format!(
                            "{}:{}:{}:{}:rand",
                            script_path.display(),
                            entity_name,
                            event,
                            line
                        )
                    })
                    .unwrap_or_else(|| {
                        format!("{}:{}:{}:rand", script_path.display(), entity_name, event)
                    });
                Some(Value::Scalar(self.random_between_spread(&key, min, max)))
            }
            "every" if args.len() == 1 || args.len() == 2 => {
                let min_interval = self
                    .eval_expr(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        line,
                        &args[0],
                        dt,
                        locals,
                        call_depth,
                    )?
                    .as_scalar()?;
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
                    )?
                    .as_scalar()?
                } else {
                    min_interval
                };
                let next_interval = min_interval.min(max_interval);
                if next_interval <= 0.0 {
                    return Some(Value::Scalar(0.0));
                }
                let line = line?;
                let key = format!(
                    "{}:{}:{}:{}",
                    script_path.display(),
                    entity_name,
                    event,
                    line
                );
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

        let elapsed_time = self.query_state.elapsed_time;
        match (target_property(target), value) {
            (ScriptProperty::X, Value::Scalar(value)) => entity.pos[0] = value,
            (ScriptProperty::Y, Value::Scalar(value)) => entity.pos[1] = value,
            (ScriptProperty::Width, Value::Scalar(value)) => entity.size[0] = value,
            (ScriptProperty::Height, Value::Scalar(value)) => entity.size[1] = value,
            (ScriptProperty::Pos, Value::Vec2(value)) => entity.pos = value,
            (ScriptProperty::Size, Value::Vec2(value)) => entity.size = value,
            (ScriptProperty::Rotation, Value::Scalar(value)) => entity.rotation = value,
            (ScriptProperty::Color, Value::Color(value)) => entity.color = value,
            (ScriptProperty::FlipX, Value::Scalar(value)) => entity.flip_x = value != 0.0,
            (ScriptProperty::FlipY, Value::Scalar(value)) => entity.flip_y = value != 0.0,
            (ScriptProperty::Vx, Value::Scalar(value)) => entity.physics.velocity[0] = value,
            (ScriptProperty::Vy, Value::Scalar(value)) => entity.physics.velocity[1] = value,
            (ScriptProperty::MoveX, Value::Scalar(value)) => {
                entity.physics.move_x = value.clamp(-1.0, 1.0)
            }
            (ScriptProperty::Jump, Value::Scalar(value)) => {
                entity.physics.jump_requested = value != 0.0
            }
            (ScriptProperty::Grounded, Value::Scalar(value)) => {
                entity.physics.grounded = value != 0.0
            }
            (ScriptProperty::Texture, Value::String(value)) => match &mut entity.kind {
                RuntimeEntityKind::Sprite {
                    texture,
                    frames,
                    current_animation,
                    animation_fps,
                    animation_mode,
                    destroy_on_animation_end,
                    ..
                } => {
                    *texture = Some(value.clone());
                    *frames = vec![value];
                    *current_animation = None;
                    *animation_fps = 0.0;
                    *animation_mode = AnimationMode::Loop;
                    *destroy_on_animation_end = false;
                }
                RuntimeEntityKind::Rect | RuntimeEntityKind::Text { .. } => return false,
            },
            (ScriptProperty::Animation, Value::String(value)) => match &mut entity.kind {
                RuntimeEntityKind::Sprite {
                    texture,
                    frames,
                    current_animation,
                    animations,
                    animation_fps,
                    animation_mode,
                    destroy_on_animation_end,
                } => {
                    let Some(animation) = animations.get(&value) else {
                        return false;
                    };
                    if current_animation.as_deref() == Some(value.as_str()) {
                        return true;
                    }
                    if animation.textures.is_empty() {
                        return false;
                    }
                    *texture = animation.textures.first().cloned();
                    *frames = animation.textures.clone();
                    *current_animation = Some(value);
                    *animation_fps = animation.fps;
                    *animation_mode = animation.mode;
                    *destroy_on_animation_end = false;
                    entity.animation_start_time = elapsed_time;
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
            ScriptProperty::Rotation => Value::Scalar(entity.rotation),
            ScriptProperty::Color => Value::Color(entity.color),
            ScriptProperty::FlipX => Value::Scalar(entity.flip_x as i32 as f32),
            ScriptProperty::FlipY => Value::Scalar(entity.flip_y as i32 as f32),
            ScriptProperty::Vx => Value::Scalar(entity.physics.velocity[0]),
            ScriptProperty::Vy => Value::Scalar(entity.physics.velocity[1]),
            ScriptProperty::MoveX => Value::Scalar(entity.physics.move_x),
            ScriptProperty::Jump => Value::Scalar(entity.physics.jump_requested as i32 as f32),
            ScriptProperty::Grounded => Value::Scalar(entity.physics.grounded as i32 as f32),
            ScriptProperty::Texture => match &entity.kind {
                RuntimeEntityKind::Sprite { texture, .. } => {
                    Value::String(texture.clone().unwrap_or_default())
                }
                RuntimeEntityKind::Rect | RuntimeEntityKind::Text { .. } => return None,
            },
            ScriptProperty::Animation => match &entity.kind {
                RuntimeEntityKind::Sprite {
                    current_animation, ..
                } => Value::String(current_animation.clone().unwrap_or_default()),
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
    fn from_compiled_scene(compiled: &CompiledProject, scene_name: &str, asset_base: &str) -> Self {
        let mut static_draw_commands = Vec::new();
        let mut colliders = Vec::new();
        let mut templates = Vec::new();
        let mut entities = Vec::new();

        for document in &compiled.parsed_scenes {
            for scene in &document.scenes {
                if scene.name != scene_name {
                    continue;
                }
                let scene = apply_scene_layout(scene);
                static_draw_commands.extend(compile_static_map_commands(&scene.maps, asset_base));
                colliders.extend(compile_map_colliders(&scene.maps));
                for high_score in &scene.high_scores {
                    if !high_score.visual.visible || high_score.visual.template {
                        continue;
                    }
                    static_draw_commands.push(DrawCommand::HighScore(rpu_core::SceneHighScore {
                        anchor: high_score.visual.anchor,
                        layer: high_score.visual.layer,
                        z: high_score.visual.z,
                        x: high_score.visual.pos[0],
                        y: high_score.visual.pos[1],
                        width: high_score.visual.size[0],
                        color: high_score.visual.color,
                        font: high_score.font.clone(),
                        font_size: high_score.font_size,
                        items: high_score.items,
                        gap: high_score.gap,
                        score_digits: high_score.score_digits,
                        visible: high_score.visual.visible,
                    }));
                }
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
            colliders,
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
        entity.animation_start_time = spawn_time;
        self.entities.push(entity);
        Some(self.entities.len() - 1)
    }

    fn remove_entity_named(&mut self, name: &str) -> bool {
        let original = self.entities.len();
        self.entities.retain(|entity| entity.name != name);
        self.entities.len() != original
    }

    fn apply_platformer_physics(&mut self, dt: f32) {
        let colliders = &self.colliders;
        for entity in &mut self.entities {
            if entity.physics.mode != rpu_core::PhysicsMode::Platformer {
                continue;
            }
            let settings = entity.physics.settings;

            if entity.physics.move_x != 0.0 {
                entity.physics.velocity[0] += entity.physics.move_x * settings.acceleration * dt;
                entity.physics.velocity[0] =
                    entity.physics.velocity[0].clamp(-settings.max_speed, settings.max_speed);
            } else if entity.physics.velocity[0] > 0.0 {
                entity.physics.velocity[0] =
                    (entity.physics.velocity[0] - settings.friction * dt).max(0.0);
            } else if entity.physics.velocity[0] < 0.0 {
                entity.physics.velocity[0] =
                    (entity.physics.velocity[0] + settings.friction * dt).min(0.0);
            }

            if entity.physics.jump_requested && entity.physics.grounded {
                entity.physics.velocity[1] = -settings.jump_speed;
                entity.physics.grounded = false;
            }
            entity.physics.jump_requested = false;

            let vx = entity.physics.velocity[0];
            entity.pos[0] += vx * dt;
            if vx != 0.0 {
                for collider in colliders {
                    if !runtime_aabb_intersects(
                        entity.pos[0],
                        entity.pos[1],
                        entity.size[0],
                        entity.size[1],
                        collider,
                    ) {
                        continue;
                    }
                    if vx > 0.0 {
                        entity.pos[0] = collider.x - entity.size[0];
                    } else {
                        entity.pos[0] = collider.x + collider.width;
                    }
                    entity.physics.velocity[0] = 0.0;
                }
            }

            entity.physics.velocity[1] = (entity.physics.velocity[1] + settings.gravity * dt)
                .clamp(-settings.jump_speed, settings.max_fall_speed);
            let vy = entity.physics.velocity[1];
            entity.pos[1] += vy * dt;
            entity.physics.grounded = false;
            if vy != 0.0 {
                for collider in colliders {
                    if !runtime_aabb_intersects(
                        entity.pos[0],
                        entity.pos[1],
                        entity.size[0],
                        entity.size[1],
                        collider,
                    ) {
                        continue;
                    }
                    if vy > 0.0 {
                        entity.pos[1] = collider.y - entity.size[1];
                        entity.physics.grounded = true;
                    } else {
                        entity.pos[1] = collider.y + collider.height;
                    }
                    entity.physics.velocity[1] = 0.0;
                }
            }
        }
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
                elapsed_time - entity.animation_start_time < duration
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

fn bind_handler_args(params: &[String], args: &[Value], locals: &mut HashMap<String, Value>) {
    for (param, value) in params.iter().zip(args.iter()) {
        locals.insert(param.clone(), value.clone());
    }
}

fn compare_values(left: &Value, op: rpu_core::CompareOp, right: &Value) -> Option<bool> {
    match (left, right) {
        (Value::Scalar(left), Value::Scalar(right)) => Some(match op {
            rpu_core::CompareOp::Less => left < right,
            rpu_core::CompareOp::LessEqual => left <= right,
            rpu_core::CompareOp::Greater => left > right,
            rpu_core::CompareOp::GreaterEqual => left >= right,
            rpu_core::CompareOp::Equal => (left - right).abs() < f32::EPSILON,
            rpu_core::CompareOp::NotEqual => (left - right).abs() >= f32::EPSILON,
        }),
        (Value::String(left), Value::String(right)) => match op {
            rpu_core::CompareOp::Equal => Some(left == right),
            rpu_core::CompareOp::NotEqual => Some(left != right),
            _ => None,
        },
        _ => None,
    }
}

fn runtime_aabb_intersects(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    collider: &RuntimeCollider,
) -> bool {
    x < collider.x + collider.width
        && x + width > collider.x
        && y < collider.y + collider.height
        && y + height > collider.y
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

impl Default for HighScoreTable {
    fn default() -> Self {
        Self {
            entries: (0..8)
                .map(|_| HighScoreEntry {
                    name: "UNKNOWN".to_string(),
                    score: 0,
                })
                .collect(),
        }
    }
}

impl HighScoreTable {
    fn submit(&mut self, name: &str, score: i32) {
        self.entries.push(HighScoreEntry {
            name: name.to_string(),
            score: score.max(0),
        });
        self.entries
            .sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
        self.entries.truncate(8);
    }

    fn name_at(&self, index: usize) -> &str {
        self.entries
            .get(index)
            .map(|entry| entry.name.as_str())
            .unwrap_or("UNKNOWN")
    }

    fn score_at(&self, index: usize) -> i32 {
        self.entries
            .get(index)
            .map(|entry| entry.score)
            .unwrap_or(0)
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

impl RuntimeApp {
    fn current_clear_color(&self) -> [f32; 4] {
        let camera = self
            .session
            .compiled
            .active_camera_for(&self.session.current_scene);
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

fn resolve_scene_name(compiled: &CompiledProject, requested: &str) -> String {
    if compiled.scene_exists(requested) {
        requested.to_string()
    } else if compiled.scene_exists(compiled.start_scene.as_str()) {
        compiled.start_scene.clone()
    } else {
        compiled.first_scene_name().unwrap_or("Main").to_string()
    }
}

fn runtime_rect_entity(rect: &RectNode) -> RuntimeEntity {
    RuntimeEntity {
        name: rect.name.clone(),
        template: rect.visual.template,
        visible: rect.visual.visible,
        spawn_time: 0.0,
        animation_start_time: 0.0,
        anchor: rect.visual.anchor,
        layer: rect.visual.layer,
        z: rect.visual.z,
        pos: rect.visual.pos,
        size: rect.visual.size,
        rotation: 0.0,
        color: rect.visual.color,
        group: rect.visual.group.clone(),
        scroll: [0.0, 0.0],
        repeat_x: false,
        repeat_y: false,
        flip_x: false,
        flip_y: false,
        physics: RuntimePhysics {
            mode: rpu_core::PhysicsMode::None,
            settings: rpu_core::PlatformerPhysicsSettings::default(),
            velocity: [0.0, 0.0],
            move_x: 0.0,
            jump_requested: false,
            grounded: false,
        },
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
        .or_else(|| markers.get(&sprite.name))
        .copied()
        .unwrap_or(sprite.visual.pos);

    RuntimeEntity {
        name: sprite.name.clone(),
        template: sprite.visual.template,
        visible: sprite.visual.visible,
        spawn_time: 0.0,
        animation_start_time: 0.0,
        anchor: sprite.visual.anchor,
        layer: sprite.visual.layer,
        z: sprite.visual.z,
        pos,
        size: sprite.visual.size,
        rotation: sprite.rotation,
        color: sprite.visual.color,
        group: sprite.visual.group.clone(),
        scroll: sprite.scroll,
        repeat_x: sprite.repeat_x,
        repeat_y: sprite.repeat_y,
        flip_x: sprite.flip_x,
        flip_y: sprite.flip_y,
        physics: RuntimePhysics {
            mode: sprite.physics,
            settings: sprite.physics_settings,
            velocity: [0.0, 0.0],
            move_x: 0.0,
            jump_requested: false,
            grounded: false,
        },
        script: sprite.visual.script_binding.clone(),
        script_state: HashMap::new(),
        kind: RuntimeEntityKind::Sprite {
            texture: sprite.textures.first().cloned(),
            frames: sprite.textures.clone(),
            current_animation: None,
            animations: sprite.animations.clone(),
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
        animation_start_time: 0.0,
        anchor: text.visual.anchor,
        layer: text.visual.layer,
        z: text.visual.z,
        pos: text.visual.pos,
        size: [0.0, 0.0],
        rotation: 0.0,
        color: text.visual.color,
        group: text.visual.group.clone(),
        scroll: [0.0, 0.0],
        repeat_x: false,
        repeat_y: false,
        flip_x: false,
        flip_y: false,
        physics: RuntimePhysics {
            mode: rpu_core::PhysicsMode::None,
            settings: rpu_core::PlatformerPhysicsSettings::default(),
            velocity: [0.0, 0.0],
            move_x: 0.0,
            jump_requested: false,
            grounded: false,
        },
        script: text.visual.script_binding.clone(),
        script_state: HashMap::new(),
        kind: RuntimeEntityKind::Text {
            value: text.value.clone(),
            font: text.font.clone(),
            font_size: text.font_size,
            align: text.align,
        },
    }
}

fn compile_static_map_commands(maps: &[AsciiMapNode], asset_base: &str) -> Vec<DrawCommand> {
    maps.iter()
        .flat_map(|map| {
            let legend: HashMap<char, &MapLegendMeaning> = map
                .legend
                .iter()
                .map(|entry| (entry.symbol, &entry.meaning))
                .collect();
            let classified = map.classify_terrain();
            let has_terrain = classified.cells.iter().any(|_| true);
            if has_terrain && !matches!(classified.render, rpu_core::TerrainRenderMode::Debug) {
                if let Some(command) = generated_terrain_map_command(map, &classified, asset_base) {
                    return vec![command];
                }
            }
            let terrain_cells: HashMap<(usize, usize), _> = classified
                .cells
                .iter()
                .map(|cell| ((cell.row, cell.col), cell))
                .collect();
            let mut commands = Vec::new();
            for (row, line) in map.rows.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    if matches!(ch, ' ' | '.') {
                        continue;
                    }
                    if let Some(MapLegendMeaning::Color(color)) = legend.get(&ch) {
                        commands.push(DrawCommand::Rect(SceneRect {
                            anchor: Anchor::World,
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
                    if let Some(MapLegendMeaning::Texture(texture)) = legend.get(&ch) {
                        commands.push(DrawCommand::Sprite(rpu_core::SceneSprite {
                            anchor: Anchor::World,
                            layer: -10,
                            z: (row as i32) * 100 + col as i32,
                            x: map.origin[0] + col as f32 * map.cell[0],
                            y: map.origin[1] + row as f32 * map.cell[1],
                            width: map.cell[0],
                            height: map.cell[1],
                            rotation: 0.0,
                            color: [1.0, 1.0, 1.0, 1.0],
                            textures: vec![texture.clone()],
                            animations: HashMap::new(),
                            animation_fps: 0.0,
                            animation_mode: rpu_core::AnimationMode::Loop,
                            destroy_on_animation_end: false,
                            scroll: [0.0, 0.0],
                            repeat_x: false,
                            repeat_y: false,
                            flip_x: false,
                            flip_y: false,
                            visible: true,
                        }));
                    }
                    if let Some(MapLegendMeaning::Terrain(_)) = legend.get(&ch) {
                        let Some(cell) = terrain_cells.get(&(row, col)) else {
                            continue;
                        };
                        let x = map.origin[0] + col as f32 * map.cell[0];
                        let y = map.origin[1] + row as f32 * map.cell[1];
                        let z = (row as i32) * 100 + col as i32;
                        commands.push(DrawCommand::Rect(SceneRect {
                            anchor: Anchor::World,
                            layer: -10,
                            z,
                            x,
                            y,
                            width: map.cell[0],
                            height: map.cell[1],
                            color: terrain_shape_debug_color(cell.shape),
                            visible: true,
                        }));
                        commands.extend(terrain_style_marker_commands(
                            x,
                            y,
                            map.cell[0],
                            map.cell[1],
                            z + 1,
                            cell.style,
                        ));
                    }
                }
            }
            commands
        })
        .collect()
}

pub fn render_terrain_map_image(
    map: &AsciiMapNode,
    asset_base: &str,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let classified = map.classify_terrain();
    if classified.width == 0 || classified.height == 0 || classified.cells.is_empty() {
        return None;
    }
    let cell_w = map.cell[0].round().max(1.0) as u32;
    let cell_h = map.cell[1].round().max(1.0) as u32;
    rasterize_terrain_map(map, &classified, asset_base, cell_w, cell_h)
}

pub fn render_terrain_fragment_image(map: &AsciiMapNode) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let classified = map.classify_terrain();
    if classified.width == 0 || classified.height == 0 || classified.cells.is_empty() {
        return None;
    }
    let cell_w = map.cell[0].round().max(1.0) as u32;
    let cell_h = map.cell[1].round().max(1.0) as u32;
    let width = classified.width as u32 * cell_w;
    let height = classified.height as u32 * cell_h;
    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([14, 18, 24, 255]));

    for cell in &classified.cells {
        let x0 = cell.col as u32 * cell_w;
        let y0 = cell.row as u32 * cell_h;
        for py in 0..cell_h {
            for px in 0..cell_w {
                let fragment =
                    runtime_terrain_fragment_for_pixel(cell, &map.terrain_style, px, py, cell_w);
                image.put_pixel(
                    x0 + px,
                    y0 + py,
                    runtime_terrain_fragment_debug_color(fragment),
                );
            }
        }
    }

    Some(image)
}

fn generated_terrain_map_command(
    map: &AsciiMapNode,
    classified: &rpu_core::ClassifiedAsciiMap,
    asset_base: &str,
) -> Option<DrawCommand> {
    let cell_w = map.cell[0].round().max(1.0) as u32;
    let cell_h = map.cell[1].round().max(1.0) as u32;
    let image = rasterize_terrain_map(map, classified, asset_base, cell_w, cell_h)?;
    let key = format!(
        "generated://terrain/{}:{}:{}x{}",
        sanitize_generated_key(asset_base),
        sanitize_generated_key(&map.name),
        image.width(),
        image.height()
    );
    register_generated_rgba_texture(&key, image.width(), image.height(), image.as_raw());
    Some(DrawCommand::Sprite(rpu_core::SceneSprite {
        anchor: Anchor::World,
        layer: -10,
        z: -10,
        x: map.origin[0],
        y: map.origin[1],
        width: image.width() as f32,
        height: image.height() as f32,
        rotation: 0.0,
        color: [1.0, 1.0, 1.0, 1.0],
        textures: vec![key],
        animations: HashMap::new(),
        animation_fps: 0.0,
        animation_mode: AnimationMode::Loop,
        destroy_on_animation_end: false,
        scroll: [0.0, 0.0],
        repeat_x: false,
        repeat_y: false,
        flip_x: false,
        flip_y: false,
        visible: true,
    }))
}

fn sanitize_generated_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn rasterize_terrain_map(
    map: &AsciiMapNode,
    classified: &rpu_core::ClassifiedAsciiMap,
    asset_base: &str,
    cell_w: u32,
    cell_h: u32,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    if classified.width == 0 || classified.height == 0 {
        return None;
    }
    let width = classified.width as u32 * cell_w;
    let height = classified.height as u32 * cell_h;
    let mut image = ImageBuffer::from_pixel(width.max(1), height.max(1), rgba([0, 0, 0, 0]));
    let use_synth = matches!(classified.render, rpu_core::TerrainRenderMode::Synth);
    let material_fields = runtime_build_material_fields(asset_base, classified, use_synth);
    let region_body_fields =
        runtime_build_region_body_fields(asset_base, classified, cell_w, cell_h, use_synth);
    let strip_fields =
        runtime_build_surface_strip_fields(asset_base, classified, cell_w, use_synth);
    let region_lookup: HashMap<usize, &rpu_core::TerrainRegion> = classified
        .regions
        .iter()
        .map(|region| (region.id, region))
        .collect();

    for cell in &classified.cells {
        let Some(region) = region_lookup.get(&cell.region_id).copied() else {
            continue;
        };
        let x0 = cell.col as u32 * cell_w;
        let y0 = cell.row as u32 * cell_h;
        for py in 0..cell_h {
            for px in 0..cell_w {
                let color = rasterize_terrain_pixel(
                    &material_fields,
                    &region_body_fields,
                    &strip_fields,
                    &map.terrain_style,
                    cell,
                    region,
                    px,
                    py,
                    cell_w,
                    use_synth,
                );
                image.put_pixel(x0 + px, y0 + py, color);
            }
        }
    }

    let _ = map;
    Some(image)
}

fn rasterize_terrain_pixel(
    material_fields: &HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    region_body_fields: &HashMap<(usize, String), ImageBuffer<Rgba<u8>, Vec<u8>>>,
    strip_fields: &HashMap<usize, RegionSurfaceStrips>,
    style: &rpu_core::TerrainStyleSettings,
    cell: &rpu_core::ClassifiedMapCell,
    region: &rpu_core::TerrainRegion,
    px: u32,
    py: u32,
    tile: u32,
    use_synth_variation: bool,
) -> Rgba<u8> {
    let (u, v) = runtime_region_space_projection_for_cell(cell, region, style, px, py, tile);
    let fragment = runtime_terrain_fragment_for_pixel(cell, style, px, py, tile);
    if matches!(fragment, TerrainPixelFragment::Empty) {
        return rgba([0, 0, 0, 0]);
    }
    let local_inward = runtime_pixel_local_inward(cell, style, px, py, tile);
    let top_material = runtime_top_material_for_stack(&cell.material_key);
    let body_material = runtime_body_material_for_cell(cell);
    let body = runtime_sample_body_material(
        material_fields,
        region_body_fields,
        body_material,
        cell,
        region,
        u,
        v,
        use_synth_variation,
    );
    if matches!(
        fragment,
        TerrainPixelFragment::FlatCap
            | TerrainPixelFragment::RampCap
            | TerrainPixelFragment::JoinCap
    ) {
        let cap_depth = runtime_cap_depth_for_cell(cell, style, tile);
        let top = if let Some(strips) = strip_fields.get(&cell.region_id) {
            runtime_sample_surface_strip_pixel(
                strips,
                cell,
                px,
                local_inward,
                cap_depth,
                tile,
                use_synth_variation,
            )
        } else {
            runtime_sample_top_material(material_fields, top_material, u, local_inward, cap_depth)
        };
        return runtime_alpha_over(top, body);
    }
    body
}

#[derive(Clone, Copy)]
enum TerrainPixelFragment {
    Empty,
    Body,
    FlatCap,
    RampCap,
    JoinCap,
}

fn runtime_terrain_fragment_for_pixel(
    cell: &rpu_core::ClassifiedMapCell,
    style: &rpu_core::TerrainStyleSettings,
    px: u32,
    py: u32,
    tile: u32,
) -> TerrainPixelFragment {
    let surface_y = runtime_surface_height_for_cell(cell, style, px, tile);
    if py < surface_y {
        return TerrainPixelFragment::Empty;
    }
    let top_material = runtime_top_material_for_stack(&cell.material_key);
    let is_surface_cell = cell.material == top_material
        && matches!(
            cell.normal,
            rpu_core::TerrainNormal::Up
                | rpu_core::TerrainNormal::UpLeft
                | rpu_core::TerrainNormal::UpRight
        );
    if !is_surface_cell {
        return TerrainPixelFragment::Body;
    }
    let local_inward = py - surface_y;
    if local_inward >= runtime_cap_depth_for_cell(cell, style, tile) {
        return TerrainPixelFragment::Body;
    }
    match (cell.contour, cell.transition_role) {
        (rpu_core::TerrainContour::RampUpLeft, _)
        | (rpu_core::TerrainContour::RampUpRight, _)
        | (_, rpu_core::TerrainTransitionRole::RampUpLeft)
        | (_, rpu_core::TerrainTransitionRole::RampUpRight) => TerrainPixelFragment::RampCap,
        (_, rpu_core::TerrainTransitionRole::JoinFromLeft)
        | (_, rpu_core::TerrainTransitionRole::JoinFromRight)
        | (_, rpu_core::TerrainTransitionRole::JoinBoth) => TerrainPixelFragment::JoinCap,
        _ => TerrainPixelFragment::FlatCap,
    }
}

fn runtime_pixel_local_inward(
    cell: &rpu_core::ClassifiedMapCell,
    style: &rpu_core::TerrainStyleSettings,
    px: u32,
    py: u32,
    tile: u32,
) -> u32 {
    py.saturating_sub(runtime_surface_height_for_cell(cell, style, px, tile))
}

fn runtime_terrain_fragment_debug_color(fragment: TerrainPixelFragment) -> Rgba<u8> {
    match fragment {
        TerrainPixelFragment::Empty => rgba([14, 18, 24, 255]),
        TerrainPixelFragment::Body => rgba([132, 86, 54, 255]),
        TerrainPixelFragment::FlatCap => rgba([68, 221, 87, 255]),
        TerrainPixelFragment::RampCap => rgba([58, 196, 222, 255]),
        TerrainPixelFragment::JoinCap => rgba([245, 206, 74, 255]),
    }
}

fn runtime_sample_body_material(
    material_fields: &HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    region_body_fields: &HashMap<(usize, String), ImageBuffer<Rgba<u8>, Vec<u8>>>,
    material: &str,
    cell: &rpu_core::ClassifiedMapCell,
    region: &rpu_core::TerrainRegion,
    u: u32,
    v: u32,
    use_synth_variation: bool,
) -> Rgba<u8> {
    let base = runtime_sample_region_or_material_field(
        material_fields,
        region_body_fields,
        region.id,
        material,
        cell,
        region,
        u,
        v,
    );
    if !use_synth_variation {
        return base;
    }

    // Break up the straight vertical wallpaper effect while staying continuous in region space.
    let seed = runtime_hash_material_seed(
        material,
        region.id as u32,
        (region.min_col as u32)
            .wrapping_mul(31)
            .wrapping_add(region.min_row as u32),
    );
    let skew_u = u
        .wrapping_add(v / 3)
        .wrapping_add((seed & 31) * 3)
        .wrapping_add(cell.surface_u * 2);
    let skew_v = v
        .wrapping_add(u / 7)
        .wrapping_add(((seed >> 5) & 31) * 2)
        .wrapping_add(cell.boundary_distance * 3);
    let detail = runtime_sample_region_or_material_field(
        material_fields,
        region_body_fields,
        region.id,
        material,
        cell,
        region,
        skew_u,
        skew_v,
    );

    let cross_u = u.wrapping_add(v / 5).wrapping_add(((seed >> 10) & 15) * 5);
    let cross_v = v.wrapping_add(u / 11).wrapping_add(((seed >> 14) & 15) * 4);
    let cross = runtime_sample_region_or_material_field(
        material_fields,
        region_body_fields,
        region.id,
        material,
        cell,
        region,
        cross_u,
        cross_v,
    );

    let rotated_u = v.wrapping_add(u / 4).wrapping_add(((seed >> 19) & 15) * 7);
    let rotated_v = u.wrapping_add(v / 9).wrapping_add(((seed >> 23) & 15) * 6);
    let rotated = runtime_sample_region_or_material_field(
        material_fields,
        region_body_fields,
        region.id,
        material,
        cell,
        region,
        rotated_u,
        rotated_v,
    );

    let mixed = runtime_lerp_rgba(base, detail, 110);
    let mixed = runtime_lerp_rgba(mixed, cross, 72);
    let mut mixed = runtime_lerp_rgba(mixed, rotated, 84);

    let deep_material = runtime_deep_material_for_stack(&cell.material_key);
    if deep_material != material
        && matches!(
            cell.depth_band,
            rpu_core::TerrainDepthBand::Interior | rpu_core::TerrainDepthBand::DeepInterior
        )
    {
        let deep = runtime_sample_region_or_material_field(
            material_fields,
            region_body_fields,
            region.id,
            deep_material,
            cell,
            region,
            rotated_u.wrapping_add(v / 6),
            rotated_v.wrapping_add(u / 8),
        );
        if matches!(cell.depth_band, rpu_core::TerrainDepthBand::DeepInterior) {
            return deep;
        }
        let max_depth = region.max_boundary_distance.max(1);
        let depth_ratio = ((cell.boundary_distance.min(max_depth) * 255) / max_depth) as u8;
        let depth_blend = match cell.depth_band {
            rpu_core::TerrainDepthBand::Interior => depth_ratio.saturating_add(32).max(128),
            rpu_core::TerrainDepthBand::DeepInterior => 255,
            _ => 0,
        };
        mixed = runtime_lerp_rgba(mixed, deep, depth_blend);
    }

    mixed
}

fn runtime_sample_region_or_material_field(
    material_fields: &HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    region_body_fields: &HashMap<(usize, String), ImageBuffer<Rgba<u8>, Vec<u8>>>,
    region_id: usize,
    material: &str,
    _cell: &rpu_core::ClassifiedMapCell,
    _region: &rpu_core::TerrainRegion,
    u: u32,
    v: u32,
) -> Rgba<u8> {
    if let Some(image) = region_body_fields.get(&(region_id, material.to_string())) {
        let x = u % image.width().max(1);
        let y = v % image.height().max(1);
        return *image.get_pixel(x, y);
    }
    runtime_sample_material_field(material_fields, material, u, v)
}

struct RegionSurfaceStrips {
    flat: ImageBuffer<Rgba<u8>, Vec<u8>>,
    join: ImageBuffer<Rgba<u8>, Vec<u8>>,
    join_left: ImageBuffer<Rgba<u8>, Vec<u8>>,
    join_right: ImageBuffer<Rgba<u8>, Vec<u8>>,
    ramp_left: ImageBuffer<Rgba<u8>, Vec<u8>>,
    ramp_right: ImageBuffer<Rgba<u8>, Vec<u8>>,
    solved: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

fn runtime_build_material_fields(
    asset_base: &str,
    classified: &rpu_core::ClassifiedAsciiMap,
    use_synth_fields: bool,
) -> HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut fields = HashMap::new();
    let materials: std::collections::HashSet<_> = classified
        .cells
        .iter()
        .flat_map(|cell| {
            cell.material_key
                .split('>')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .collect();
    for material in materials {
        let source = runtime_load_material_source(asset_base, &material)
            .unwrap_or_else(|| runtime_builtin_material_image(&material));
        let image = if use_synth_fields {
            runtime_wfc_material_field(&material, &source)
                .unwrap_or_else(|| runtime_quilt_material_field(&material, &source))
        } else {
            source
        };
        fields.insert(material, image);
    }
    fields
}

fn runtime_build_region_body_fields(
    asset_base: &str,
    classified: &rpu_core::ClassifiedAsciiMap,
    tile_w: u32,
    tile_h: u32,
    use_synth_fields: bool,
) -> HashMap<(usize, String), ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut fields = HashMap::new();
    let cells_by_region: HashMap<usize, &rpu_core::ClassifiedMapCell> = classified
        .cells
        .iter()
        .map(|cell| (cell.region_id, cell))
        .collect();

    for region in &classified.regions {
        let Some(cell) = cells_by_region.get(&region.id).copied() else {
            continue;
        };
        let stack_materials: std::collections::HashSet<String> = cell
            .material_key
            .split('>')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        for material in stack_materials {
            let source = runtime_load_material_source(asset_base, &material)
                .unwrap_or_else(|| runtime_builtin_material_image(&material));
            let width = ((region.max_col.saturating_sub(region.min_col) + 1) as u32)
                .saturating_mul(tile_w)
                .max(source.width());
            let height = ((region.max_row.saturating_sub(region.min_row) + 1) as u32)
                .saturating_mul(tile_h)
                .max(source.height());
            let image = if use_synth_fields {
                let solve_width = width.min(96).max(source.width());
                let solve_height = height.min(96).max(source.height());
                runtime_wfc_source_field(
                    &format!("{}:region_body:{}", material, region.id),
                    &source,
                    solve_width,
                    solve_height,
                    1,
                )
                .map(|solved| {
                    runtime_quilt_image_to_size(
                        &format!("{}:region_body_expand:{}", material, region.id),
                        &solved,
                        width,
                        height,
                    )
                })
                .unwrap_or_else(|| {
                    runtime_quilt_image_to_size(
                        &format!("{}:region_body:{}", material, region.id),
                        &source,
                        width,
                        height,
                    )
                })
            } else {
                runtime_quilt_image_to_size(
                    &format!("{}:region_body:{}", material, region.id),
                    &source,
                    width,
                    height,
                )
            };
            fields.insert((region.id, material), image);
        }
    }

    fields
}

fn runtime_build_surface_strip_fields(
    asset_base: &str,
    classified: &rpu_core::ClassifiedAsciiMap,
    tile: u32,
    use_synth_solve: bool,
) -> HashMap<usize, RegionSurfaceStrips> {
    let mut fields = HashMap::new();
    let cells_by_region: HashMap<usize, &rpu_core::ClassifiedMapCell> = classified
        .cells
        .iter()
        .map(|cell| (cell.region_id, cell))
        .collect();

    for region in &classified.regions {
        let Some(cell) = cells_by_region.get(&region.id).copied() else {
            continue;
        };
        let source = runtime_build_surface_strip_source(asset_base, &cell.material_key);
        let ramp_left_source = runtime_build_ramp_strip_source(&source, -1);
        let ramp_right_source = runtime_build_ramp_strip_source(&source, 1);
        let width = (region.boundary_loop.len().max(1) as u32)
            .saturating_mul(tile)
            .max(source.width());
        let flat =
            runtime_solve_surface_strip_1d(&format!("{}:flat", &cell.material_key), &source, width)
                .unwrap_or_else(|| {
                    runtime_quilt_surface_strip_horizontally(
                        &format!("{}:flat", &cell.material_key),
                        &source,
                        width,
                    )
                });
        let (join_left_source, join_right_source, join_source) =
            runtime_build_join_strip_sources(&source, &ramp_left_source, &ramp_right_source);
        let join_left = runtime_solve_surface_strip_1d(
            &format!("{}:join_left", &cell.material_key),
            &join_left_source,
            width,
        )
        .unwrap_or_else(|| {
            runtime_quilt_surface_strip_horizontally(
                &format!("{}:join_left", &cell.material_key),
                &join_left_source,
                width,
            )
        });
        let join_right = runtime_solve_surface_strip_1d(
            &format!("{}:join_right", &cell.material_key),
            &join_right_source,
            width,
        )
        .unwrap_or_else(|| {
            runtime_quilt_surface_strip_horizontally(
                &format!("{}:join_right", &cell.material_key),
                &join_right_source,
                width,
            )
        });
        let join = runtime_solve_surface_strip_1d(
            &format!("{}:join", &cell.material_key),
            &join_source,
            width,
        )
        .unwrap_or_else(|| {
            runtime_quilt_surface_strip_horizontally(
                &format!("{}:join", &cell.material_key),
                &join_source,
                width,
            )
        });
        let ramp_left = runtime_tile_surface_strip_horizontally(&ramp_left_source, width);
        let ramp_right = runtime_tile_surface_strip_horizontally(&ramp_right_source, width);
        let solved = if use_synth_solve {
            runtime_solve_state_constrained_surface_strip_2d(
                &format!("{}:surface", &cell.material_key),
                region,
                classified,
                tile,
                &flat,
                &join,
                &ramp_right,
            )
            .unwrap_or_else(|| flat.clone())
        } else {
            flat.clone()
        };
        fields.insert(
            region.id,
            RegionSurfaceStrips {
                flat,
                join,
                join_left,
                join_right,
                ramp_left,
                ramp_right,
                solved,
            },
        );
    }

    fields
}

fn runtime_build_surface_strip_source(
    asset_base: &str,
    stack_key: &str,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let materials = stack_key
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let top = materials.first().copied().unwrap_or("rock");
    let body = materials.get(1).copied().unwrap_or(top);

    let top_source = runtime_load_material_source(asset_base, top)
        .unwrap_or_else(|| runtime_builtin_material_image(top));
    let body_source = runtime_load_material_source(asset_base, body)
        .unwrap_or_else(|| runtime_builtin_material_image(body));

    let width = top_source.width().max(body_source.width()).max(1);
    let top_h = if top == "grass" {
        (top_source.height().max(1) / 2).max(1)
    } else {
        top_source.height().max(1)
    };
    let body_h = body_source.height().max(1);
    let body_sample_h = (body_h / 2).max(1);
    let height = top_h + body_sample_h;
    let mut image = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));

    for y in 0..top_h {
        for x in 0..width {
            let sy = if top == "grass" && top_source.height() > top_h {
                top_source.height() - top_h + y
            } else {
                y % top_source.height().max(1)
            };
            let p = *top_source.get_pixel(x % top_source.width().max(1), sy);
            image.put_pixel(x, y, p);
        }
    }
    for y in 0..body_sample_h {
        for x in 0..width {
            let sy = (body_source.height().saturating_sub(body_sample_h) + y)
                % body_source.height().max(1);
            let p = *body_source.get_pixel(x % body_source.width().max(1), sy);
            image.put_pixel(x, top_h + y, p);
        }
    }

    image
}

fn runtime_build_join_strip_sources(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ramp_left_source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ramp_right_source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> (
    ImageBuffer<Rgba<u8>, Vec<u8>>,
    ImageBuffer<Rgba<u8>, Vec<u8>>,
    ImageBuffer<Rgba<u8>, Vec<u8>>,
) {
    let width = source.width().max(1);
    let height = source.height().max(1);
    let mut left = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    let mut right = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    let mut center = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    for y in 0..height {
        for x in 0..width {
            let flat = *source.get_pixel(x % width, y % height);
            let x_t = x as f32 / width.saturating_sub(1).max(1) as f32;
            let y_t = y as f32 / height.saturating_sub(1).max(1) as f32;

            let left_depth_shift = ((1.0 - x_t) * y_t * height as f32 * 0.55).round() as u32;
            let right_depth_shift = (x_t * y_t * height as f32 * 0.55).round() as u32;
            let left_ramp_y = (y + left_depth_shift).min(height.saturating_sub(1));
            let right_ramp_y = (y + right_depth_shift).min(height.saturating_sub(1));
            let left_ramp = *ramp_left_source.get_pixel(x % width, left_ramp_y);
            let right_ramp = *ramp_right_source.get_pixel(x % width, right_ramp_y);

            let left_weight = ((1.0 - x_t).powf(1.35) * 255.0).round() as u8;
            let right_weight = (x_t.powf(1.35) * 255.0).round() as u8;
            let left_px = runtime_lerp_rgba(flat, left_ramp, left_weight);
            let right_px = runtime_lerp_rgba(flat, right_ramp, right_weight);
            let center_px = runtime_lerp_rgba(left_px, right_px, 128);
            left.put_pixel(x, y, left_px);
            right.put_pixel(x, y, right_px);
            center.put_pixel(x, y, center_px);
        }
    }
    (left, right, center)
}

fn runtime_build_ramp_strip_source(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    direction: i32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let width = source.width().max(1);
    let height = source.height().max(1);
    let mut image = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    for y in 0..height {
        let t = y as f32 / (height.saturating_sub(1).max(1)) as f32;
        let shift = (t * (width as f32 * 0.35)).round() as u32;
        for x in 0..width {
            let src_x = if direction < 0 {
                x.wrapping_add(width).wrapping_sub(shift % width) % width
            } else {
                (x + shift) % width
            };
            let base = *source.get_pixel(src_x, y);
            let next_x = if direction < 0 {
                src_x.wrapping_add(width).wrapping_sub(1) % width
            } else {
                (src_x + 1) % width
            };
            let next = *source.get_pixel(next_x, y);
            let blended = runtime_lerp_rgba(base, next, (t * 160.0).round() as u8);
            image.put_pixel(x, y, blended);
        }
    }
    image
}

fn runtime_load_material_source(
    asset_base: &str,
    material: &str,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let base = Path::new(asset_base);
    let candidates = [
        base.join(format!("{material}.png")),
        base.join("terrain").join(format!("{material}.png")),
        base.join(format!("terrain_{material}.png")),
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

fn runtime_sample_material_field(
    material_fields: &HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    material: &str,
    u: u32,
    v: u32,
) -> Rgba<u8> {
    if let Some(image) = material_fields.get(material) {
        let x = u % image.width().max(1);
        let y = v % image.height().max(1);
        return *image.get_pixel(x, y);
    }
    runtime_sample_material_exemplar(material, u, v)
}

fn runtime_sample_top_material(
    material_fields: &HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    material: &str,
    u: u32,
    local_inward: u32,
    cap_depth: u32,
) -> Rgba<u8> {
    let source_h = material_fields
        .get(material)
        .map(|image| image.height().max(1))
        .unwrap_or(16);
    let sampled_h = if material == "grass" {
        (source_h / 2).max(1)
    } else {
        source_h
    };
    let sampled_offset = if material == "grass" && source_h > sampled_h {
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
    runtime_sample_material_field(material_fields, material, u, cap_v)
}

fn runtime_sample_surface_strip_pixel(
    strips: &RegionSurfaceStrips,
    cell: &rpu_core::ClassifiedMapCell,
    px: u32,
    local_inward: u32,
    cap_depth: u32,
    tile: u32,
    use_synth_variation: bool,
) -> Rgba<u8> {
    let (sample_px, sample_inward) = runtime_cap_sample_coords(cell, px, local_inward, tile);
    let solved_u = runtime_surface_strip_u_for_cell(cell, sample_px, sample_inward, tile);
    let solved_v = runtime_cap_sample_v(&strips.solved, sample_inward, cap_depth);
    let solved = runtime_sample_stack_field(&strips.solved, solved_u, solved_v);
    let flat = runtime_sample_stack_field(&strips.flat, solved_u, solved_v);
    let join = runtime_sample_stack_field(&strips.join, solved_u, solved_v);
    let join_left = runtime_sample_stack_field(&strips.join_left, solved_u, solved_v);
    let join_right = runtime_sample_stack_field(&strips.join_right, solved_u, solved_v);
    let ramp_left = runtime_sample_stack_field(&strips.ramp_left, solved_u, solved_v);
    let ramp_right = runtime_sample_stack_field(&strips.ramp_right, solved_u, solved_v);
    let anchored = match (cell.contour, cell.transition_role) {
        (rpu_core::TerrainContour::RampUpRight, _) => {
            let t = (px.saturating_mul(255) / tile.saturating_sub(1).max(1)) as u8;
            runtime_lerp_rgba(ramp_right, join_right, t)
        }
        (rpu_core::TerrainContour::RampUpLeft, _) => {
            let t = ((tile.saturating_sub(1).saturating_sub(px)).saturating_mul(255)
                / tile.saturating_sub(1).max(1)) as u8;
            runtime_lerp_rgba(ramp_left, join_left, t)
        }
        (_, rpu_core::TerrainTransitionRole::JoinFromLeft) => {
            let t = ((tile.saturating_sub(1).saturating_sub(px)).saturating_mul(255)
                / tile.saturating_sub(1).max(1)) as u8;
            runtime_lerp_rgba(flat, join_left, t)
        }
        (_, rpu_core::TerrainTransitionRole::JoinFromRight) => {
            let t = (px.saturating_mul(255) / tile.saturating_sub(1).max(1)) as u8;
            runtime_lerp_rgba(flat, join_right, t)
        }
        (_, rpu_core::TerrainTransitionRole::JoinBoth) => {
            let center = tile.saturating_sub(1) as f32 * 0.5;
            let distance = ((px as f32 - center).abs() / center.max(1.0)).clamp(0.0, 1.0);
            let t = ((1.0 - distance) * 255.0).round() as u8;
            runtime_lerp_rgba(flat, join, t)
        }
        _ => flat,
    };

    let blend = match (cell.contour, cell.transition_role) {
        (rpu_core::TerrainContour::RampUpLeft, _) | (rpu_core::TerrainContour::RampUpRight, _) => {
            72
        }
        (_, rpu_core::TerrainTransitionRole::JoinFromLeft)
        | (_, rpu_core::TerrainTransitionRole::JoinFromRight)
        | (_, rpu_core::TerrainTransitionRole::JoinBoth) => 96,
        _ => 168,
    };
    if use_synth_variation {
        runtime_lerp_rgba(anchored, solved, blend)
    } else {
        anchored
    }
}

fn runtime_cap_sample_coords(
    cell: &rpu_core::ClassifiedMapCell,
    px: u32,
    local_inward: u32,
    tile: u32,
) -> (u32, u32) {
    let max = tile.saturating_sub(1).max(1);
    match cell.contour {
        rpu_core::TerrainContour::RampUpRight => {
            let along = ((px + max.saturating_sub(local_inward.min(max))) / 2).min(max);
            (along, local_inward)
        }
        rpu_core::TerrainContour::RampUpLeft => {
            let along =
                ((max.saturating_sub(px) + max.saturating_sub(local_inward.min(max))) / 2).min(max);
            (max.saturating_sub(along), local_inward)
        }
        _ => (px, local_inward),
    }
}

fn runtime_cap_sample_v(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    local_inward: u32,
    cap_depth: u32,
) -> u32 {
    if cap_depth <= 1 {
        0
    } else {
        (local_inward.saturating_mul(image.height().saturating_sub(1)))
            / cap_depth.saturating_sub(1)
    }
}

fn runtime_surface_strip_u_for_cell(
    cell: &rpu_core::ClassifiedMapCell,
    px: u32,
    local_inward: u32,
    tile: u32,
) -> u32 {
    let along = runtime_along_surface_projection(cell.tangent, px, local_inward, tile);
    let base = cell.surface_u.saturating_mul(tile).saturating_add(along);
    let skew = match cell.contour {
        rpu_core::TerrainContour::RampUpRight => local_inward,
        rpu_core::TerrainContour::RampUpLeft => tile.saturating_sub(1).saturating_sub(local_inward),
        rpu_core::TerrainContour::FlatTop => match cell.transition_role {
            rpu_core::TerrainTransitionRole::JoinFromLeft => local_inward.saturating_mul(3) / 4,
            rpu_core::TerrainTransitionRole::JoinFromRight => tile
                .saturating_sub(1)
                .saturating_sub(local_inward.saturating_mul(3) / 4),
            rpu_core::TerrainTransitionRole::JoinBoth => tile / 2,
            _ => 0,
        },
        _ => 0,
    };
    match (cell.contour, cell.transition_role) {
        (rpu_core::TerrainContour::RampUpLeft, _)
        | (rpu_core::TerrainContour::FlatTop, rpu_core::TerrainTransitionRole::JoinFromRight) => {
            base.saturating_sub(tile.saturating_sub(1).saturating_sub(skew))
        }
        _ => base.saturating_add(skew),
    }
}

fn runtime_sample_stack_field(field: &ImageBuffer<Rgba<u8>, Vec<u8>>, u: u32, v: u32) -> Rgba<u8> {
    let x = u % field.width().max(1);
    let y = v % field.height().max(1);
    *field.get_pixel(x, y)
}

fn runtime_top_material_for_stack(stack_key: &str) -> &str {
    stack_key
        .split('>')
        .map(str::trim)
        .find(|part| !part.is_empty())
        .unwrap_or("rock")
}

fn runtime_body_material_for_cell<'a>(cell: &'a rpu_core::ClassifiedMapCell) -> &'a str {
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

fn runtime_deep_material_for_stack(stack_key: &str) -> &str {
    stack_key
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .last()
        .unwrap_or("rock")
}

fn runtime_cap_depth_for_cell(
    cell: &rpu_core::ClassifiedMapCell,
    style: &rpu_core::TerrainStyleSettings,
    tile: u32,
) -> u32 {
    let (depth, min_depth) = match (cell.contour, cell.transition_role) {
        (rpu_core::TerrainContour::RampUpLeft, _) | (rpu_core::TerrainContour::RampUpRight, _) => {
            (style.ramp_cap_depth, 6)
        }
        (_, rpu_core::TerrainTransitionRole::JoinFromLeft)
        | (_, rpu_core::TerrainTransitionRole::JoinFromRight)
        | (_, rpu_core::TerrainTransitionRole::JoinBoth) => (style.join_cap_depth, 8),
        _ => (style.cap_depth, 8),
    };
    let base = ((tile as f32) * depth).round() as u32;
    let cap_variation = (cell.surface_u % 5) as i32 - 2;
    (base as i32 + cap_variation).max(min_depth) as u32
}

fn runtime_surface_height_for_cell(
    cell: &rpu_core::ClassifiedMapCell,
    style: &rpu_core::TerrainStyleSettings,
    px: u32,
    tile: u32,
) -> u32 {
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

    let base = match cell.transition_role {
        rpu_core::TerrainTransitionRole::RampUpRight
        | rpu_core::TerrainTransitionRole::RampUpLeft => ramp,
        rpu_core::TerrainTransitionRole::JoinFromLeft => {
            runtime_shoulder_height(max.saturating_sub(x), max, style)
        }
        rpu_core::TerrainTransitionRole::JoinFromRight => runtime_shoulder_height(x, max, style),
        rpu_core::TerrainTransitionRole::JoinBoth => {
            runtime_shoulder_height(x.min(max.saturating_sub(x)), max / 2, style)
        }
        rpu_core::TerrainTransitionRole::None => flat,
    };
    let roughness = runtime_surface_profile_offset(cell, style, px, tile);
    base.saturating_add(roughness).min(max)
}

fn runtime_shoulder_height(raw: u32, max: u32, style: &rpu_core::TerrainStyleSettings) -> u32 {
    let width = style.shoulder_width.clamp(0.0, 1.0);
    if width <= f32::EPSILON {
        return 0;
    }
    let scaled_max = ((max as f32) * width).round().max(1.0);
    let t = (raw as f32 / scaled_max).clamp(0.0, 1.0);
    let shaped = match style.shoulder_shape {
        rpu_core::TerrainShoulderShape::Linear => t,
        rpu_core::TerrainShoulderShape::Bend => t * t * (3.0 - 2.0 * t),
    };
    (shaped * max as f32).round().clamp(0.0, max as f32) as u32
}

fn runtime_surface_profile_offset(
    cell: &rpu_core::ClassifiedMapCell,
    style: &rpu_core::TerrainStyleSettings,
    px: u32,
    tile: u32,
) -> u32 {
    if style.surface_roughness <= f32::EPSILON {
        return 0;
    }
    if !matches!(
        cell.contour,
        rpu_core::TerrainContour::FlatTop
            | rpu_core::TerrainContour::RampUpLeft
            | rpu_core::TerrainContour::RampUpRight
    ) {
        return 0;
    }

    let max = tile.saturating_sub(1).max(1);
    let x = px.min(max);
    let edge_fade = match cell.contour {
        rpu_core::TerrainContour::RampUpLeft | rpu_core::TerrainContour::RampUpRight => {
            let edge = x.min(max.saturating_sub(x)) as f32 / (tile as f32 * 0.18).max(1.0);
            edge.clamp(0.0, 1.0)
        }
        _ => 1.0,
    };
    let amplitude = (tile as f32 * style.surface_roughness)
        .round()
        .clamp(0.0, 8.0);
    if amplitude <= 0.0 || edge_fade <= 0.0 {
        return 0;
    }

    let world_x = (cell.col as u32).saturating_mul(tile).saturating_add(x);
    let period = (tile / 5).max(5);
    let a = runtime_surface_noise_value(world_x / period, cell.region_id as u32);
    let b = runtime_surface_noise_value(world_x / period + 1, cell.region_id as u32);
    let t = (world_x % period) as f32 / period as f32;
    let smooth_t = t * t * (3.0 - 2.0 * t);
    let noise = a * (1.0 - smooth_t) + b * smooth_t;
    (noise.max(0.0) * amplitude * edge_fade).round() as u32
}

fn runtime_surface_noise_value(x: u32, seed: u32) -> f32 {
    let mut n = x
        .wrapping_mul(0x9e37_79b1)
        .wrapping_add(seed.wrapping_mul(0x85eb_ca6b));
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb_352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846c_a68b);
    n ^= n >> 16;
    (n as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn runtime_region_space_projection_for_cell(
    cell: &rpu_core::ClassifiedMapCell,
    region: &rpu_core::TerrainRegion,
    style: &rpu_core::TerrainStyleSettings,
    px: u32,
    py: u32,
    tile: u32,
) -> (u32, u32) {
    let region_x = (cell.col.saturating_sub(region.min_col) as u32) * tile + px;
    let local_inward = py.saturating_sub(runtime_surface_height_for_cell(cell, style, px, tile));
    let inward = cell.boundary_distance * tile + local_inward;
    (region_x, inward)
}

fn runtime_along_surface_projection(
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
        rpu_core::TerrainTangent::UpLeft => {
            (tile.saturating_sub(1).saturating_sub(px) + tile.saturating_sub(1).saturating_sub(py))
                / 2
        }
        rpu_core::TerrainTangent::UpRight => (px + tile.saturating_sub(1).saturating_sub(py)) / 2,
        rpu_core::TerrainTangent::DownLeft => (tile.saturating_sub(1).saturating_sub(px) + py) / 2,
        rpu_core::TerrainTangent::DownRight => (px + py) / 2,
    }
}

fn runtime_alpha_over(top: Rgba<u8>, bottom: Rgba<u8>) -> Rgba<u8> {
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

fn runtime_sample_material_exemplar(material: &str, u: u32, v: u32) -> Rgba<u8> {
    let (pattern, palette) = runtime_material_exemplar(material);
    let w = pattern[0].len() as u32;
    let h = pattern.len() as u32;
    let ix = (u % w) as usize;
    let iy = (v % h) as usize;
    rgba(palette[pattern[iy][ix] as usize])
}

fn runtime_builtin_material_image(material: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (pattern, palette) = runtime_material_exemplar(material);
    let width = pattern[0].len() as u32;
    let height = pattern.len() as u32;
    let mut image = ImageBuffer::from_pixel(width, height, rgba([0, 0, 0, 0]));
    for y in 0..height {
        for x in 0..width {
            image.put_pixel(
                x,
                y,
                rgba(palette[pattern[y as usize][x as usize] as usize]),
            );
        }
    }
    image
}

fn runtime_material_exemplar(material: &str) -> (&'static [&'static [u8]], &'static [[u8; 4]]) {
    match material {
        "grass" => (
            &[
                &[0, 0, 1, 0, 1, 0, 1, 0],
                &[0, 1, 2, 1, 2, 1, 2, 1],
                &[2, 2, 3, 2, 3, 2, 3, 2],
                &[3, 3, 4, 3, 4, 3, 4, 3],
                &[4, 4, 4, 4, 4, 4, 4, 4],
                &[4, 4, 4, 4, 4, 4, 4, 4],
            ],
            &[
                [0, 0, 0, 0],
                [116, 214, 95, 255],
                [79, 176, 62, 255],
                [53, 118, 43, 255],
                [126, 82, 49, 255],
            ],
        ),
        "dirt" => (
            &[
                &[0, 1, 0, 0, 1, 0, 0, 1],
                &[1, 2, 1, 0, 2, 1, 0, 2],
                &[0, 1, 2, 1, 0, 2, 1, 0],
                &[2, 1, 0, 2, 1, 0, 2, 1],
                &[1, 0, 2, 1, 0, 2, 1, 0],
                &[0, 2, 1, 0, 2, 1, 0, 2],
            ],
            &[[114, 74, 48, 255], [138, 92, 59, 255], [93, 58, 38, 255]],
        ),
        _ => (
            &[
                &[0, 1, 0, 2, 0, 1, 0, 2],
                &[1, 0, 2, 0, 1, 0, 2, 0],
                &[0, 2, 0, 1, 0, 2, 0, 1],
                &[2, 0, 1, 0, 2, 0, 1, 0],
                &[0, 1, 0, 2, 0, 1, 0, 2],
                &[1, 0, 2, 0, 1, 0, 2, 0],
            ],
            &[[78, 76, 88, 255], [108, 104, 122, 255], [57, 56, 65, 255]],
        ),
    }
}

fn rgba(color: [u8; 4]) -> Rgba<u8> {
    Rgba(color)
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct RuntimeStripPattern {
    pixels: Vec<[u8; 4]>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RuntimeStripState {
    Flat,
    Join,
    Ramp,
}

#[derive(Clone)]
struct RuntimeStateWfcPattern {
    pixels: Vec<[u8; 4]>,
    state: RuntimeStripState,
}

fn runtime_solve_surface_strip_1d(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let pattern_width = 6usize.min(source.width().max(1) as usize);
    if pattern_width < 2 {
        return None;
    }
    let patterns = runtime_extract_strip_patterns(source, pattern_width);
    if patterns.is_empty() {
        return None;
    }
    let target_columns = width.max(pattern_width as u32) as usize;
    let cells = target_columns
        .saturating_sub(pattern_width)
        .saturating_add(1);
    let mut chosen = Vec::with_capacity(cells);

    let start =
        runtime_hash_material_seed(material, width, source.height()) as usize % patterns.len();
    chosen.push(start);
    for i in 1..cells {
        let prev = chosen[i - 1];
        let compatible = runtime_compatible_strip_patterns(&patterns, prev);
        let pick_from: Vec<usize> = if compatible.is_empty() {
            (0..patterns.len()).collect()
        } else {
            compatible
        };
        let choice = pick_from
            [runtime_hash_material_seed(material, i as u32, width) as usize % pick_from.len()];
        chosen.push(choice);
    }

    Some(runtime_reconstruct_strip_image(
        &patterns,
        &chosen,
        target_columns,
    ))
}

fn runtime_extract_strip_patterns(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    pattern_width: usize,
) -> Vec<RuntimeStripPattern> {
    use std::collections::HashSet;
    let sw = source.width().max(1) as usize;
    let sh = source.height().max(1) as usize;
    let mut seen: HashSet<Vec<[u8; 4]>> = HashSet::new();
    let mut patterns = Vec::new();
    for sx in 0..sw {
        let mut pixels = Vec::with_capacity(pattern_width * sh);
        for y in 0..sh {
            for x in 0..pattern_width {
                pixels.push(source.get_pixel(((sx + x) % sw) as u32, y as u32).0);
            }
        }
        if seen.insert(pixels.clone()) {
            patterns.push(RuntimeStripPattern {
                pixels,
                width: pattern_width,
                height: sh,
            });
        }
    }
    patterns
}

fn runtime_compatible_strip_patterns(
    patterns: &[RuntimeStripPattern],
    left_idx: usize,
) -> Vec<usize> {
    let mut out = Vec::new();
    for (right_idx, right) in patterns.iter().enumerate() {
        if runtime_strip_patterns_compatible(&patterns[left_idx], right) {
            out.push(right_idx);
        }
    }
    out
}

fn runtime_strip_patterns_compatible(
    left: &RuntimeStripPattern,
    right: &RuntimeStripPattern,
) -> bool {
    for y in 0..left.height {
        for x in 1..left.width {
            let li = y * left.width + x;
            let ri = y * right.width + (x - 1);
            if left.pixels[li] != right.pixels[ri] {
                return false;
            }
        }
    }
    true
}

fn runtime_reconstruct_strip_image(
    patterns: &[RuntimeStripPattern],
    chosen: &[usize],
    target_columns: usize,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let height = patterns[0].height;
    let mut image =
        ImageBuffer::from_pixel(target_columns as u32, height as u32, rgba([0, 0, 0, 0]));
    for y in 0..height {
        for (i, pattern_idx) in chosen.iter().enumerate() {
            let pattern = &patterns[*pattern_idx];
            let x = i;
            let p = pattern.pixels[y * pattern.width];
            image.put_pixel(x as u32, y as u32, rgba(p));
        }
        let last = &patterns[*chosen.last().unwrap_or(&0)];
        for extra in 1..last.width {
            let x = chosen.len().saturating_sub(1) + extra;
            if x >= target_columns {
                break;
            }
            let p = last.pixels[y * last.width + extra];
            image.put_pixel(x as u32, y as u32, rgba(p));
        }
    }
    image
}

fn runtime_quilt_surface_strip_horizontally(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let height = source.height().max(1);
    let patch = 8u32.min(source.width().max(1));
    let overlap = 3u32.min(patch.saturating_sub(1));
    let step = patch.saturating_sub(overlap).max(1);
    let mut field = ImageBuffer::from_pixel(width.max(1), height, rgba([0, 0, 0, 0]));
    let mut filled = vec![false; (field.width() * field.height()) as usize];

    let max_x = if width > patch { width - patch } else { 0 };
    let mut bx = 0;
    while bx <= max_x {
        let sx = runtime_choose_strip_patch_origin(
            material, source, &field, &filled, bx, patch, overlap,
        );
        runtime_blit_strip_patch(source, &mut field, &mut filled, sx, bx, patch);
        if bx == max_x {
            break;
        }
        bx = (bx + step).min(max_x);
    }

    field
}

fn runtime_tile_surface_strip_horizontally(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut field =
        ImageBuffer::from_pixel(width.max(1), source.height().max(1), rgba([0, 0, 0, 0]));
    for y in 0..field.height() {
        for x in 0..field.width() {
            let p = *source.get_pixel(x % source.width().max(1), y % source.height().max(1));
            field.put_pixel(x, y, p);
        }
    }
    field
}

fn runtime_quilt_material_field(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    runtime_quilt_image_to_size(material, source, 256, 256)
}

fn runtime_quilt_image_to_size(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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
            let (sx, sy) = runtime_choose_patch_origin(
                material, source, &field, &filled, bx, by, patch, overlap,
            );
            runtime_blit_patch(source, &mut field, &mut filled, sx, sy, bx, by, patch);
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
struct RuntimeWfcPattern {
    pixels: Vec<[u8; 4]>,
    band: usize,
}

fn runtime_wfc_material_field(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    runtime_wfc_source_field(material, source, 64, 64, 1)
}

fn runtime_wfc_source_field(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
    height: u32,
    band_count: usize,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let n = 3usize
        .min(source.width().max(1) as usize)
        .min(source.height().max(1) as usize);
    if n == 0 {
        return None;
    }
    let band_height = (source.height().max(1) as usize / band_count.max(1)).max(1);
    let patterns = runtime_extract_wfc_patterns(source, n, band_height, band_count);
    if patterns.is_empty() {
        return None;
    }
    let compat = runtime_build_wfc_compatibility(&patterns, n);
    for salt in 0..2u32 {
        if let Some(field) = runtime_solve_wfc_field(
            material,
            &patterns,
            &compat,
            n,
            width.max(n as u32) as usize,
            height.max(n as u32) as usize,
            band_count,
            salt,
        ) {
            return Some(field);
        }
    }
    None
}

fn runtime_extract_wfc_patterns(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    n: usize,
    band_height: usize,
    band_count: usize,
) -> Vec<RuntimeWfcPattern> {
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
                patterns.push(RuntimeWfcPattern { pixels, band });
            }
        }
    }
    patterns
}

fn runtime_build_wfc_compatibility(
    patterns: &[RuntimeWfcPattern],
    n: usize,
) -> [Vec<Vec<usize>>; 4] {
    let mut right = vec![Vec::new(); patterns.len()];
    let mut left = vec![Vec::new(); patterns.len()];
    let mut down = vec![Vec::new(); patterns.len()];
    let mut up = vec![Vec::new(); patterns.len()];
    for (i, a) in patterns.iter().enumerate() {
        for (j, b) in patterns.iter().enumerate() {
            if runtime_patterns_compatible_right(a, b, n) {
                right[i].push(j);
                left[j].push(i);
            }
            if runtime_patterns_compatible_down(a, b, n) {
                down[i].push(j);
                up[j].push(i);
            }
        }
    }
    [right, left, down, up]
}

fn runtime_patterns_compatible_right(
    a: &RuntimeWfcPattern,
    b: &RuntimeWfcPattern,
    n: usize,
) -> bool {
    for y in 0..n {
        for x in 1..n {
            if a.pixels[y * n + x] != b.pixels[y * n + (x - 1)] {
                return false;
            }
        }
    }
    true
}

fn runtime_patterns_compatible_down(
    a: &RuntimeWfcPattern,
    b: &RuntimeWfcPattern,
    n: usize,
) -> bool {
    for y in 1..n {
        for x in 0..n {
            if a.pixels[y * n + x] != b.pixels[(y - 1) * n + x] {
                return false;
            }
        }
    }
    true
}

fn runtime_solve_wfc_field(
    material: &str,
    patterns: &[RuntimeWfcPattern],
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
        let allowed: Vec<usize> = (0..pcount)
            .filter(|&p| wave[cell_idx * pcount + p])
            .collect();
        if allowed.is_empty() {
            return None;
        }
        let choice = allowed
            [runtime_hash_material_seed(material, cell_idx as u32, salt) as usize % allowed.len()];
        for p in 0..pcount {
            wave[cell_idx * pcount + p] = p == choice;
        }
        counts[cell_idx] = 1;
        if !runtime_propagate_wfc(
            &mut wave,
            &mut counts,
            compat,
            width,
            height,
            pcount,
            cell_idx,
        ) {
            return None;
        }
    }

    runtime_reconstruct_wfc_image(patterns, &wave, width, height, n, pcount)
}

fn runtime_propagate_wfc(
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
            if x + 1 < width {
                Some((idx + 1, 0usize))
            } else {
                None
            },
            if x > 0 { Some((idx - 1, 1usize)) } else { None },
            if y + 1 < height {
                Some((idx + width, 2usize))
            } else {
                None
            },
            if y > 0 {
                Some((idx - width, 3usize))
            } else {
                None
            },
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

fn runtime_reconstruct_wfc_image(
    patterns: &[RuntimeWfcPattern],
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

fn runtime_choose_patch_origin(
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
            let score =
                runtime_patch_overlap_score(source, field, filled, sx, sy, bx, by, patch, overlap);
            if score < best_score {
                best_score = score;
                best.clear();
                best.push((sx, sy));
            } else if score == best_score {
                best.push((sx, sy));
            }
        }
    }
    let choice = runtime_hash_material_seed(material, bx, by) as usize % best.len().max(1);
    best.get(choice).copied().unwrap_or((0, 0))
}

fn runtime_patch_overlap_score(
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
            let src = *source.get_pixel(
                sx.wrapping_add(px) % source.width().max(1),
                sy.wrapping_add(py) % source.height().max(1),
            );
            let dst = *field.get_pixel(fx, fy);
            score += runtime_pixel_distance(src, dst);
        }
    }
    score
}

fn runtime_blit_patch(
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
            let src = *source.get_pixel(
                sx.wrapping_add(px) % source.width().max(1),
                sy.wrapping_add(py) % source.height().max(1),
            );
            field.put_pixel(fx, fy, src);
            let idx = (fy * field.width() + fx) as usize;
            if let Some(slot) = filled.get_mut(idx) {
                *slot = true;
            }
        }
    }
}

fn runtime_choose_strip_patch_origin(
    material: &str,
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    field: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filled: &[bool],
    bx: u32,
    patch: u32,
    overlap: u32,
) -> u32 {
    let mut best = Vec::new();
    let mut best_score = u64::MAX;
    for sx in 0..source.width().max(1) {
        let score = runtime_strip_overlap_score(source, field, filled, sx, bx, patch, overlap);
        if score < best_score {
            best_score = score;
            best.clear();
            best.push(sx);
        } else if score == best_score {
            best.push(sx);
        }
    }
    let choice = runtime_hash_material_seed(material, bx, 0) as usize % best.len().max(1);
    best.get(choice).copied().unwrap_or(0)
}

fn runtime_strip_overlap_score(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    field: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    filled: &[bool],
    sx: u32,
    bx: u32,
    patch: u32,
    overlap: u32,
) -> u64 {
    let mut score = 0u64;
    if bx == 0 {
        return score;
    }
    for px in 0..patch.min(overlap) {
        let fx = bx + px;
        for y in 0..field.height() {
            let idx = (y * field.width() + fx) as usize;
            if !filled.get(idx).copied().unwrap_or(false) {
                continue;
            }
            let src = *source.get_pixel(
                (sx + px) % source.width().max(1),
                y % source.height().max(1),
            );
            let dst = *field.get_pixel(fx, y);
            score += runtime_pixel_distance(src, dst);
        }
    }
    score
}

fn runtime_blit_strip_patch(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    field: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    filled: &mut [bool],
    sx: u32,
    bx: u32,
    patch: u32,
) {
    for px in 0..patch {
        let fx = bx + px;
        if fx >= field.width() {
            break;
        }
        for y in 0..field.height() {
            let p = *source.get_pixel(
                (sx + px) % source.width().max(1),
                y % source.height().max(1),
            );
            field.put_pixel(fx, y, p);
            let idx = (y * field.width() + fx) as usize;
            if let Some(slot) = filled.get_mut(idx) {
                *slot = true;
            }
        }
    }
}

fn runtime_solve_state_constrained_surface_strip_2d(
    material: &str,
    region: &rpu_core::TerrainRegion,
    classified: &rpu_core::ClassifiedAsciiMap,
    tile: u32,
    flat: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    join: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ramp: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let width = flat.width().max(join.width()).max(ramp.width()).max(1) as usize;
    let height = flat.height().max(join.height()).max(ramp.height()).max(1) as usize;
    let n = 4usize.min(width).min(height);
    if n < 2 {
        return None;
    }

    let states = runtime_build_region_surface_states(region, classified, tile, width as u32);
    let patterns = runtime_extract_state_wfc_patterns(flat, join, ramp, n);
    if patterns.is_empty() {
        return None;
    }
    let compat = runtime_build_state_wfc_compatibility(&patterns, n);
    runtime_solve_state_wfc_field(material, &states, &patterns, &compat, width, height, n)
}

fn runtime_build_region_surface_states(
    region: &rpu_core::TerrainRegion,
    classified: &rpu_core::ClassifiedAsciiMap,
    tile: u32,
    width: u32,
) -> Vec<RuntimeStripState> {
    let mut states = vec![RuntimeStripState::Flat; width.max(1) as usize];
    for cell in classified
        .cells
        .iter()
        .filter(|cell| cell.region_id == region.id)
    {
        let state = match (cell.contour, cell.transition_role) {
            (rpu_core::TerrainContour::RampUpLeft, _)
            | (rpu_core::TerrainContour::RampUpRight, _) => RuntimeStripState::Ramp,
            (_, rpu_core::TerrainTransitionRole::JoinFromLeft)
            | (_, rpu_core::TerrainTransitionRole::JoinFromRight)
            | (_, rpu_core::TerrainTransitionRole::JoinBoth) => RuntimeStripState::Join,
            _ => RuntimeStripState::Flat,
        };
        let start = cell
            .surface_u
            .saturating_mul(tile)
            .min(width.saturating_sub(1));
        let end = (start + tile).min(width);
        for u in start..end {
            let idx = u as usize;
            states[idx] = match (states[idx], state) {
                (RuntimeStripState::Ramp, _) | (_, RuntimeStripState::Ramp) => {
                    RuntimeStripState::Ramp
                }
                (RuntimeStripState::Join, _) | (_, RuntimeStripState::Join) => {
                    RuntimeStripState::Join
                }
                _ => RuntimeStripState::Flat,
            };
        }
    }
    states
}

fn runtime_required_strip_state(
    states: &[RuntimeStripState],
    start: usize,
    width: usize,
) -> RuntimeStripState {
    let center = start + width / 2;
    states
        .get(center.min(states.len().saturating_sub(1)))
        .copied()
        .unwrap_or(RuntimeStripState::Flat)
}

fn runtime_extract_state_wfc_patterns(
    flat: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    join: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ramp: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    n: usize,
) -> Vec<RuntimeStateWfcPattern> {
    let mut patterns = Vec::new();
    patterns.extend(runtime_extract_family_wfc_patterns(
        flat,
        n,
        RuntimeStripState::Flat,
    ));
    patterns.extend(runtime_extract_family_wfc_patterns(
        join,
        n,
        RuntimeStripState::Join,
    ));
    patterns.extend(runtime_extract_family_wfc_patterns(
        ramp,
        n,
        RuntimeStripState::Ramp,
    ));
    patterns
}

fn runtime_extract_family_wfc_patterns(
    source: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    n: usize,
    state: RuntimeStripState,
) -> Vec<RuntimeStateWfcPattern> {
    use std::collections::HashSet;
    let sw = source.width().max(1) as usize;
    let sh = source.height().max(1) as usize;
    let mut seen: HashSet<Vec<[u8; 4]>> = HashSet::new();
    let mut patterns = Vec::new();
    for sy in 0..sh {
        for sx in 0..sw {
            let mut variants = Vec::new();

            let mut forward = Vec::with_capacity(n * n);
            for py in 0..n {
                for px in 0..n {
                    forward.push(
                        source
                            .get_pixel(((sx + px) % sw) as u32, ((sy + py) % sh) as u32)
                            .0,
                    );
                }
            }
            variants.push(forward);

            let mut mirrored = Vec::with_capacity(n * n);
            for py in 0..n {
                for px in 0..n {
                    mirrored.push(
                        source
                            .get_pixel(((sx + (n - 1 - px)) % sw) as u32, ((sy + py) % sh) as u32)
                            .0,
                    );
                }
            }
            variants.push(mirrored);

            for pixels in variants {
                if seen.insert(pixels.clone()) {
                    patterns.push(RuntimeStateWfcPattern { pixels, state });
                }
            }
        }
    }
    patterns
}

fn runtime_build_state_wfc_compatibility(
    patterns: &[RuntimeStateWfcPattern],
    n: usize,
) -> [Vec<Vec<usize>>; 4] {
    let mut right = vec![Vec::new(); patterns.len()];
    let mut left = vec![Vec::new(); patterns.len()];
    let mut down = vec![Vec::new(); patterns.len()];
    let mut up = vec![Vec::new(); patterns.len()];
    for (i, a) in patterns.iter().enumerate() {
        for (j, b) in patterns.iter().enumerate() {
            if runtime_state_wfc_patterns_compatible_right(a, b, n) {
                right[i].push(j);
                left[j].push(i);
            }
            if runtime_state_wfc_patterns_compatible_down(a, b, n) {
                down[i].push(j);
                up[j].push(i);
            }
        }
    }
    [right, left, down, up]
}

fn runtime_state_wfc_patterns_compatible_right(
    a: &RuntimeStateWfcPattern,
    b: &RuntimeStateWfcPattern,
    n: usize,
) -> bool {
    for y in 0..n {
        for x in 1..n {
            if a.pixels[y * n + x] != b.pixels[y * n + (x - 1)] {
                return false;
            }
        }
    }
    true
}

fn runtime_state_wfc_patterns_compatible_down(
    a: &RuntimeStateWfcPattern,
    b: &RuntimeStateWfcPattern,
    n: usize,
) -> bool {
    for y in 1..n {
        for x in 0..n {
            if a.pixels[y * n + x] != b.pixels[(y - 1) * n + x] {
                return false;
            }
        }
    }
    true
}

fn runtime_solve_state_wfc_field(
    material: &str,
    states: &[RuntimeStripState],
    patterns: &[RuntimeStateWfcPattern],
    compat: &[Vec<Vec<usize>>; 4],
    width: usize,
    height: usize,
    n: usize,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let cells_w = width.saturating_sub(n).saturating_add(1);
    let cells_h = height.saturating_sub(n).saturating_add(1);
    let cells = cells_w * cells_h;
    let pcount = patterns.len();
    let mut wave = vec![true; cells * pcount];
    let mut counts = vec![0usize; cells];

    for y in 0..cells_h {
        for x in 0..cells_w {
            let idx = y * cells_w + x;
            let required = runtime_required_strip_state(states, x, n);
            let mut count = 0usize;
            for p in 0..pcount {
                let allowed = patterns[p].state == required;
                wave[idx * pcount + p] = allowed;
                if allowed {
                    count += 1;
                }
            }
            if count == 0 {
                for p in 0..pcount {
                    wave[idx * pcount + p] = true;
                }
                count = pcount;
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
        let allowed: Vec<usize> = (0..pcount)
            .filter(|&p| wave[cell_idx * pcount + p])
            .collect();
        if allowed.is_empty() {
            return None;
        }
        let choice = allowed[runtime_hash_material_seed(material, cell_idx as u32, width as u32)
            as usize
            % allowed.len()];
        for p in 0..pcount {
            wave[cell_idx * pcount + p] = p == choice;
        }
        counts[cell_idx] = 1;
        if !runtime_propagate_state_wfc(
            &mut wave,
            &mut counts,
            compat,
            cells_w,
            cells_h,
            pcount,
            cell_idx,
        ) {
            return None;
        }
    }

    runtime_reconstruct_state_wfc_image(patterns, &wave, cells_w, cells_h, n, pcount)
}

fn runtime_propagate_state_wfc(
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
            if x + 1 < width {
                Some((idx + 1, 0usize))
            } else {
                None
            },
            if x > 0 { Some((idx - 1, 1usize)) } else { None },
            if y + 1 < height {
                Some((idx + width, 2usize))
            } else {
                None
            },
            if y > 0 {
                Some((idx - width, 3usize))
            } else {
                None
            },
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

fn runtime_reconstruct_state_wfc_image(
    patterns: &[RuntimeStateWfcPattern],
    wave: &[bool],
    cells_w: usize,
    cells_h: usize,
    n: usize,
    pcount: usize,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut chosen = vec![0usize; cells_w * cells_h];
    for idx in 0..chosen.len() {
        let Some(pattern_idx) = (0..pcount).find(|&p| wave[idx * pcount + p]) else {
            return None;
        };
        chosen[idx] = pattern_idx;
    }

    let out_w = cells_w + n.saturating_sub(1);
    let out_h = cells_h + n.saturating_sub(1);
    let mut sums = vec![[0u32; 4]; out_w * out_h];
    let mut counts = vec![0u32; out_w * out_h];

    for y in 0..cells_h {
        for x in 0..cells_w {
            let pattern = &patterns[chosen[y * cells_w + x]];
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

fn runtime_pixel_distance(a: Rgba<u8>, b: Rgba<u8>) -> u64 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    let da = a[3] as i32 - b[3] as i32;
    (dr * dr + dg * dg + db * db + da * da) as u64
}

fn runtime_hash_material_seed(material: &str, x: u32, y: u32) -> u32 {
    let mut hash = 2166136261u32;
    for byte in material.bytes() {
        hash = hash.wrapping_mul(16777619) ^ byte as u32;
    }
    hash ^ x.wrapping_mul(0x9e3779b1) ^ y.wrapping_mul(0x85ebca6b)
}

fn runtime_lerp_rgba(a: Rgba<u8>, b: Rgba<u8>, t: u8) -> Rgba<u8> {
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

fn terrain_shape_debug_color(shape: rpu_core::TerrainShape) -> [f32; 4] {
    match shape {
        rpu_core::TerrainShape::Empty => [0.0, 0.0, 0.0, 0.0],
        rpu_core::TerrainShape::Isolated => [242.0 / 255.0, 82.0 / 255.0, 82.0 / 255.0, 1.0],
        rpu_core::TerrainShape::Interior => [36.0 / 255.0, 66.0 / 255.0, 189.0 / 255.0, 1.0],
        rpu_core::TerrainShape::Top => [89.0 / 255.0, 224.0 / 255.0, 107.0 / 255.0, 1.0],
        rpu_core::TerrainShape::Bottom => [184.0 / 255.0, 84.0 / 255.0, 214.0 / 255.0, 1.0],
        rpu_core::TerrainShape::Left => [250.0 / 255.0, 184.0 / 255.0, 66.0 / 255.0, 1.0],
        rpu_core::TerrainShape::Right => [250.0 / 255.0, 143.0 / 255.0, 46.0 / 255.0, 1.0],
        rpu_core::TerrainShape::TopLeftOuter => [66.0 / 255.0, 235.0 / 255.0, 235.0 / 255.0, 1.0],
        rpu_core::TerrainShape::TopRightOuter => [48.0 / 255.0, 209.0 / 255.0, 250.0 / 255.0, 1.0],
        rpu_core::TerrainShape::BottomLeftOuter => {
            [224.0 / 255.0, 105.0 / 255.0, 207.0 / 255.0, 1.0]
        }
        rpu_core::TerrainShape::BottomRightOuter => {
            [191.0 / 255.0, 84.0 / 255.0, 242.0 / 255.0, 1.0]
        }
        rpu_core::TerrainShape::TopLeftInner => [158.0 / 255.0, 240.0 / 255.0, 158.0 / 255.0, 1.0],
        rpu_core::TerrainShape::TopRightInner => [140.0 / 255.0, 224.0 / 255.0, 140.0 / 255.0, 1.0],
        rpu_core::TerrainShape::BottomLeftInner => {
            [237.0 / 255.0, 148.0 / 255.0, 148.0 / 255.0, 1.0]
        }
        rpu_core::TerrainShape::BottomRightInner => {
            [219.0 / 255.0, 125.0 / 255.0, 125.0 / 255.0, 1.0]
        }
    }
}

fn terrain_style_marker_commands(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    z: i32,
    style: rpu_core::TerrainEdgeStyle,
) -> Vec<DrawCommand> {
    let marker = [0.08, 0.08, 0.1, 0.95];
    let center_w = (width * 0.24).max(2.0);
    let center_h = (height * 0.24).max(2.0);
    let center_x = x + (width - center_w) * 0.5;
    let center_y = y + (height - center_h) * 0.5;
    let mut rects = Vec::new();

    match style {
        rpu_core::TerrainEdgeStyle::Square => {
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: center_x,
                y: center_y,
                width: center_w,
                height: center_h,
                color: marker,
                visible: true,
            }));
        }
        rpu_core::TerrainEdgeStyle::Round => {
            let arm_w = (width * 0.14).max(1.5);
            let arm_h = (height * 0.14).max(1.5);
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: center_x,
                y: center_y,
                width: center_w,
                height: center_h,
                color: marker,
                visible: true,
            }));
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: center_x - arm_w * 0.9,
                y: center_y + (center_h - arm_h) * 0.5,
                width: arm_w,
                height: arm_h,
                color: marker,
                visible: true,
            }));
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: center_x + center_w - arm_w * 0.1,
                y: center_y + (center_h - arm_h) * 0.5,
                width: arm_w,
                height: arm_h,
                color: marker,
                visible: true,
            }));
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: center_x + (center_w - arm_w) * 0.5,
                y: center_y - arm_h * 0.9,
                width: arm_w,
                height: arm_h,
                color: marker,
                visible: true,
            }));
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: center_x + (center_w - arm_w) * 0.5,
                y: center_y + center_h - arm_h * 0.1,
                width: arm_w,
                height: arm_h,
                color: marker,
                visible: true,
            }));
        }
        rpu_core::TerrainEdgeStyle::Diagonal => {
            let step_w = (width * 0.16).max(1.5);
            let step_h = (height * 0.16).max(1.5);
            let left = x + width * 0.26;
            let top = y + height * 0.62;
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: left,
                y: top,
                width: step_w,
                height: step_h,
                color: marker,
                visible: true,
            }));
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: left + step_w * 0.95,
                y: top - step_h * 0.95,
                width: step_w,
                height: step_h,
                color: marker,
                visible: true,
            }));
            rects.push(DrawCommand::Rect(SceneRect {
                anchor: Anchor::World,
                layer: -9,
                z,
                x: left + step_w * 1.9,
                y: top - step_h * 1.9,
                width: step_w,
                height: step_h,
                color: marker,
                visible: true,
            }));
        }
    }

    rects
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
                let Some(meaning) = legend.get(&ch) else {
                    continue;
                };
                let pos = [
                    map.origin[0] + col as f32 * map.cell[0],
                    map.origin[1] + row as f32 * map.cell[1],
                ];
                match meaning {
                    MapLegendMeaning::Marker => {
                        markers.entry(ch.to_string()).or_insert(pos);
                    }
                    MapLegendMeaning::Spawn(name) => {
                        markers.entry(ch.to_string()).or_insert(pos);
                        markers.entry(name.clone()).or_insert(pos);
                    }
                    _ => {}
                }
            }
        }
    }
    markers
}

fn compile_map_colliders(maps: &[AsciiMapNode]) -> Vec<RuntimeCollider> {
    let mut colliders = Vec::new();
    for map in maps {
        let legend: HashMap<char, &MapLegendMeaning> = map
            .legend
            .iter()
            .map(|entry| (entry.symbol, &entry.meaning))
            .collect();
        for (row, line) in map.rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if !map_symbol_is_solid(ch, &legend) {
                    continue;
                }
                colliders.push(RuntimeCollider {
                    x: map.origin[0] + col as f32 * map.cell[0],
                    y: map.origin[1] + row as f32 * map.cell[1],
                    width: map.cell[0],
                    height: map.cell[1],
                });
            }
        }
    }
    colliders
}

fn map_symbol_is_solid(ch: char, legend: &HashMap<char, &MapLegendMeaning>) -> bool {
    matches!(
        legend.get(&ch),
        Some(MapLegendMeaning::Texture(_))
            | Some(MapLegendMeaning::Color(_))
            | Some(MapLegendMeaning::Terrain(_))
    )
}

fn submit_draw_command(
    frame: &mut SceneFrame,
    camera: &SceneCamera,
    view: &RenderView,
    stable_order: i32,
    command: &DrawCommand,
    asset_base: &str,
    high_scores: &HighScoreTable,
    elapsed_time: f32,
) {
    match command {
        DrawCommand::Rect(rect) => {
            let (x, y, width, height) = screen_rect_for_anchor(
                rect.anchor,
                camera,
                view,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            );
            frame.push_rect(
                rect.layer,
                rect.z * 1000 + stable_order,
                x,
                y,
                width,
                height,
                rect.color,
            );
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
            let texture_path = texture_name.as_deref().map(|texture| {
                if texture.starts_with("generated://") {
                    texture.to_string()
                } else {
                    format!("{}/{}", asset_base.trim_end_matches('/'), texture)
                }
            });
            if sprite.anchor != Anchor::World && !sprite.repeat_x && !sprite.repeat_y {
                let (sx, sy, sw, sh) = screen_rect_for_anchor(
                    sprite.anchor,
                    camera,
                    view,
                    sprite.x,
                    sprite.y,
                    sprite.width,
                    sprite.height,
                );
                frame.push_sprite(
                    sprite.layer,
                    sprite.z * 1000 + stable_order,
                    sx,
                    sy,
                    sw,
                    sh,
                    sprite.rotation,
                    sprite.color,
                    sprite.flip_x,
                    sprite.flip_y,
                    texture_path.as_deref(),
                );
            } else {
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
                    sprite.rotation,
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
        }
        DrawCommand::Text(text) => {
            let font_path = format!("{}/{}", asset_base.trim_end_matches('/'), text.font);
            let (x, y) = screen_point_for_anchor(text.anchor, camera, view, text.x, text.y);
            frame.push_text(
                text.layer,
                text.z * 1000 + stable_order,
                x,
                y,
                &text.value,
                &font_path,
                text.font_size * view.scale.max(0.01),
                text.color,
                Anchor::World,
                text.align,
            );
        }
        DrawCommand::HighScore(table) => {
            let font_path = format!("{}/{}", asset_base.trim_end_matches('/'), table.font);
            let (base_x, base_y) =
                screen_point_for_anchor(table.anchor, camera, view, table.x, table.y);
            let score_x = base_x + table.width * view.scale;
            let gap = table.gap * view.scale;
            let font_size = table.font_size * view.scale.max(0.01);
            for (index, entry) in high_scores.entries.iter().take(table.items).enumerate() {
                let row_y = base_y + gap * index as f32;
                frame.push_text(
                    table.layer,
                    table.z * 1000 + stable_order + index as i32 * 2,
                    base_x,
                    row_y,
                    &entry.name,
                    &font_path,
                    font_size,
                    table.color,
                    Anchor::World,
                    TextAlign::Left,
                );
                let score = format!("{:0width$}", entry.score.max(0), width = table.score_digits);
                frame.push_text(
                    table.layer,
                    table.z * 1000 + stable_order + index as i32 * 2 + 1,
                    score_x,
                    row_y,
                    &score,
                    &font_path,
                    font_size,
                    table.color,
                    Anchor::World,
                    TextAlign::Right,
                );
            }
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
                diagnostic.severity, diagnostic.message, line
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

fn anchored_virtual_rect(
    anchor: Anchor,
    view: &RenderView,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> (f32, f32, f32, f32) {
    let virtual_w = view.virtual_size.0;
    let virtual_h = view.virtual_size.1;
    let anchored_x = match anchor {
        Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => x,
        Anchor::Top | Anchor::Center | Anchor::Bottom => virtual_w * 0.5 - width * 0.5 + x,
        Anchor::TopRight | Anchor::Right | Anchor::BottomRight => virtual_w - width + x,
        Anchor::World => x,
    };
    let anchored_y = match anchor {
        Anchor::TopLeft | Anchor::Top | Anchor::TopRight => y,
        Anchor::Left | Anchor::Center | Anchor::Right => virtual_h * 0.5 - height * 0.5 + y,
        Anchor::BottomLeft | Anchor::Bottom | Anchor::BottomRight => virtual_h - height + y,
        Anchor::World => y,
    };
    (anchored_x, anchored_y, width, height)
}

fn screen_rect_for_anchor(
    anchor: Anchor,
    camera: &SceneCamera,
    view: &RenderView,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> (f32, f32, f32, f32) {
    if anchor == Anchor::World {
        world_to_screen(camera, view, x, y, width, height)
    } else {
        let (vx, vy, vw, vh) = anchored_virtual_rect(anchor, view, x, y, width, height);
        view.map_rect(vx, vy, vw, vh)
    }
}

fn screen_point_for_anchor(
    anchor: Anchor,
    camera: &SceneCamera,
    view: &RenderView,
    x: f32,
    y: f32,
) -> (f32, f32) {
    let (sx, sy, _, _) = screen_rect_for_anchor(anchor, camera, view, x, y, 0.0, 0.0);
    (sx, sy)
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
    rotation: f32,
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
        let (sx, sy, sw, sh) = screen_rect_for_anchor(
            Anchor::World,
            camera,
            view,
            scrolled_x,
            scrolled_y,
            width,
            height,
        );
        frame.push_sprite(
            layer,
            order,
            sx,
            sy,
            sw,
            sh,
            rotation,
            color,
            flip_x,
            flip_y,
            texture_path,
        );
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
            frame.push_sprite(
                layer,
                order,
                sx,
                sy,
                sw,
                sh,
                rotation,
                color,
                flip_x,
                flip_y,
                texture_path,
            );
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
