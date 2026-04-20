use anyhow::Result;
use rpu_core::{
    AsciiMapNode, BinaryOp, CompareOp, CompiledProject, Condition, DrawCommand, Expr,
    MapLegendMeaning, OpCode, RectNode, RpuProject, SceneCamera, SceneRect, ScriptProperty,
    ScriptTarget, SpriteNode,
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
        Some((1280, 720))
    }

    fn init(&mut self, _ctx: &mut RuntimeContext) {
        self.session.mark_initialized();
    }

    fn update(&mut self, _ctx: &mut RuntimeContext) {
        self.session.tick();
    }

    fn render(&mut self, _ctx: &mut RuntimeContext, frame: &mut SceneFrame) {
        frame.clear_color(self.current_clear_color());
        let camera = self.session.compiled.active_camera();
        let viewport = frame.size();

        for (index, command) in self.session.world.static_draw_commands.iter().enumerate() {
            submit_draw_command(frame, &camera, viewport, index as i32, command, &self.session.project);
        }

        let base_order = self.session.world.static_draw_commands.len() as i32;
        for (index, entity) in self.session.world.entities.iter().enumerate() {
            let stable_order = base_order + index as i32;
            let (x, y, width, height) = world_to_screen(
                &camera,
                viewport,
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
                RuntimeEntityKind::Sprite { texture } => {
                    let texture_path = texture
                        .as_deref()
                        .map(|texture| self.session.project.root().join("assets").join(texture))
                        .map(pathbuf_to_string);
                    frame.push_sprite(
                        entity.layer,
                        entity.z * 1000 + stable_order,
                        x,
                        y,
                        width,
                        height,
                        entity.color,
                        texture_path.as_deref(),
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
}

struct RuntimeWorld {
    static_draw_commands: Vec<DrawCommand>,
    entities: Vec<RuntimeEntity>,
}

struct RuntimeEntity {
    name: String,
    layer: i32,
    z: i32,
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    script: Option<String>,
    kind: RuntimeEntityKind,
}

enum RuntimeEntityKind {
    Rect,
    Sprite { texture: Option<String> },
}

#[derive(Clone, Copy)]
enum Value {
    Scalar(f32),
    Vec2([f32; 2]),
    Color([f32; 4]),
}

impl RuntimeSession {
    fn new(project: RpuProject) -> Result<Self> {
        let compiled = project.compile()?;
        log_compilation("initial compile", &compiled);
        let world = RuntimeWorld::from_compiled(&compiled);
        Ok(Self {
            project,
            compiled,
            world,
            initialized: false,
            ticks: 0,
            last_reload_poll: Instant::now(),
            last_tick_instant: Instant::now(),
        })
    }

    fn mark_initialized(&mut self) {
        self.initialized = true;
        self.execute_event("ready", 0.0);
        self.last_tick_instant = Instant::now();
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_tick_instant).as_secs_f32().min(0.1);
        self.last_tick_instant = now;

        if self.initialized {
            self.ticks = self.ticks.saturating_add(1);
            self.execute_event("update", dt);
        }
        self.maybe_reload();
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

        for (entity_index, entity_name, script_path, ops) in scheduled {
            let mut locals = HashMap::new();
            self.apply_ops(entity_index, &entity_name, &script_path, event, &ops, dt, &mut locals);
        }
    }

    fn apply_ops(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        ops: &[OpCode],
        dt: f32,
        locals: &mut HashMap<String, Value>,
    ) {
        for op in ops {
            self.apply_op(entity_index, entity_name, script_path, event, op, dt, locals);
        }
    }

    fn apply_op(
        &mut self,
        entity_index: usize,
        entity_name: &str,
        script_path: &Path,
        event: &str,
        op: &OpCode,
        dt: f32,
        locals: &mut HashMap<String, Value>,
    ) {
        match op {
            OpCode::Log(message) => {
                eprintln!("rpu: script: {entity_name}: {message}");
            }
            OpCode::IgnoreValue(_) => {}
            OpCode::Let(name, expr) => {
                let Some(value) = self.eval_expr(entity_index, expr, dt, locals) else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate local binding in {} for {}:{}",
                        script_path.display(),
                        entity_name,
                        event
                    );
                    return;
                };
                locals.insert(name.clone(), value);
            }
            OpCode::Assign(target, expr) => {
                let Some(value) = self.eval_expr(entity_index, expr, dt, locals) else {
                    eprintln!(
                        "rpu: script warning: failed to evaluate assignment in {} for {}:{}",
                        script_path.display(),
                        entity_name,
                        event
                    );
                    return;
                };
                if !self.assign_target(entity_index, target, value) {
                    eprintln!(
                        "rpu: script warning: invalid assignment target in {} for {}:{}",
                        script_path.display(),
                        entity_name,
                        event
                    );
                }
            }
            OpCode::If(condition, body, else_body) => {
                if self.eval_condition(entity_index, condition, dt, locals).unwrap_or(false) {
                    self.apply_ops(entity_index, entity_name, script_path, event, body, dt, locals);
                } else {
                    self.apply_ops(
                        entity_index,
                        entity_name,
                        script_path,
                        event,
                        else_body,
                        dt,
                        locals,
                    );
                }
            }
            OpCode::MoveBy(delta) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                entity.pos[0] += delta[0];
                entity.pos[1] += delta[1];
            }
            OpCode::MoveByDt(velocity) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                entity.pos[0] += velocity[0] * dt;
                entity.pos[1] += velocity[1] * dt;
            }
            OpCode::SetPos(pos) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                entity.pos = *pos;
            }
            OpCode::SetColor(color) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                entity.color = *color;
            }
            OpCode::CopyPos(target) => {
                let Some(pos) = self.world.find_entity(target).map(|entity| entity.pos) else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {} for {}:{}",
                        target,
                        script_path.display(),
                        entity_name,
                        event
                    );
                    return;
                };
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                entity.pos = pos;
            }
            OpCode::ClampX(range) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                let min = range[0].min(range[1]);
                let max = range[0].max(range[1]);
                entity.pos[0] = entity.pos[0].clamp(min, max);
            }
            OpCode::ClampY(range) => {
                let Some(entity) = self.world.entities.get_mut(entity_index) else {
                    return;
                };
                let min = range[0].min(range[1]);
                let max = range[0].max(range[1]);
                entity.pos[1] = entity.pos[1].clamp(min, max);
            }
            OpCode::MoveByTarget(target, delta) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.pos[0] += delta[0];
                    entity.pos[1] += delta[1];
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {} for {}:{}",
                        target,
                        script_path.display(),
                        entity_name,
                        event
                    );
                }
            }
            OpCode::MoveByDtTarget(target, velocity) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.pos[0] += velocity[0] * dt;
                    entity.pos[1] += velocity[1] * dt;
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {} for {}:{}",
                        target,
                        script_path.display(),
                        entity_name,
                        event
                    );
                }
            }
            OpCode::SetPosTarget(target, pos) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.pos = *pos;
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {} for {}:{}",
                        target,
                        script_path.display(),
                        entity_name,
                        event
                    );
                }
            }
            OpCode::SetColorTarget(target, color) => {
                if let Some(entity) = self.world.find_entity_mut(target) {
                    entity.color = *color;
                } else {
                    eprintln!(
                        "rpu: script warning: missing target `{}` in {} for {}:{}",
                        target,
                        script_path.display(),
                        entity_name,
                        event
                    );
                }
            }
            OpCode::Raw(raw) => {
                eprintln!(
                    "rpu: script warning: unsupported opcode `{}` in {} for {}:{}",
                    raw,
                    script_path.display(),
                    entity_name,
                    event
                );
            }
        }
    }

    fn eval_condition(
        &self,
        entity_index: usize,
        condition: &Condition,
        dt: f32,
        locals: &HashMap<String, Value>,
    ) -> Option<bool> {
        Some(match condition {
            Condition::Compare { left, op, right } => {
                let left = self.eval_expr(entity_index, left, dt, locals)?.as_scalar()?;
                let right = self.eval_expr(entity_index, right, dt, locals)?.as_scalar()?;
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
                self.eval_condition(entity_index, left, dt, locals)?
                    && self.eval_condition(entity_index, right, dt, locals)?
            }
            Condition::Or(left, right) => {
                self.eval_condition(entity_index, left, dt, locals)?
                    || self.eval_condition(entity_index, right, dt, locals)?
            }
            Condition::Not(inner) => !self.eval_condition(entity_index, inner, dt, locals)?,
        })
    }

    fn eval_expr(
        &self,
        entity_index: usize,
        expr: &Expr,
        dt: f32,
        locals: &HashMap<String, Value>,
    ) -> Option<Value> {
        match expr {
            Expr::Number(value) => Some(Value::Scalar(*value)),
            Expr::Dt => Some(Value::Scalar(dt)),
            Expr::Variable(name) => locals.get(name).copied(),
            Expr::Target(target) => self.read_target(entity_index, target),
            Expr::Color(color) => Some(Value::Color(*color)),
            Expr::Binary(left, op, right) => {
                let left = self.eval_expr(entity_index, left, dt, locals)?.as_scalar()?;
                let right = self.eval_expr(entity_index, right, dt, locals)?.as_scalar()?;
                let value = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    BinaryOp::Div => left / right,
                };
                Some(Value::Scalar(value))
            }
            Expr::Clamp(value, min, max) => {
                let value = self.eval_expr(entity_index, value, dt, locals)?.as_scalar()?;
                let min = self.eval_expr(entity_index, min, dt, locals)?.as_scalar()?;
                let max = self.eval_expr(entity_index, max, dt, locals)?.as_scalar()?;
                Some(Value::Scalar(value.clamp(min.min(max), min.max(max))))
            }
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
            (ScriptProperty::Pos, Value::Vec2(value)) => entity.pos = value,
            (ScriptProperty::Color, Value::Color(value)) => entity.color = value,
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
            ScriptProperty::Pos => Value::Vec2(entity.pos),
            ScriptProperty::Color => Value::Color(entity.color),
        })
    }
}

impl RuntimeWorld {
    fn from_compiled(compiled: &CompiledProject) -> Self {
        let mut static_draw_commands = Vec::new();
        let mut entities = Vec::new();

        for document in &compiled.parsed_scenes {
            for scene in &document.scenes {
                static_draw_commands.extend(compile_static_map_commands(&scene.maps));
                let markers = compile_map_markers(&scene.maps);
                entities.extend(scene.rects.iter().map(runtime_rect_entity));
                entities.extend(
                    scene
                        .sprites
                        .iter()
                        .map(|sprite| runtime_sprite_entity(sprite, &markers)),
                );
            }
        }

        Self {
            static_draw_commands,
            entities,
        }
    }

    fn find_entity_mut(&mut self, name: &str) -> Option<&mut RuntimeEntity> {
        self.entities.iter_mut().find(|entity| entity.name == name)
    }

    fn find_entity(&self, name: &str) -> Option<&RuntimeEntity> {
        self.entities.iter().find(|entity| entity.name == name)
    }
}

impl Value {
    fn as_scalar(self) -> Option<f32> {
        match self {
            Value::Scalar(value) => Some(value),
            _ => None,
        }
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
        layer: rect.visual.layer,
        z: rect.visual.z,
        pos: rect.visual.pos,
        size: rect.visual.size,
        color: rect.visual.color,
        script: rect.visual.script.clone(),
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
        layer: sprite.visual.layer,
        z: sprite.visual.z,
        pos,
        size: sprite.visual.size,
        color: sprite.visual.color,
        script: sprite.visual.script.clone(),
        kind: RuntimeEntityKind::Sprite {
            texture: sprite.texture.clone(),
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
    viewport: (u32, u32),
    stable_order: i32,
    command: &DrawCommand,
    project: &RpuProject,
) {
    match command {
        DrawCommand::Rect(rect) => {
            let (x, y, width, height) =
                world_to_screen(camera, viewport, rect.x, rect.y, rect.width, rect.height);
            frame.push_rect(rect.layer, rect.z * 1000 + stable_order, x, y, width, height, rect.color);
        }
        DrawCommand::Sprite(sprite) => {
            let (x, y, width, height) =
                world_to_screen(camera, viewport, sprite.x, sprite.y, sprite.width, sprite.height);
            let texture_path = sprite
                .texture
                .as_deref()
                .map(|texture| project.root().join("assets").join(texture))
                .map(pathbuf_to_string);
            frame.push_sprite(
                sprite.layer,
                sprite.z * 1000 + stable_order,
                x,
                y,
                width,
                height,
                sprite.color,
                texture_path.as_deref(),
            );
        }
    }
}

fn log_compilation(label: &str, compiled: &CompiledProject) {
    eprintln!(
        "rpu: {}: {} scene(s), {} script(s), {} camera(s), {} rect(s), {} sprite(s), {} handler(s), {} op(s), {} asset(s), {} warning(s), {} error(s)",
        label,
        compiled.scene_count(),
        compiled.scripts.len(),
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
    viewport: (u32, u32),
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> (f32, f32, f32, f32) {
    let zoom = camera.zoom.max(0.01);
    let screen_x = (x - camera.x) * zoom + viewport.0 as f32 * 0.5;
    let screen_y = (y - camera.y) * zoom + viewport.1 as f32 * 0.5;
    (screen_x, screen_y, width * zoom, height * zoom)
}

fn pathbuf_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}
