use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectInfo,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub debug: DebugConfig,
    #[serde(default, alias = "apple", skip_serializing_if = "MetaConfig::is_empty")]
    pub meta: MetaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_start_scene")]
    pub start_scene: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: u32,
    #[serde(default = "default_window_height")]
    pub height: u32,
    #[serde(default = "default_window_scale")]
    pub default_scale: f32,
    #[serde(default)]
    pub resize: ResizeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(
        default,
        alias = "apple_development_team",
        alias = "team_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub development_team: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebugConfig {
    #[serde(default)]
    pub physics: bool,
}

impl MetaConfig {
    fn is_empty(&self) -> bool {
        self.bundle_id.is_none() && self.display_name.is_none() && self.development_team.is_none()
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
            default_scale: default_window_scale(),
            resize: ResizeMode::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResizeMode {
    #[default]
    Letterbox,
    Stretch,
}

#[derive(Debug, Clone)]
pub struct RpuProject {
    root: PathBuf,
    manifest: ProjectManifest,
}

#[derive(Debug, Clone)]
pub struct ProjectPaths {
    pub manifest: PathBuf,
    pub scenes: PathBuf,
    pub scripts: PathBuf,
    pub assets: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub relative_path: PathBuf,
    pub contents: String,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct BundledAsset {
    pub relative_path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BundledProject {
    manifest: ProjectManifest,
    scenes: Vec<SourceFile>,
    scripts: Vec<SourceFile>,
    assets: Vec<BundledAsset>,
}

#[derive(Debug, Clone)]
pub struct CompiledProject {
    pub name: String,
    pub version: String,
    pub start_scene: String,
    pub window: WindowConfig,
    pub debug: DebugConfig,
    pub scenes: Vec<SourceFile>,
    pub parsed_scenes: Vec<SceneDocument>,
    pub scripts: Vec<SourceFile>,
    pub bytecode_scripts: Vec<BytecodeScript>,
    pub draw_commands: Vec<DrawCommand>,
    pub assets: Vec<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
    pub script_references: Vec<String>,
    pub texture_references: Vec<String>,
    pub fingerprint: ProjectFingerprint,
}

#[derive(Debug, Clone)]
pub struct ProjectFingerprint {
    pub latest_modified: Option<SystemTime>,
    pub source_file_count: usize,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub path: Option<PathBuf>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct BytecodeScript {
    pub path: PathBuf,
    pub state: Vec<BytecodeState>,
    pub handlers: Vec<BytecodeHandler>,
    pub functions: Vec<BytecodeFunction>,
}

#[derive(Debug, Clone)]
pub struct BytecodeState {
    pub name: String,
    pub init: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct BytecodeHandler {
    pub event: String,
    pub params: Vec<String>,
    pub ops: Vec<BytecodeOp>,
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub params: Vec<String>,
    pub ops: Vec<BytecodeOp>,
}

#[derive(Debug, Clone)]
pub struct BytecodeOp {
    pub line: usize,
    pub op: OpCode,
}

#[derive(Debug, Clone)]
pub enum ScriptProperty {
    X,
    Y,
    Width,
    Height,
    Pos,
    Size,
    Rotation,
    Color,
    Texture,
    Animation,
    FlipX,
    FlipY,
    Vx,
    Vy,
    MoveX,
    Jump,
    Grounded,
    Text,
    State(String),
}

#[derive(Debug, Clone)]
pub enum ScriptTarget {
    SelfEntity(ScriptProperty),
    NamedEntity(String, ScriptProperty),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f32),
    Dt,
    String(String),
    Variable(String),
    Call(String, Vec<Expr>),
    Target(ScriptTarget),
    Color([f32; 4]),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Clamp(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Compare {
        left: Expr,
        op: CompareOp,
        right: Expr,
    },
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
}

#[derive(Debug, Clone)]
pub enum OpCode {
    Log(String),
    IgnoreValue(String),
    Call(String, Vec<Expr>),
    Return(Expr),
    Let(String, Expr),
    StateSet(String, Expr),
    Assign(ScriptTarget, Expr),
    If(Condition, Vec<BytecodeOp>, Vec<BytecodeOp>),
    Spawn(String, Option<String>, Expr, Expr),
    Destroy(DestroyTarget),
    DestroyExpr(Expr),
    MoveBy([f32; 2]),
    MoveByDt([f32; 2]),
    SetPos([f32; 2]),
    SetColor([f32; 4]),
    CopyPos(String),
    ClampX([f32; 2]),
    ClampY([f32; 2]),
    MoveByTarget(String, [f32; 2]),
    MoveByDtTarget(String, [f32; 2]),
    SetPosTarget(String, [f32; 2]),
    SetColorTarget(String, [f32; 4]),
    Raw(String),
}

#[derive(Debug, Clone)]
pub enum DestroyTarget {
    SelfEntity,
    Named(String),
}

#[derive(Debug, Clone)]
pub struct SceneDocument {
    pub path: PathBuf,
    pub scenes: Vec<SceneNode>,
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub name: String,
    pub meta: SceneMeta,
    pub camera: Option<CameraNode>,
    pub maps: Vec<AsciiMapNode>,
    pub stacks: Vec<StackNode>,
    pub rects: Vec<RectNode>,
    pub sprites: Vec<SpriteNode>,
    pub texts: Vec<TextNode>,
    pub high_scores: Vec<HighScoreNode>,
}

#[derive(Debug, Clone, Default)]
pub struct SceneMeta {
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VisualNode {
    pub visible: bool,
    pub template: bool,
    pub group: Option<String>,
    pub parent: Option<String>,
    pub order: i32,
    pub anchor: Anchor,
    pub layer: i32,
    pub z: i32,
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub size_explicit: bool,
    pub color: [f32; 4],
    pub script: Option<String>,
    pub script_binding: Option<String>,
    pub inline_script: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RectNode {
    pub name: String,
    pub visual: VisualNode,
}

#[derive(Debug, Clone)]
pub struct CameraNode {
    pub name: String,
    pub pos: [f32; 2],
    pub zoom: f32,
    pub background: [f32; 4],
    pub follow: Option<String>,
    pub follow_offset: [f32; 2],
    pub bounds_min: Option<[f32; 2]>,
    pub bounds_max: Option<[f32; 2]>,
    pub follow_smoothing: f32,
    pub dead_zone: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct TextNode {
    pub name: String,
    pub visual: VisualNode,
    pub value: String,
    pub font: String,
    pub font_size: f32,
    pub align: TextAlign,
}

#[derive(Debug, Clone)]
pub struct HighScoreNode {
    pub name: String,
    pub visual: VisualNode,
    pub font: String,
    pub font_size: f32,
    pub items: usize,
    pub gap: f32,
    pub score_digits: usize,
}

#[derive(Debug, Clone)]
pub struct StackNode {
    pub name: String,
    pub anchor: Anchor,
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub direction: LayoutDirection,
    pub gap: f32,
    pub align: StackAlign,
}

#[derive(Debug, Clone)]
pub struct SpriteNode {
    pub name: String,
    pub visual: VisualNode,
    pub rotation: f32,
    pub textures: Vec<String>,
    pub animations: std::collections::HashMap<String, SpriteAnimation>,
    pub animation_fps: f32,
    pub animation_mode: AnimationMode,
    pub destroy_on_animation_end: bool,
    pub symbol: Option<String>,
    pub scroll: [f32; 2],
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub collider_offset: [f32; 2],
    pub collider_size: Option<[f32; 2]>,
    pub physics: PhysicsMode,
    pub physics_settings: PlatformerPhysicsSettings,
}

#[derive(Debug, Clone)]
pub struct SpriteAnimation {
    pub textures: Vec<String>,
    pub fps: f32,
    pub mode: AnimationMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationMode {
    Loop,
    Once,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsMode {
    None,
    Platformer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformerPhysicsSettings {
    pub acceleration: f32,
    pub friction: f32,
    pub max_speed: f32,
    pub gravity: f32,
    pub jump_speed: f32,
    pub max_fall_speed: f32,
    pub coyote_time: f32,
    pub jump_buffer: f32,
}

impl Default for PlatformerPhysicsSettings {
    fn default() -> Self {
        Self {
            acceleration: 520.0,
            friction: 840.0,
            max_speed: 96.0,
            gravity: 560.0,
            jump_speed: 255.0,
            max_fall_speed: 280.0,
            coyote_time: 0.08,
            jump_buffer: 0.10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    World,
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Default for Anchor {
    fn default() -> Self {
        Self::World
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone)]
pub struct AsciiMapNode {
    pub name: String,
    pub origin: [f32; 2],
    pub cell: [f32; 2],
    pub render: TerrainRenderMode,
    pub terrain_style: TerrainStyleSettings,
    pub legend: Vec<MapLegendEntry>,
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainRenderMode {
    Debug,
    Basic,
    Synth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainStyleSettings {
    pub cap_depth: f32,
    pub ramp_cap_depth: f32,
    pub join_cap_depth: f32,
    pub shoulder_width: f32,
    pub surface_roughness: f32,
    pub shoulder_shape: TerrainShoulderShape,
}

impl Default for TerrainStyleSettings {
    fn default() -> Self {
        Self {
            cap_depth: 0.5,
            ramp_cap_depth: 5.0 / 12.0,
            join_cap_depth: 0.5,
            shoulder_width: 1.0,
            surface_roughness: 0.0,
            shoulder_shape: TerrainShoulderShape::Linear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainShoulderShape {
    Linear,
    Bend,
}

#[derive(Debug, Clone)]
pub struct MapLegendEntry {
    pub symbol: char,
    pub meaning: MapLegendMeaning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTerrainEntry {
    pub topology: TerrainTopology,
    pub material: String,
    pub material_stack: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTopology {
    Solid,
    SlopeUp,
    SlopeDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainShape {
    Empty,
    Isolated,
    Interior,
    Top,
    Bottom,
    Left,
    Right,
    TopLeftOuter,
    TopRightOuter,
    BottomLeftOuter,
    BottomRightOuter,
    TopLeftInner,
    TopRightInner,
    BottomLeftInner,
    BottomRightInner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainContour {
    None,
    FlatTop,
    RampUpRight,
    RampUpLeft,
    CapLeft,
    CapRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTransitionRole {
    None,
    RampUpRight,
    RampUpLeft,
    JoinFromLeft,
    JoinFromRight,
    JoinBoth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedMapCell {
    pub row: usize,
    pub col: usize,
    pub symbol: char,
    pub topology: TerrainTopology,
    pub material_key: String,
    pub material: String,
    pub shape: TerrainShape,
    pub contour: TerrainContour,
    pub transition_role: TerrainTransitionRole,
    pub transition_strength: u8,
    pub style: TerrainEdgeStyle,
    pub normal: TerrainNormal,
    pub tangent: TerrainTangent,
    pub surface_u: u32,
    pub boundary_distance: u32,
    pub depth_band: TerrainDepthBand,
    pub region_id: usize,
    pub exposed: TerrainExposedSides,
    pub is_boundary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainRegion {
    pub id: usize,
    pub material: String,
    pub min_row: usize,
    pub min_col: usize,
    pub max_row: usize,
    pub max_col: usize,
    pub cells: Vec<(usize, usize)>,
    pub boundary_cells: Vec<(usize, usize)>,
    pub boundary_loop: Vec<(usize, usize)>,
    pub max_boundary_distance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedAsciiMap {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub render: TerrainRenderMode,
    pub cells: Vec<ClassifiedMapCell>,
    pub regions: Vec<TerrainRegion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TerrainExposedSides {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl TerrainExposedSides {
    pub fn any(self) -> bool {
        self.top || self.bottom || self.left || self.right
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainEdgeStyle {
    Square,
    Round,
    Diagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainNormal {
    None,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTangent {
    None,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainDepthBand {
    Edge,
    NearSurface,
    Interior,
    DeepInterior,
}

#[derive(Debug, Clone)]
pub enum MapLegendMeaning {
    Marker,
    Spawn(String),
    Color([f32; 4]),
    Tile(MapTileEntry),
    Texture(String),
    Terrain(MapTerrainEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTileEntry {
    pub texture: String,
    pub collision: MapTileCollision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapTileCollision {
    Solid,
    OneWay,
    None,
}

impl AsciiMapNode {
    pub fn classify_terrain(&self) -> ClassifiedAsciiMap {
        let legend: std::collections::HashMap<char, &MapTerrainEntry> = self
            .legend
            .iter()
            .filter_map(|entry| match &entry.meaning {
                MapLegendMeaning::Terrain(terrain) => Some((entry.symbol, terrain)),
                _ => None,
            })
            .collect();

        let height = self.rows.len();
        let width = self
            .rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        let occupancy = build_terrain_occupancy(self, &legend, width, height);
        let regions = extract_terrain_regions(self, &legend, width, height);
        let region_lookup: std::collections::HashMap<(usize, usize), usize> = regions
            .iter()
            .flat_map(|region| {
                region
                    .cells
                    .iter()
                    .map(|&(row, col)| ((row, col), region.id))
            })
            .collect();
        let distance_lookup: std::collections::HashMap<(usize, usize), u32> = regions
            .iter()
            .flat_map(|region| {
                compute_region_boundary_distances(region)
                    .into_iter()
                    .map(|(cell, distance)| (cell, distance))
            })
            .collect();
        let surface_u_lookup: std::collections::HashMap<(usize, usize), u32> = regions
            .iter()
            .flat_map(|region| compute_region_surface_coordinates(region).into_iter())
            .collect();
        let mut cells = Vec::new();

        for (row, line) in self.rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let Some(terrain) = legend.get(&ch) else {
                    continue;
                };
                let exposed = terrain_exposed_sides(&occupancy, row, col);
                let shape = classify_terrain_shape(&occupancy, row, col);
                let boundary_distance = *distance_lookup.get(&(row, col)).unwrap_or(&0);
                let depth_band = classify_terrain_depth_band(boundary_distance);
                cells.push(ClassifiedMapCell {
                    row,
                    col,
                    symbol: ch,
                    topology: terrain.topology,
                    material_key: terrain.material.clone(),
                    material: terrain_material_for_depth_band(
                        &terrain.material_stack,
                        depth_band,
                        classify_terrain_normal(exposed),
                        classify_terrain_edge_style(terrain.topology, shape),
                    )
                    .to_string(),
                    shape,
                    contour: classify_terrain_contour(terrain.topology, shape),
                    transition_role: TerrainTransitionRole::None,
                    transition_strength: 0,
                    style: classify_terrain_edge_style(terrain.topology, shape),
                    normal: {
                        let normal = classify_terrain_normal(exposed);
                        normal
                    },
                    tangent: classify_terrain_tangent(classify_terrain_normal(exposed)),
                    surface_u: *surface_u_lookup.get(&(row, col)).unwrap_or(&0),
                    boundary_distance,
                    depth_band,
                    region_id: *region_lookup.get(&(row, col)).unwrap_or(&0),
                    exposed,
                    is_boundary: exposed.any(),
                });
            }
        }

        let contour_lookup: std::collections::HashMap<(usize, usize), TerrainContour> = cells
            .iter()
            .map(|cell| ((cell.row, cell.col), cell.contour))
            .collect();

        for cell in &mut cells {
            cell.transition_role =
                classify_terrain_transition_role(cell.row, cell.col, cell.contour, &contour_lookup);
        }
        compute_transition_strengths(&mut cells);

        ClassifiedAsciiMap {
            name: self.name.clone(),
            width,
            height,
            render: self.render,
            cells,
            regions,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Rect(SceneRect),
    Sprite(SceneSprite),
    Text(SceneText),
    HighScore(SceneHighScore),
}

#[derive(Debug, Clone)]
pub struct SceneRect {
    pub anchor: Anchor,
    pub layer: i32,
    pub z: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct SceneSprite {
    pub anchor: Anchor,
    pub layer: i32,
    pub z: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub color: [f32; 4],
    pub textures: Vec<String>,
    pub animations: std::collections::HashMap<String, SpriteAnimation>,
    pub animation_fps: f32,
    pub animation_mode: AnimationMode,
    pub destroy_on_animation_end: bool,
    pub scroll: [f32; 2],
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct SceneText {
    pub anchor: Anchor,
    pub align: TextAlign,
    pub layer: i32,
    pub z: i32,
    pub x: f32,
    pub y: f32,
    pub color: [f32; 4],
    pub value: String,
    pub font: String,
    pub font_size: f32,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct SceneHighScore {
    pub anchor: Anchor,
    pub layer: i32,
    pub z: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub color: [f32; 4],
    pub font: String,
    pub font_size: f32,
    pub items: usize,
    pub gap: f32,
    pub score_digits: usize,
    pub visible: bool,
}

impl RpuProject {
    pub fn create(root: &Path, name: &str) -> Result<Self> {
        if root.exists() {
            bail!("destination already exists: {}", root.display());
        }

        fs::create_dir_all(root.join("scenes"))
            .with_context(|| format!("failed to create {}", root.join("scenes").display()))?;
        fs::create_dir_all(root.join("scripts"))
            .with_context(|| format!("failed to create {}", root.join("scripts").display()))?;
        fs::create_dir_all(root.join("assets"))
            .with_context(|| format!("failed to create {}", root.join("assets").display()))?;

        let manifest = ProjectManifest {
            project: ProjectInfo {
                name: name.to_string(),
                version: default_version(),
                start_scene: default_start_scene(),
            },
            window: WindowConfig::default(),
            debug: DebugConfig::default(),
            meta: MetaConfig::default(),
        };

        fs::write(root.join("rpu.toml"), toml::to_string_pretty(&manifest)?)
            .with_context(|| format!("failed to write {}", root.join("rpu.toml").display()))?;
        fs::write(root.join("scenes/main.rpu"), default_scene(name)).with_context(|| {
            format!("failed to write {}", root.join("scenes/main.rpu").display())
        })?;
        fs::write(root.join("scripts/main.rpu"), default_script()).with_context(|| {
            format!(
                "failed to write {}",
                root.join("scripts/main.rpu").display()
            )
        })?;
        fs::write(root.join(".gitignore"), default_gitignore())
            .with_context(|| format!("failed to write {}", root.join(".gitignore").display()))?;

        Ok(Self {
            root: root.to_path_buf(),
            manifest,
        })
    }

    pub fn load(root: &Path) -> Result<Self> {
        let manifest_path = root.join("rpu.toml");
        let manifest_text = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        let manifest: ProjectManifest =
            toml::from_str(&manifest_text).context("failed to parse rpu.toml")?;

        Ok(Self {
            root: root.to_path_buf(),
            manifest,
        })
    }

    pub fn compile(&self) -> Result<CompiledProject> {
        let paths = self.paths();
        let scenes = collect_source_files(&paths.scenes, &self.root)?;
        let scripts = collect_source_files(&paths.scripts, &self.root)?;
        let assets = collect_asset_files(&paths.assets, &self.root)?
            .into_iter()
            .map(|relative_path| BundledAsset {
                bytes: fs::read(self.root.join(&relative_path)).unwrap_or_default(),
                relative_path,
            })
            .collect::<Vec<_>>();

        compile_project_sources(
            &self.manifest,
            scenes,
            scripts,
            assets,
            self.source_file_count()?,
        )
    }

    pub fn has_source_changes_since(&self, since: Option<SystemTime>) -> Result<bool> {
        let Some(since) = since else {
            return Ok(true);
        };

        for path in self.source_roots() {
            if newest_modification_in_dir(&path)? > Some(since) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn name(&self) -> &str {
        &self.manifest.project.name
    }

    pub fn version(&self) -> &str {
        &self.manifest.project.version
    }

    pub fn start_scene(&self) -> &str {
        &self.manifest.project.start_scene
    }

    pub fn window(&self) -> &WindowConfig {
        &self.manifest.window
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn bundle_id(&self) -> Option<&str> {
        self.manifest.meta.bundle_id.as_deref()
    }

    pub fn display_name(&self) -> Option<&str> {
        self.manifest.meta.display_name.as_deref()
    }

    pub fn development_team(&self) -> Option<&str> {
        self.manifest.meta.development_team.as_deref()
    }

    pub fn paths(&self) -> ProjectPaths {
        ProjectPaths {
            manifest: self.root.join("rpu.toml"),
            scenes: self.root.join("scenes"),
            scripts: self.root.join("scripts"),
            assets: self.root.join("assets"),
        }
    }

    fn source_roots(&self) -> [PathBuf; 2] {
        let paths = self.paths();
        [paths.scenes, paths.scripts]
    }

    fn source_file_count(&self) -> Result<usize> {
        let paths = self.paths();
        Ok(count_source_files(&paths.scenes)? + count_source_files(&paths.scripts)?)
    }
}

impl BundledProject {
    pub fn new(
        manifest_toml: &str,
        scenes: Vec<(PathBuf, String)>,
        scripts: Vec<(PathBuf, String)>,
        assets: Vec<(PathBuf, Vec<u8>)>,
    ) -> Result<Self> {
        let manifest: ProjectManifest =
            toml::from_str(manifest_toml).context("failed to parse bundled rpu.toml")?;
        Ok(Self {
            manifest,
            scenes: scenes
                .into_iter()
                .map(|(relative_path, contents)| SourceFile {
                    relative_path,
                    contents,
                    modified: None,
                })
                .collect(),
            scripts: scripts
                .into_iter()
                .map(|(relative_path, contents)| SourceFile {
                    relative_path,
                    contents,
                    modified: None,
                })
                .collect(),
            assets: assets
                .into_iter()
                .map(|(relative_path, bytes)| BundledAsset {
                    relative_path,
                    bytes,
                })
                .collect(),
        })
    }

    pub fn compile(&self) -> Result<CompiledProject> {
        compile_project_sources(
            &self.manifest,
            self.scenes.clone(),
            self.scripts.clone(),
            self.assets.clone(),
            self.scenes.len() + self.scripts.len(),
        )
    }
}

impl CompiledProject {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn handler_count(&self) -> usize {
        self.bytecode_scripts
            .iter()
            .map(|script| script.handlers.len())
            .sum()
    }

    pub fn op_count(&self) -> usize {
        self.bytecode_scripts
            .iter()
            .flat_map(|script| script.handlers.iter())
            .map(|handler| count_ops(&handler.ops))
            .sum()
    }

    pub fn rect_count(&self) -> usize {
        self.draw_commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Rect(_)))
            .count()
    }

    pub fn scene_count(&self) -> usize {
        self.parsed_scenes
            .iter()
            .map(|document| document.scenes.len())
            .sum()
    }

    pub fn scene_exists(&self, name: &str) -> bool {
        self.parsed_scenes
            .iter()
            .flat_map(|document| document.scenes.iter())
            .any(|scene| scene.name == name)
    }

    pub fn first_scene_name(&self) -> Option<&str> {
        self.parsed_scenes
            .iter()
            .flat_map(|document| document.scenes.iter())
            .map(|scene| scene.name.as_str())
            .next()
    }

    pub fn sprite_count(&self) -> usize {
        self.draw_commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Sprite(_)))
            .count()
    }

    pub fn camera_count(&self) -> usize {
        self.parsed_scenes
            .iter()
            .flat_map(|document| document.scenes.iter())
            .filter(|scene| scene.camera.is_some())
            .count()
    }

    pub fn active_camera(&self) -> SceneCamera {
        self.active_camera_for(&self.start_scene)
    }

    pub fn active_camera_for(&self, scene_name: &str) -> SceneCamera {
        self.parsed_scenes
            .iter()
            .flat_map(|document| document.scenes.iter())
            .find(|scene| scene.name == scene_name)
            .and_then(|scene| scene.camera.as_ref())
            .map(|camera| SceneCamera {
                x: camera.pos[0],
                y: camera.pos[1],
                zoom: camera.zoom,
                background: camera.background,
                follow: camera.follow.clone(),
                follow_offset: camera.follow_offset,
                bounds_min: camera.bounds_min,
                bounds_max: camera.bounds_max,
                follow_smoothing: camera.follow_smoothing,
                dead_zone: camera.dead_zone,
            })
            .unwrap_or_else(SceneCamera::default)
    }
}

#[derive(Debug, Clone)]
pub struct SceneCamera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub background: [f32; 4],
    pub follow: Option<String>,
    pub follow_offset: [f32; 2],
    pub bounds_min: Option<[f32; 2]>,
    pub bounds_max: Option<[f32; 2]>,
    pub follow_smoothing: f32,
    pub dead_zone: [f32; 2],
}

impl Default for SceneCamera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            background: [0.04, 0.05, 0.08, 1.0],
            follow: None,
            follow_offset: [0.0, 0.0],
            bounds_min: None,
            bounds_max: None,
            follow_smoothing: 0.0,
            dead_zone: [0.0, 0.0],
        }
    }
}

impl Diagnostic {
    pub fn warning(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            path,
            line: None,
        }
    }

    pub fn error(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            path,
            line: None,
        }
    }

    pub fn warning_at(message: impl Into<String>, path: Option<PathBuf>, line: usize) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            path,
            line: Some(line),
        }
    }

    pub fn error_at(message: impl Into<String>, path: Option<PathBuf>, line: usize) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            path,
            line: Some(line),
        }
    }
}

fn compile_project_sources(
    manifest: &ProjectManifest,
    scenes: Vec<SourceFile>,
    scripts: Vec<SourceFile>,
    assets: Vec<BundledAsset>,
    source_file_count: usize,
) -> Result<CompiledProject> {
    let mut diagnostics = Vec::new();
    let asset_paths = assets
        .iter()
        .map(|asset| asset.relative_path.clone())
        .collect::<Vec<_>>();

    if scenes.is_empty() {
        diagnostics.push(Diagnostic::error(
            "project does not contain any scene files",
            Some(PathBuf::from("scenes")),
        ));
    }

    let mut parsed_scenes = parse_scene_documents(&scenes, &mut diagnostics);
    let has_main_scene = parsed_scenes.iter().any(|document| {
        document.path == PathBuf::from("scenes/main.rpu")
            || document.scenes.iter().any(|scene| scene.name == "Main")
    });
    if !has_main_scene {
        diagnostics.push(Diagnostic::warning(
            "no main scene detected; expected scenes/main.rpu or `scene Main`",
            Some(PathBuf::from("scenes")),
        ));
    }

    let inline_scripts = collect_inline_script_sources(&parsed_scenes, &scenes, &scripts);
    let script_references = extract_script_references(&parsed_scenes);
    let external_script_references = extract_external_script_references(&parsed_scenes);
    let texture_references = extract_texture_references(&parsed_scenes);
    let font_references = extract_font_references(&parsed_scenes);
    if !scenes.is_empty() && script_references.is_empty() {
        diagnostics.push(Diagnostic::warning(
            "scene files do not reference any script files",
            Some(PathBuf::from("scenes")),
        ));
    }

    if scripts.is_empty() && inline_scripts.is_empty() {
        diagnostics.push(Diagnostic::warning(
            "project does not contain any script files",
            Some(PathBuf::from("scripts")),
        ));
    }

    for script_name in &external_script_references {
        let expected = PathBuf::from("scripts").join(script_name);
        if !scripts
            .iter()
            .any(|script| script.relative_path == expected)
        {
            diagnostics.push(Diagnostic::error(
                format!("referenced script is missing: {}", script_name),
                Some(expected),
            ));
        }
    }

    for texture_name in &texture_references {
        let expected = PathBuf::from("assets").join(texture_name);
        if !asset_paths.iter().any(|asset| asset == &expected) {
            diagnostics.push(Diagnostic::error(
                format!("referenced sprite texture is missing: {}", texture_name),
                Some(expected),
            ));
        }
    }

    for font_name in &font_references {
        let expected = PathBuf::from("assets").join(font_name);
        if !asset_paths.iter().any(|asset| asset == &expected) {
            diagnostics.push(Diagnostic::error(
                format!("referenced text font is missing: {}", font_name),
                Some(expected),
            ));
        }
    }

    resolve_sprite_texture_sizes_from_assets(&assets, &mut parsed_scenes, &mut diagnostics);

    let draw_commands = compile_scene_draw_commands(&parsed_scenes);

    let mut all_script_sources = scripts.clone();
    all_script_sources.extend(inline_scripts);
    let bytecode_scripts = compile_scripts(&all_script_sources, &mut diagnostics);

    let latest_modified = scenes
        .iter()
        .chain(scripts.iter())
        .filter_map(|file| file.modified)
        .max();

    Ok(CompiledProject {
        name: manifest.project.name.clone(),
        version: manifest.project.version.clone(),
        start_scene: manifest.project.start_scene.clone(),
        window: manifest.window.clone(),
        debug: manifest.debug.clone(),
        scenes,
        parsed_scenes,
        scripts,
        bytecode_scripts,
        draw_commands,
        assets: asset_paths,
        diagnostics,
        script_references,
        texture_references,
        fingerprint: ProjectFingerprint {
            latest_modified,
            source_file_count,
        },
    })
}

fn collect_source_files(dir: &Path, root: &Path) -> Result<Vec<SourceFile>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    collect_files_recursive(dir, &mut |path| {
        if path.extension() == Some(OsStr::new("rpu")) {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let metadata = fs::metadata(path)
                .with_context(|| format!("failed to read metadata for {}", path.display()))?;
            out.push(SourceFile {
                relative_path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
                contents,
                modified: metadata.modified().ok(),
            });
        }
        Ok(())
    })?;
    out.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(out)
}

fn collect_asset_files(dir: &Path, root: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    collect_files_recursive(dir, &mut |path| {
        if path.is_file() {
            out.push(path.strip_prefix(root).unwrap_or(path).to_path_buf());
        }
        Ok(())
    })?;
    out.sort();
    Ok(out)
}

fn count_source_files(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0usize;
    collect_files_recursive(dir, &mut |path| {
        if path.extension() == Some(OsStr::new("rpu")) {
            count += 1;
        }
        Ok(())
    })?;
    Ok(count)
}

fn newest_modification_in_dir(dir: &Path) -> Result<Option<SystemTime>> {
    if !dir.exists() {
        return Ok(None);
    }

    let mut latest = None;
    collect_files_recursive(dir, &mut |path| {
        if path.extension() == Some(OsStr::new("rpu")) {
            let modified = fs::metadata(path)
                .with_context(|| format!("failed to read metadata for {}", path.display()))?
                .modified()
                .ok();
            latest = max_system_time(latest, modified);
        }
        Ok(())
    })?;
    Ok(latest)
}

fn collect_files_recursive(dir: &Path, visit: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("failed to read directory entries for {}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(&path, visit)?;
        } else {
            visit(&path)?;
        }
    }
    Ok(())
}

fn parse_scene_documents(
    scenes: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<SceneDocument> {
    scenes
        .iter()
        .map(|scene| parse_scene_document(scene, diagnostics))
        .collect()
}

fn parse_scene_document(scene: &SourceFile, diagnostics: &mut Vec<Diagnostic>) -> SceneDocument {
    let mut parsed = SceneDocument {
        path: scene.relative_path.clone(),
        scenes: Vec::new(),
    };
    let mut current_scene: Option<SceneNode> = None;
    let mut current_camera: Option<CameraNode> = None;
    let mut current_map: Option<AsciiMapNode> = None;
    let mut current_stack: Option<StackNode> = None;
    let mut current_rect: Option<RectNode> = None;
    let mut current_sprite: Option<SpriteNode> = None;
    let mut current_sprite_animation: Option<(String, SpriteAnimation)> = None;
    let mut current_text: Option<TextNode> = None;
    let mut current_high_score: Option<HighScoreNode> = None;
    let mut inline_script_capture: Option<(usize, Vec<String>)> = None;
    let mut in_meta = false;
    let mut in_legend = false;
    let mut in_ascii = false;

    for (index, raw_line) in scene.contents.lines().enumerate() {
        let raw_line = raw_line.trim_end_matches('\r');
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if in_ascii {
            if line == "}" {
                in_ascii = false;
            } else if let Some(map) = current_map.as_mut() {
                map.rows.push(raw_line.to_string());
            }
            continue;
        }

        if let Some((depth, lines)) = inline_script_capture.as_mut() {
            lines.push(raw_line.to_string());
            *depth = update_brace_depth(*depth, raw_line);
            if *depth == 0 {
                let source = lines.join("\n");
                if let Some(rect) = current_rect.as_mut() {
                    append_inline_script(&mut rect.visual, &source);
                } else if let Some(sprite) = current_sprite.as_mut() {
                    append_inline_script(&mut sprite.visual, &source);
                } else if let Some(text) = current_text.as_mut() {
                    append_inline_script(&mut text.visual, &source);
                }
                inline_script_capture = None;
            }
            continue;
        }

        if current_sprite_animation.is_some() {
            if line == "}" {
                if let Some((name, animation)) = current_sprite_animation.take()
                    && let Some(sprite) = current_sprite.as_mut()
                {
                    sprite.animations.insert(name, animation);
                }
                continue;
            }
            if let Some(property) = parse_property(line)
                && let Some((_, animation)) = current_sprite_animation.as_mut()
            {
                apply_sprite_animation_block_property(
                    animation,
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(name) = parse_block_start(line, "scene") {
            if current_scene.is_some() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("nested scene block at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
            }
            current_scene = Some(SceneNode {
                name,
                meta: SceneMeta::default(),
                camera: None,
                maps: Vec::new(),
                stacks: Vec::new(),
                rects: Vec::new(),
                sprites: Vec::new(),
                texts: Vec::new(),
                high_scores: Vec::new(),
            });
            continue;
        }

        if line == "meta {" {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("meta block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
            }
            in_meta = true;
            continue;
        }

        if let Some(name) = parse_block_start(line, "map") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("map block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_map = Some(AsciiMapNode {
                name,
                origin: [0.0, 0.0],
                cell: [32.0, 32.0],
                render: TerrainRenderMode::Basic,
                terrain_style: TerrainStyleSettings::default(),
                legend: Vec::new(),
                rows: Vec::new(),
            });
            continue;
        }

        if let Some(name) = parse_block_start(line, "stack") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("stack block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_stack = Some(StackNode {
                name,
                anchor: Anchor::World,
                pos: [0.0, 0.0],
                size: [120.0, 120.0],
                direction: LayoutDirection::Vertical,
                gap: 8.0,
                align: StackAlign::Center,
            });
            continue;
        }

        if line == "legend {" {
            if current_map.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("legend block outside map at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
            }
            in_legend = true;
            continue;
        }

        if line == "ascii {" {
            if current_map.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("ascii block outside map at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
            }
            in_ascii = true;
            continue;
        }

        if let Some(name) = parse_block_start(line, "rect") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("rect block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            if current_rect.is_some() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("nested rect block at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
            }
            current_rect = Some(RectNode {
                name,
                visual: default_visual_node(0, [120.0, 120.0], [0.95, 0.45, 0.20, 1.0]),
            });
            continue;
        }

        if let Some(name) = parse_block_start(line, "sprite") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("sprite block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_sprite = Some(SpriteNode {
                name,
                visual: VisualNode {
                    size_explicit: false,
                    ..default_visual_node(1, [128.0, 128.0], [1.0, 1.0, 1.0, 1.0])
                },
                textures: Vec::new(),
                animations: std::collections::HashMap::new(),
                animation_fps: 0.0,
                animation_mode: AnimationMode::Loop,
                destroy_on_animation_end: false,
                symbol: None,
                rotation: 0.0,
                scroll: [0.0, 0.0],
                repeat_x: false,
                repeat_y: false,
                flip_x: false,
                flip_y: false,
                collider_offset: [0.0, 0.0],
                collider_size: None,
                physics: PhysicsMode::None,
                physics_settings: PlatformerPhysicsSettings::default(),
            });
            continue;
        }

        if let Some(name) = parse_block_start(line, "animation") {
            if current_sprite.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("animation block outside sprite at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_sprite_animation = Some((name, default_sprite_animation()));
            continue;
        }

        if let Some(name) = parse_block_start(line, "text") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("text block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_text = Some(TextNode {
                name,
                visual: default_visual_node(2, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
                value: String::new(),
                font: String::new(),
                font_size: 16.0,
                align: TextAlign::Left,
            });
            continue;
        }

        if let Some(name) = parse_block_start(line, "highscore") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("highscore block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_high_score = Some(HighScoreNode {
                name,
                visual: default_visual_node(2, [112.0, 64.0], [1.0, 1.0, 1.0, 1.0]),
                font: String::new(),
                font_size: 12.0,
                items: 8,
                gap: 12.0,
                score_digits: 4,
            });
            continue;
        }

        if let Some(name) = parse_block_start(line, "camera") {
            if current_scene.is_none() {
                diagnostics.push(Diagnostic::warning_at(
                    format!("camera block outside scene at line {}", index + 1),
                    Some(scene.relative_path.clone()),
                    index + 1,
                ));
                continue;
            }
            current_camera = Some(CameraNode {
                name,
                pos: [0.0, 0.0],
                zoom: 1.0,
                background: [0.04, 0.05, 0.08, 1.0],
                follow: None,
                follow_offset: [0.0, 0.0],
                bounds_min: None,
                bounds_max: None,
                follow_smoothing: 0.0,
                dead_zone: [0.0, 0.0],
            });
            continue;
        }

        if (current_rect.is_some()
            || current_sprite.is_some()
            || current_text.is_some()
            || current_high_score.is_some())
            && (line.starts_with("on ") || line.starts_with("fn "))
            && line.ends_with('{')
        {
            inline_script_capture =
                Some((update_brace_depth(0, raw_line), vec![raw_line.to_string()]));
            continue;
        }

        if line == "}" {
            if let Some((name, animation)) = current_sprite_animation.take() {
                if let Some(sprite) = current_sprite.as_mut() {
                    sprite.animations.insert(name, animation);
                }
                continue;
            }
            if in_legend {
                in_legend = false;
                continue;
            }
            if let Some(map) = current_map.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    scene_node.maps.push(normalize_ascii_map(map));
                }
                continue;
            }
            if let Some(stack) = current_stack.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    scene_node.stacks.push(stack);
                }
                continue;
            }
            if let Some(camera) = current_camera.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    scene_node.camera = Some(camera);
                }
                continue;
            }
            if let Some(mut sprite) = current_sprite.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    finalize_visual_script_binding(
                        &mut sprite.visual,
                        &scene.relative_path,
                        &scene_node.name,
                        &sprite.name,
                    );
                    scene_node.sprites.push(sprite);
                }
                continue;
            }
            if let Some(mut text) = current_text.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    finalize_visual_script_binding(
                        &mut text.visual,
                        &scene.relative_path,
                        &scene_node.name,
                        &text.name,
                    );
                    scene_node.texts.push(text);
                }
                continue;
            }
            if let Some(high_score) = current_high_score.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    scene_node.high_scores.push(high_score);
                }
                continue;
            }
            if let Some(mut rect) = current_rect.take() {
                if let Some(scene_node) = current_scene.as_mut() {
                    finalize_visual_script_binding(
                        &mut rect.visual,
                        &scene.relative_path,
                        &scene_node.name,
                        &rect.name,
                    );
                    scene_node.rects.push(rect);
                }
                continue;
            }
            if in_meta {
                in_meta = false;
                continue;
            }
            if let Some(scene_node) = current_scene.take() {
                parsed.scenes.push(scene_node);
            }
            continue;
        }

        if in_meta {
            if let Some(property) = parse_property(line) {
                if let Some(("title", PropertyValue::String(title))) = parse_schema_value(
                    META_SCHEMA,
                    &property,
                    index + 1,
                    "meta",
                    &scene.relative_path,
                    diagnostics,
                ) {
                    if let Some(scene_node) = current_scene.as_mut() {
                        scene_node.meta.title = Some(title);
                    }
                }
            }
            continue;
        }

        if in_legend {
            if let Some(property) = parse_property(line) {
                apply_map_legend_property(
                    current_map.as_mut(),
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(rect) = current_rect.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_visual_property(
                    &mut rect.visual,
                    &property,
                    index + 1,
                    "rect",
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(sprite) = current_sprite.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_sprite_property(
                    sprite,
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(text) = current_text.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_text_property(
                    text,
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(high_score) = current_high_score.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_high_score_property(
                    high_score,
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(map) = current_map.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_map_property(map, &property, index + 1, &scene.relative_path, diagnostics);
            }
            continue;
        }

        if let Some(stack) = current_stack.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_stack_property(
                    stack,
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
            continue;
        }

        if let Some(camera) = current_camera.as_mut() {
            if let Some(property) = parse_property(line) {
                apply_camera_property(
                    camera,
                    &property,
                    index + 1,
                    &scene.relative_path,
                    diagnostics,
                );
            }
        }
    }

    if let Some((_, lines)) = inline_script_capture.take() {
        let source = lines.join("\n");
        if let Some(rect) = current_rect.as_mut() {
            append_inline_script(&mut rect.visual, &source);
        } else if let Some(sprite) = current_sprite.as_mut() {
            append_inline_script(&mut sprite.visual, &source);
        } else if let Some(text) = current_text.as_mut() {
            append_inline_script(&mut text.visual, &source);
        }
    }
    if let Some((name, animation)) = current_sprite_animation.take()
        && let Some(sprite) = current_sprite.as_mut()
    {
        sprite.animations.insert(name, animation);
    }
    if let Some(camera) = current_camera.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            scene_node.camera = Some(camera);
        }
    }
    if let Some(map) = current_map.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            scene_node.maps.push(normalize_ascii_map(map));
        }
    }
    if let Some(stack) = current_stack.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            scene_node.stacks.push(stack);
        }
    }
    if let Some(mut sprite) = current_sprite.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            finalize_visual_script_binding(
                &mut sprite.visual,
                &scene.relative_path,
                &scene_node.name,
                &sprite.name,
            );
            scene_node.sprites.push(sprite);
        }
    }
    if let Some(mut text) = current_text.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            finalize_visual_script_binding(
                &mut text.visual,
                &scene.relative_path,
                &scene_node.name,
                &text.name,
            );
            scene_node.texts.push(text);
        }
    }
    if let Some(high_score) = current_high_score.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            scene_node.high_scores.push(high_score);
        }
    }
    if let Some(mut rect) = current_rect.take() {
        if let Some(scene_node) = current_scene.as_mut() {
            finalize_visual_script_binding(
                &mut rect.visual,
                &scene.relative_path,
                &scene_node.name,
                &rect.name,
            );
            scene_node.rects.push(rect);
        }
    }
    if let Some(scene_node) = current_scene.take() {
        parsed.scenes.push(scene_node);
    }

    parsed
}

fn extract_script_references(parsed_scenes: &[SceneDocument]) -> Vec<String> {
    parsed_scenes
        .iter()
        .flat_map(|document| document.scenes.iter())
        .flat_map(|scene| {
            scene
                .rects
                .iter()
                .filter_map(|rect| rect.visual.script_binding.clone())
                .chain(
                    scene
                        .sprites
                        .iter()
                        .filter_map(|sprite| sprite.visual.script_binding.clone()),
                )
                .chain(
                    scene
                        .texts
                        .iter()
                        .filter_map(|text| text.visual.script_binding.clone()),
                )
        })
        .collect()
}

fn extract_external_script_references(parsed_scenes: &[SceneDocument]) -> Vec<String> {
    parsed_scenes
        .iter()
        .flat_map(|document| document.scenes.iter())
        .flat_map(|scene| {
            scene
                .rects
                .iter()
                .filter_map(|rect| rect.visual.script.clone())
                .chain(
                    scene
                        .sprites
                        .iter()
                        .filter_map(|sprite| sprite.visual.script.clone()),
                )
        })
        .collect()
}

fn extract_texture_references(parsed_scenes: &[SceneDocument]) -> Vec<String> {
    parsed_scenes
        .iter()
        .flat_map(|document| document.scenes.iter())
        .flat_map(|scene| {
            let sprite_textures = scene.sprites.iter().flat_map(|sprite| {
                sprite.textures.iter().cloned().chain(
                    sprite
                        .animations
                        .values()
                        .flat_map(|animation| animation.textures.iter().cloned()),
                )
            });
            let map_textures = scene.maps.iter().flat_map(|map| {
                map.legend.iter().filter_map(|entry| match &entry.meaning {
                    MapLegendMeaning::Tile(tile) => Some(tile.texture.clone()),
                    MapLegendMeaning::Texture(texture) => Some(texture.clone()),
                    _ => None,
                })
            });
            sprite_textures.chain(map_textures)
        })
        .collect()
}

fn extract_font_references(parsed_scenes: &[SceneDocument]) -> Vec<String> {
    parsed_scenes
        .iter()
        .flat_map(|document| document.scenes.iter())
        .flat_map(|scene| {
            scene.texts.iter().map(|text| text.font.clone()).chain(
                scene
                    .high_scores
                    .iter()
                    .map(|high_score| high_score.font.clone()),
            )
        })
        .filter(|font| !font.is_empty())
        .collect()
}

fn collect_inline_script_sources(
    parsed_scenes: &[SceneDocument],
    scenes: &[SourceFile],
    scripts: &[SourceFile],
) -> Vec<SourceFile> {
    let scene_modified: std::collections::HashMap<&Path, Option<SystemTime>> = scenes
        .iter()
        .map(|scene| (scene.relative_path.as_path(), scene.modified))
        .collect();
    let script_sources: std::collections::HashMap<&Path, &SourceFile> = scripts
        .iter()
        .map(|script| (script.relative_path.as_path(), script))
        .collect();
    let mut generated = Vec::new();

    for document in parsed_scenes {
        for scene in &document.scenes {
            for rect in &scene.rects {
                if let Some(source) = compile_inline_visual_script(
                    &document.path,
                    scene,
                    &rect.name,
                    &rect.visual,
                    &scene_modified,
                    &script_sources,
                ) {
                    generated.push(source);
                }
            }
            for sprite in &scene.sprites {
                if let Some(source) = compile_inline_visual_script(
                    &document.path,
                    scene,
                    &sprite.name,
                    &sprite.visual,
                    &scene_modified,
                    &script_sources,
                ) {
                    generated.push(source);
                }
            }
            for text in &scene.texts {
                if let Some(source) = compile_inline_visual_script(
                    &document.path,
                    scene,
                    &text.name,
                    &text.visual,
                    &scene_modified,
                    &script_sources,
                ) {
                    generated.push(source);
                }
            }
        }
    }

    generated
}

fn compile_inline_visual_script(
    document_path: &Path,
    scene: &SceneNode,
    node_name: &str,
    visual: &VisualNode,
    scene_modified: &std::collections::HashMap<&Path, Option<SystemTime>>,
    script_sources: &std::collections::HashMap<&Path, &SourceFile>,
) -> Option<SourceFile> {
    let inline = visual.inline_script.as_ref()?;
    let binding = visual
        .script_binding
        .clone()
        .unwrap_or_else(|| inline_script_name(document_path, &scene.name, node_name));
    let mut contents = String::new();

    if let Some(script_name) = &visual.script {
        let path = PathBuf::from("scripts").join(script_name);
        if let Some(source) = script_sources.get(path.as_path()) {
            contents.push_str(&source.contents);
            if !source.contents.ends_with('\n') {
                contents.push('\n');
            }
            contents.push('\n');
        }
    }

    contents.push_str(inline);

    Some(SourceFile {
        relative_path: PathBuf::from("scripts").join(binding),
        contents,
        modified: scene_modified.get(document_path).copied().flatten(),
    })
}

fn estimate_text_width(text: &TextNode) -> f32 {
    text.value.chars().count() as f32 * text.font_size * 0.6
}

fn estimate_text_height(text: &TextNode) -> f32 {
    text.font_size * 1.2
}

pub fn apply_scene_layout(scene: &SceneNode) -> SceneNode {
    let mut scene = scene.clone();
    for stack in &scene.stacks.clone() {
        let mut placements: Vec<(String, [f32; 2], Option<TextAlign>, Anchor)> = Vec::new();
        let mut ordered: Vec<(String, i32, [f32; 2], bool)> = Vec::new();

        for rect in &scene.rects {
            if rect.visual.parent.as_deref() == Some(stack.name.as_str()) {
                ordered.push((
                    rect.name.clone(),
                    rect.visual.order,
                    rect.visual.size,
                    false,
                ));
            }
        }
        for sprite in &scene.sprites {
            if sprite.visual.parent.as_deref() == Some(stack.name.as_str()) {
                ordered.push((
                    sprite.name.clone(),
                    sprite.visual.order,
                    sprite.visual.size,
                    false,
                ));
            }
        }
        for text in &scene.texts {
            if text.visual.parent.as_deref() == Some(stack.name.as_str()) {
                ordered.push((
                    text.name.clone(),
                    text.visual.order,
                    [estimate_text_width(text), estimate_text_height(text)],
                    true,
                ));
            }
        }
        for high_score in &scene.high_scores {
            if high_score.visual.parent.as_deref() == Some(stack.name.as_str()) {
                ordered.push((
                    high_score.name.clone(),
                    high_score.visual.order,
                    high_score.visual.size,
                    false,
                ));
            }
        }

        ordered.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        let mut cursor = 0.0;
        for (name, _order, size, is_text) in ordered {
            let (pos, align_override) = match stack.direction {
                LayoutDirection::Vertical => {
                    let x = match (stack.align, is_text) {
                        (StackAlign::Start, true) => 0.0,
                        (StackAlign::Center, true) => stack.size[0] * 0.5,
                        (StackAlign::End, true) => stack.size[0],
                        (StackAlign::Start, false) => 0.0,
                        (StackAlign::Center, false) => (stack.size[0] - size[0]) * 0.5,
                        (StackAlign::End, false) => stack.size[0] - size[0],
                    };
                    let align = if is_text {
                        Some(match stack.align {
                            StackAlign::Start => TextAlign::Left,
                            StackAlign::Center => TextAlign::Center,
                            StackAlign::End => TextAlign::Right,
                        })
                    } else {
                        None
                    };
                    let pos = [stack.pos[0] + x, stack.pos[1] + cursor];
                    cursor += size[1] + stack.gap;
                    (pos, align)
                }
                LayoutDirection::Horizontal => {
                    let y = match stack.align {
                        StackAlign::Start => 0.0,
                        StackAlign::Center => (stack.size[1] - size[1]) * 0.5,
                        StackAlign::End => stack.size[1] - size[1],
                    };
                    let pos = [stack.pos[0] + cursor, stack.pos[1] + y];
                    cursor += size[0] + stack.gap;
                    (pos, None)
                }
            };
            placements.push((name, pos, align_override, stack.anchor));
        }

        for (name, pos, align_override, anchor) in placements {
            if let Some(rect) = scene.rects.iter_mut().find(|node| node.name == name) {
                rect.visual.pos = pos;
                rect.visual.anchor = anchor;
                continue;
            }
            if let Some(sprite) = scene.sprites.iter_mut().find(|node| node.name == name) {
                sprite.visual.pos = pos;
                sprite.visual.anchor = anchor;
                continue;
            }
            if let Some(text) = scene.texts.iter_mut().find(|node| node.name == name) {
                text.visual.pos = pos;
                text.visual.anchor = anchor;
                if let Some(align) = align_override {
                    text.align = align;
                }
                continue;
            }
            if let Some(high_score) = scene.high_scores.iter_mut().find(|node| node.name == name) {
                high_score.visual.pos = pos;
                high_score.visual.anchor = anchor;
            }
        }
    }
    scene
}

fn compile_scene_draw_commands(parsed_scenes: &[SceneDocument]) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    for document in parsed_scenes {
        for scene in &document.scenes {
            let scene = apply_scene_layout(scene);
            let markers = compile_map_markers(&scene.maps);
            commands.extend(scene.maps.iter().flat_map(compile_map_rects));
            for rect in &scene.rects {
                if !rect.visual.visible || rect.visual.template {
                    continue;
                }
                commands.push(DrawCommand::Rect(SceneRect {
                    anchor: rect.visual.anchor,
                    layer: rect.visual.layer,
                    z: rect.visual.z,
                    x: rect.visual.pos[0],
                    y: rect.visual.pos[1],
                    width: rect.visual.size[0],
                    height: rect.visual.size[1],
                    color: rect.visual.color,
                    visible: rect.visual.visible,
                }));
            }
            for sprite in &scene.sprites {
                if !sprite.visual.visible || sprite.visual.template {
                    continue;
                }
                let pos = sprite
                    .symbol
                    .as_deref()
                    .and_then(|symbol| markers.get(symbol))
                    .or_else(|| markers.get(&sprite.name))
                    .copied()
                    .unwrap_or(sprite.visual.pos);
                commands.push(DrawCommand::Sprite(SceneSprite {
                    anchor: sprite.visual.anchor,
                    layer: sprite.visual.layer,
                    z: sprite.visual.z,
                    x: pos[0],
                    y: pos[1],
                    width: sprite.visual.size[0],
                    height: sprite.visual.size[1],
                    rotation: sprite.rotation,
                    color: sprite.visual.color,
                    textures: sprite.textures.clone(),
                    animations: sprite.animations.clone(),
                    animation_fps: sprite.animation_fps,
                    animation_mode: sprite.animation_mode,
                    destroy_on_animation_end: sprite.destroy_on_animation_end,
                    scroll: sprite.scroll,
                    repeat_x: sprite.repeat_x,
                    repeat_y: sprite.repeat_y,
                    flip_x: sprite.flip_x,
                    flip_y: sprite.flip_y,
                    visible: sprite.visual.visible,
                }));
            }
            for text in &scene.texts {
                if !text.visual.visible || text.visual.template {
                    continue;
                }
                commands.push(DrawCommand::Text(SceneText {
                    anchor: text.visual.anchor,
                    align: text.align,
                    layer: text.visual.layer,
                    z: text.visual.z,
                    x: text.visual.pos[0],
                    y: text.visual.pos[1],
                    color: text.visual.color,
                    value: text.value.clone(),
                    font: text.font.clone(),
                    font_size: text.font_size,
                    visible: text.visual.visible,
                }));
            }
            for high_score in &scene.high_scores {
                if !high_score.visual.visible || high_score.visual.template {
                    continue;
                }
                commands.push(DrawCommand::HighScore(SceneHighScore {
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
        }
    }
    commands
}

fn compile_map_rects(map: &AsciiMapNode) -> Vec<DrawCommand> {
    let legend: std::collections::HashMap<char, &MapLegendMeaning> = map
        .legend
        .iter()
        .map(|entry| (entry.symbol, &entry.meaning))
        .collect();
    let classified = map.classify_terrain();
    let terrain_cells: std::collections::HashMap<(usize, usize), &ClassifiedMapCell> = classified
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
            if let Some(texture) = legend_tile_texture(legend.get(&ch)) {
                commands.push(DrawCommand::Sprite(SceneSprite {
                    anchor: Anchor::World,
                    layer: -10,
                    z: (row as i32) * 100 + col as i32,
                    x: map.origin[0] + col as f32 * map.cell[0],
                    y: map.origin[1] + row as f32 * map.cell[1],
                    width: map.cell[0],
                    height: map.cell[1],
                    rotation: 0.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    textures: vec![texture],
                    animations: std::collections::HashMap::new(),
                    animation_fps: 0.0,
                    animation_mode: AnimationMode::Loop,
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
                commands.push(DrawCommand::Rect(SceneRect {
                    anchor: Anchor::World,
                    layer: -10,
                    z: (row as i32) * 100 + col as i32,
                    x: map.origin[0] + col as f32 * map.cell[0],
                    y: map.origin[1] + row as f32 * map.cell[1],
                    width: map.cell[0],
                    height: map.cell[1],
                    color: terrain_shape_debug_color(cell.shape),
                    visible: true,
                }));
            }
        }
    }
    commands
}

fn legend_tile_texture(meaning: Option<&&MapLegendMeaning>) -> Option<String> {
    match meaning {
        Some(MapLegendMeaning::Tile(tile)) => Some(tile.texture.clone()),
        Some(MapLegendMeaning::Texture(texture)) => Some((*texture).clone()),
        _ => None,
    }
}

fn compile_map_markers(maps: &[AsciiMapNode]) -> std::collections::HashMap<String, [f32; 2]> {
    let mut markers = std::collections::HashMap::new();
    for map in maps {
        let legend: std::collections::HashMap<char, &MapLegendMeaning> = map
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

fn parse_block_start(line: &str, keyword: &str) -> Option<String> {
    let rest = line.strip_prefix(keyword)?.trim();
    let rest = rest.strip_suffix('{')?.trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

struct Property<'a> {
    key: &'a str,
    raw: &'a str,
}

#[derive(Clone, Copy)]
enum PropertyKind {
    String,
    BareString,
    StringList,
    Bool,
    I32,
    F32,
    Vec2,
    Color,
    Symbol,
}

#[derive(Clone, Copy)]
struct SchemaEntry {
    key: &'static str,
    kind: PropertyKind,
}

enum PropertyValue {
    String(String),
    StringList(Vec<String>),
    Bool(bool),
    I32(i32),
    F32(f32),
    Vec2([f32; 2]),
    Color([f32; 4]),
    Symbol(String),
}

const META_SCHEMA: &[SchemaEntry] = &[SchemaEntry {
    key: "title",
    kind: PropertyKind::String,
}];

const VISUAL_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "visible",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "template",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "group",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "parent",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "order",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "anchor",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "layer",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "z",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "pos",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "size",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "color",
        kind: PropertyKind::Color,
    },
    SchemaEntry {
        key: "script",
        kind: PropertyKind::String,
    },
];

const SPRITE_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "texture",
        kind: PropertyKind::StringList,
    },
    SchemaEntry {
        key: "animation_mode",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "symbol",
        kind: PropertyKind::Symbol,
    },
    SchemaEntry {
        key: "rotation",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "scroll",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "repeat_x",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "repeat_y",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "flip_x",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "flip_y",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "collider_offset",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "collider_size",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "physics",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "acceleration",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "friction",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "max_speed",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "gravity",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "jump_speed",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "max_fall_speed",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "coyote_time",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "jump_buffer",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "animation_fps",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "destroy_on_animation_end",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "visible",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "template",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "group",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "parent",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "order",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "anchor",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "layer",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "z",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "pos",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "size",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "color",
        kind: PropertyKind::Color,
    },
    SchemaEntry {
        key: "script",
        kind: PropertyKind::String,
    },
];

const SPRITE_ANIMATION_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "frames",
        kind: PropertyKind::StringList,
    },
    SchemaEntry {
        key: "fps",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "mode",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "loop",
        kind: PropertyKind::Bool,
    },
];

const TEXT_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "value",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "font",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "font_size",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "align",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "visible",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "template",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "group",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "parent",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "order",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "anchor",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "layer",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "z",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "pos",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "color",
        kind: PropertyKind::Color,
    },
    SchemaEntry {
        key: "script",
        kind: PropertyKind::String,
    },
];

const HIGHSCORE_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "font",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "font_size",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "items",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "gap",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "score_digits",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "visible",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "template",
        kind: PropertyKind::Bool,
    },
    SchemaEntry {
        key: "group",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "parent",
        kind: PropertyKind::String,
    },
    SchemaEntry {
        key: "order",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "anchor",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "layer",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "z",
        kind: PropertyKind::I32,
    },
    SchemaEntry {
        key: "pos",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "size",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "color",
        kind: PropertyKind::Color,
    },
];

const STACK_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "anchor",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "pos",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "size",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "direction",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "gap",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "align",
        kind: PropertyKind::BareString,
    },
];

const CAMERA_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "pos",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "zoom",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "background",
        kind: PropertyKind::Color,
    },
    SchemaEntry {
        key: "follow",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "follow_offset",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "bounds_min",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "bounds_max",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "follow_smoothing",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "dead_zone",
        kind: PropertyKind::Vec2,
    },
];

const MAP_SCHEMA: &[SchemaEntry] = &[
    SchemaEntry {
        key: "origin",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "cell",
        kind: PropertyKind::Vec2,
    },
    SchemaEntry {
        key: "render",
        kind: PropertyKind::BareString,
    },
    SchemaEntry {
        key: "cap_depth",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "ramp_cap_depth",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "join_cap_depth",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "shoulder_width",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "surface_roughness",
        kind: PropertyKind::F32,
    },
    SchemaEntry {
        key: "shoulder_shape",
        kind: PropertyKind::BareString,
    },
];

impl<'a> Property<'a> {
    fn as_string(&self) -> Option<String> {
        parse_string(self.raw)
    }

    fn as_bare_string(&self) -> Option<String> {
        let value = self.raw.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self.raw.trim() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    fn as_i32(&self) -> Option<i32> {
        self.raw.parse().ok()
    }

    fn as_f32(&self) -> Option<f32> {
        self.raw.parse().ok()
    }

    fn as_vec2(&self) -> Option<[f32; 2]> {
        parse_f32_tuple::<2>(self.raw)
    }

    fn as_color(&self) -> Option<[f32; 4]> {
        parse_color(self.raw)
    }

    fn parse_as(&self, kind: PropertyKind) -> Option<PropertyValue> {
        match kind {
            PropertyKind::String => self.as_string().map(PropertyValue::String),
            PropertyKind::BareString => self.as_bare_string().map(PropertyValue::String),
            PropertyKind::StringList => parse_string_list(self.raw).map(PropertyValue::StringList),
            PropertyKind::Bool => self.as_bool().map(PropertyValue::Bool),
            PropertyKind::I32 => self.as_i32().map(PropertyValue::I32),
            PropertyKind::F32 => self.as_f32().map(PropertyValue::F32),
            PropertyKind::Vec2 => self.as_vec2().map(PropertyValue::Vec2),
            PropertyKind::Color => self.as_color().map(PropertyValue::Color),
            PropertyKind::Symbol => parse_symbol_ref(self.raw).map(PropertyValue::Symbol),
        }
    }
}

fn parse_property(line: &str) -> Option<Property<'_>> {
    let (key, raw) = line.split_once('=')?;
    Some(Property {
        key: key.trim(),
        raw: raw.trim(),
    })
}

fn parse_string(value: &str) -> Option<String> {
    Some(value.strip_prefix('"')?.strip_suffix('"')?.to_string())
}

fn parse_string_list(value: &str) -> Option<Vec<String>> {
    if let Some(single) = parse_string(value) {
        return Some(vec![single]);
    }

    let inner = value.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }

    inner
        .split(',')
        .map(|part| parse_string(part.trim()))
        .collect()
}

fn parse_symbol_ref(value: &str) -> Option<String> {
    if let Some(value) = parse_string(value) {
        if value.chars().count() == 1 {
            return Some(value);
        }
        return None;
    }

    let value = value.trim();
    if value.chars().count() == 1 {
        return Some(value.to_string());
    }

    None
}

fn append_inline_script(visual: &mut VisualNode, source: &str) {
    match visual.inline_script.as_mut() {
        Some(existing) => {
            if !existing.is_empty() {
                existing.push_str("\n\n");
            }
            existing.push_str(source);
            existing.push('\n');
        }
        None => {
            visual.inline_script = Some(format!("{source}\n"));
        }
    }
}

fn update_brace_depth(current: usize, line: &str) -> usize {
    let opens = line.chars().filter(|ch| *ch == '{').count();
    let closes = line.chars().filter(|ch| *ch == '}').count();
    current.saturating_add(opens).saturating_sub(closes)
}

fn finalize_visual_script_binding(
    visual: &mut VisualNode,
    document_path: &Path,
    scene_name: &str,
    node_name: &str,
) {
    visual.script_binding = if visual.inline_script.is_some() {
        Some(inline_script_name(document_path, scene_name, node_name))
    } else {
        visual.script.clone()
    };
}

fn inline_script_name(document_path: &Path, scene_name: &str, node_name: &str) -> String {
    let document = sanitize_script_name(&document_path.to_string_lossy());
    let scene = sanitize_script_name(scene_name);
    let node = sanitize_script_name(node_name);
    format!("__inline__/{document}__{scene}__{node}.rpu")
}

fn sanitize_script_name(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

fn schema_entry<'a>(schema: &'a [SchemaEntry], property: &Property<'_>) -> Option<&'a SchemaEntry> {
    schema.iter().find(|entry| entry.key == property.key)
}

fn parse_schema_value(
    schema: &[SchemaEntry],
    property: &Property<'_>,
    line: usize,
    kind: &str,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(&'static str, PropertyValue)> {
    let entry = schema_entry(schema, property)?;
    match property.parse_as(entry.kind) {
        Some(value) => Some((entry.key, value)),
        None => {
            diagnostics.push(Diagnostic::warning_at(
                format!("invalid {kind} {}", entry.key),
                Some(path.to_path_buf()),
                line,
            ));
            None
        }
    }
}

fn default_visual_node(layer: i32, size: [f32; 2], color: [f32; 4]) -> VisualNode {
    VisualNode {
        visible: true,
        template: false,
        group: None,
        parent: None,
        order: 0,
        anchor: Anchor::World,
        layer,
        z: 0,
        pos: [0.0, 0.0],
        size,
        size_explicit: true,
        color,
        script: None,
        script_binding: None,
        inline_script: None,
    }
}

fn parse_anchor(value: &str) -> Option<Anchor> {
    match value {
        "world" => Some(Anchor::World),
        "top_left" => Some(Anchor::TopLeft),
        "top" => Some(Anchor::Top),
        "top_right" => Some(Anchor::TopRight),
        "left" => Some(Anchor::Left),
        "center" => Some(Anchor::Center),
        "right" => Some(Anchor::Right),
        "bottom_left" => Some(Anchor::BottomLeft),
        "bottom" => Some(Anchor::Bottom),
        "bottom_right" => Some(Anchor::BottomRight),
        _ => None,
    }
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value {
        "left" => Some(TextAlign::Left),
        "center" => Some(TextAlign::Center),
        "right" => Some(TextAlign::Right),
        _ => None,
    }
}

fn parse_layout_direction(value: &str) -> Option<LayoutDirection> {
    match value {
        "vertical" => Some(LayoutDirection::Vertical),
        "horizontal" => Some(LayoutDirection::Horizontal),
        _ => None,
    }
}

fn parse_stack_align(value: &str) -> Option<StackAlign> {
    match value {
        "start" | "left" | "top" => Some(StackAlign::Start),
        "center" => Some(StackAlign::Center),
        "end" | "right" | "bottom" => Some(StackAlign::End),
        _ => None,
    }
}

fn apply_visual_property(
    visual: &mut VisualNode,
    property: &Property<'_>,
    line: usize,
    kind: &str,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) =
        parse_schema_value(VISUAL_SCHEMA, property, line, kind, path, diagnostics)
    {
        match (key, value) {
            ("visible", PropertyValue::Bool(visible)) => visual.visible = visible,
            ("template", PropertyValue::Bool(template)) => visual.template = template,
            ("group", PropertyValue::String(group)) => visual.group = Some(group),
            ("parent", PropertyValue::String(parent)) => visual.parent = Some(parent),
            ("order", PropertyValue::I32(order)) => visual.order = order,
            ("anchor", PropertyValue::String(anchor)) => match parse_anchor(&anchor) {
                Some(anchor) => visual.anchor = anchor,
                None => diagnostics.push(Diagnostic::warning_at(
                    format!("invalid {kind} anchor"),
                    Some(path.to_path_buf()),
                    line,
                )),
            },
            ("layer", PropertyValue::I32(layer)) => visual.layer = layer,
            ("z", PropertyValue::I32(z)) => visual.z = z,
            ("pos", PropertyValue::Vec2(pos)) => visual.pos = pos,
            ("size", PropertyValue::Vec2(size)) => {
                visual.size = [size[0].max(1.0), size[1].max(1.0)];
                visual.size_explicit = true;
            }
            ("color", PropertyValue::Color(color)) => visual.color = color,
            ("script", PropertyValue::String(script)) => visual.script = Some(script),
            _ => {}
        }
    }
}

fn apply_stack_property(
    stack: &mut StackNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) =
        parse_schema_value(STACK_SCHEMA, property, line, "stack", path, diagnostics)
    {
        match (key, value) {
            ("anchor", PropertyValue::String(anchor)) => match parse_anchor(&anchor) {
                Some(anchor) => stack.anchor = anchor,
                None => diagnostics.push(Diagnostic::warning_at(
                    "invalid stack anchor".to_string(),
                    Some(path.to_path_buf()),
                    line,
                )),
            },
            ("pos", PropertyValue::Vec2(pos)) => stack.pos = pos,
            ("size", PropertyValue::Vec2(size)) => stack.size = size,
            ("direction", PropertyValue::String(direction)) => {
                match parse_layout_direction(&direction) {
                    Some(direction) => stack.direction = direction,
                    None => diagnostics.push(Diagnostic::warning_at(
                        "invalid stack direction".to_string(),
                        Some(path.to_path_buf()),
                        line,
                    )),
                }
            }
            ("gap", PropertyValue::F32(gap)) => stack.gap = gap.max(0.0),
            ("align", PropertyValue::String(align)) => match parse_stack_align(&align) {
                Some(align) => stack.align = align,
                None => diagnostics.push(Diagnostic::warning_at(
                    "invalid stack align".to_string(),
                    Some(path.to_path_buf()),
                    line,
                )),
            },
            _ => {}
        }
    }
}

fn resolve_sprite_texture_sizes_from_assets(
    assets: &[BundledAsset],
    parsed_scenes: &mut [SceneDocument],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut texture_sizes = std::collections::HashMap::new();
    for asset in assets {
        let Ok(image) = image::load_from_memory(&asset.bytes) else {
            continue;
        };
        let width = image.width();
        let height = image.height();
        texture_sizes.insert(
            asset.relative_path.clone(),
            [width.max(1) as f32, height.max(1) as f32],
        );
    }

    for document in parsed_scenes {
        for scene in &mut document.scenes {
            for sprite in &mut scene.sprites {
                if sprite.visual.size_explicit {
                    continue;
                }
                let Some(texture_name) = sprite.textures.first() else {
                    continue;
                };
                let asset_path = PathBuf::from("assets").join(texture_name);
                match texture_sizes.get(&asset_path) {
                    Some(size) => sprite.visual.size = *size,
                    None => diagnostics.push(Diagnostic::warning(
                        format!(
                            "sprite `{}` does not declare a size and texture dimensions could not be resolved",
                            sprite.name
                        ),
                        Some(document.path.clone()),
                    )),
                }
            }
        }
    }
}

fn apply_camera_property(
    camera: &mut CameraNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) =
        parse_schema_value(CAMERA_SCHEMA, property, line, "camera", path, diagnostics)
    {
        match (key, value) {
            ("pos", PropertyValue::Vec2(pos)) => camera.pos = pos,
            ("zoom", PropertyValue::F32(zoom)) => camera.zoom = zoom.max(0.01),
            ("background", PropertyValue::Color(color)) => camera.background = color,
            ("follow", PropertyValue::String(name)) => camera.follow = Some(name),
            ("follow_offset", PropertyValue::Vec2(offset)) => camera.follow_offset = offset,
            ("bounds_min", PropertyValue::Vec2(bounds)) => camera.bounds_min = Some(bounds),
            ("bounds_max", PropertyValue::Vec2(bounds)) => camera.bounds_max = Some(bounds),
            ("follow_smoothing", PropertyValue::F32(smoothing)) => {
                camera.follow_smoothing = smoothing.max(0.0)
            }
            ("dead_zone", PropertyValue::Vec2(dead_zone)) => {
                camera.dead_zone = [dead_zone[0].max(0.0), dead_zone[1].max(0.0)]
            }
            _ => {}
        }
    }
}

fn apply_sprite_property(
    sprite: &mut SpriteNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if apply_sprite_animation_property(sprite, property, line, path, diagnostics) {
        return;
    }

    if let Some((key, value)) =
        parse_schema_value(SPRITE_SCHEMA, property, line, "sprite", path, diagnostics)
    {
        match (key, value) {
            ("texture", PropertyValue::StringList(textures)) => sprite.textures = textures,
            ("animation_fps", PropertyValue::F32(fps)) => sprite.animation_fps = fps.max(0.0),
            ("animation_mode", PropertyValue::String(mode)) => {
                sprite.animation_mode = parse_animation_mode(&mode)
            }
            ("destroy_on_animation_end", PropertyValue::Bool(value)) => {
                sprite.destroy_on_animation_end = value
            }
            ("symbol", PropertyValue::Symbol(symbol)) => sprite.symbol = Some(symbol),
            ("rotation", PropertyValue::F32(rotation)) => sprite.rotation = rotation,
            ("scroll", PropertyValue::Vec2(scroll)) => sprite.scroll = scroll,
            ("repeat_x", PropertyValue::Bool(repeat_x)) => sprite.repeat_x = repeat_x,
            ("repeat_y", PropertyValue::Bool(repeat_y)) => sprite.repeat_y = repeat_y,
            ("flip_x", PropertyValue::Bool(flip_x)) => sprite.flip_x = flip_x,
            ("flip_y", PropertyValue::Bool(flip_y)) => sprite.flip_y = flip_y,
            ("collider_offset", PropertyValue::Vec2(offset)) => sprite.collider_offset = offset,
            ("collider_size", PropertyValue::Vec2(size)) => {
                sprite.collider_size = Some([size[0].max(1.0), size[1].max(1.0)])
            }
            ("physics", PropertyValue::String(mode)) => {
                sprite.physics = match mode.as_str() {
                    "platformer" => PhysicsMode::Platformer,
                    _ => PhysicsMode::None,
                }
            }
            ("acceleration", PropertyValue::F32(value)) => {
                sprite.physics_settings.acceleration = value.max(0.0)
            }
            ("friction", PropertyValue::F32(value)) => {
                sprite.physics_settings.friction = value.max(0.0)
            }
            ("max_speed", PropertyValue::F32(value)) => {
                sprite.physics_settings.max_speed = value.max(0.0)
            }
            ("gravity", PropertyValue::F32(value)) => {
                sprite.physics_settings.gravity = value.max(0.0)
            }
            ("jump_speed", PropertyValue::F32(value)) => {
                sprite.physics_settings.jump_speed = value.max(0.0)
            }
            ("max_fall_speed", PropertyValue::F32(value)) => {
                sprite.physics_settings.max_fall_speed = value.max(0.0)
            }
            ("coyote_time", PropertyValue::F32(value)) => {
                sprite.physics_settings.coyote_time = value.max(0.0)
            }
            ("jump_buffer", PropertyValue::F32(value)) => {
                sprite.physics_settings.jump_buffer = value.max(0.0)
            }
            _ => apply_visual_property(
                &mut sprite.visual,
                property,
                line,
                "sprite",
                path,
                diagnostics,
            ),
        }
    }
}

fn apply_sprite_animation_property(
    sprite: &mut SpriteNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if matches!(property.key, "animation_fps" | "animation_mode") {
        return false;
    }
    let Some(rest) = property.key.strip_prefix("animation_") else {
        return false;
    };
    if let Some(name) = rest.strip_suffix("_fps") {
        match property.as_f32() {
            Some(fps) => {
                sprite
                    .animations
                    .entry(name.to_string())
                    .or_insert_with(default_sprite_animation)
                    .fps = fps.max(0.0);
            }
            None => diagnostics.push(Diagnostic::warning_at(
                "invalid sprite animation fps",
                Some(path.to_path_buf()),
                line,
            )),
        }
        return true;
    }
    if let Some(name) = rest.strip_suffix("_mode") {
        match property.as_string().or_else(|| property.as_bare_string()) {
            Some(mode) => {
                sprite
                    .animations
                    .entry(name.to_string())
                    .or_insert_with(default_sprite_animation)
                    .mode = parse_animation_mode(&mode);
            }
            None => diagnostics.push(Diagnostic::warning_at(
                "invalid sprite animation mode",
                Some(path.to_path_buf()),
                line,
            )),
        }
        return true;
    }

    match parse_string_list(property.raw) {
        Some(textures) => {
            sprite
                .animations
                .entry(rest.to_string())
                .or_insert_with(default_sprite_animation)
                .textures = textures;
        }
        None => diagnostics.push(Diagnostic::warning_at(
            "invalid sprite animation texture list",
            Some(path.to_path_buf()),
            line,
        )),
    }
    true
}

fn apply_sprite_animation_block_property(
    animation: &mut SpriteAnimation,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) = parse_schema_value(
        SPRITE_ANIMATION_SCHEMA,
        property,
        line,
        "sprite animation",
        path,
        diagnostics,
    ) {
        match (key, value) {
            ("frames", PropertyValue::StringList(textures)) => animation.textures = textures,
            ("fps", PropertyValue::F32(fps)) => animation.fps = fps.max(0.0),
            ("mode", PropertyValue::String(mode)) => {
                animation.mode = parse_animation_mode(&mode);
            }
            ("loop", PropertyValue::Bool(looping)) => {
                animation.mode = if looping {
                    AnimationMode::Loop
                } else {
                    AnimationMode::Once
                };
            }
            _ => {}
        }
    }
}

fn default_sprite_animation() -> SpriteAnimation {
    SpriteAnimation {
        textures: Vec::new(),
        fps: 0.0,
        mode: AnimationMode::Loop,
    }
}

fn parse_animation_mode(mode: &str) -> AnimationMode {
    match mode {
        "once" => AnimationMode::Once,
        _ => AnimationMode::Loop,
    }
}

fn apply_text_property(
    text: &mut TextNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) =
        parse_schema_value(TEXT_SCHEMA, property, line, "text", path, diagnostics)
    {
        match (key, value) {
            ("value", PropertyValue::String(value)) => text.value = value,
            ("font", PropertyValue::String(font)) => text.font = font,
            ("font_size", PropertyValue::F32(size)) => text.font_size = size.max(1.0),
            ("align", PropertyValue::String(align)) => match parse_text_align(&align) {
                Some(align) => text.align = align,
                None => diagnostics.push(Diagnostic::warning_at(
                    "invalid text align".to_string(),
                    Some(path.to_path_buf()),
                    line,
                )),
            },
            _ => apply_visual_property(&mut text.visual, property, line, "text", path, diagnostics),
        }
    }
}

fn apply_high_score_property(
    high_score: &mut HighScoreNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) = parse_schema_value(
        HIGHSCORE_SCHEMA,
        property,
        line,
        "highscore",
        path,
        diagnostics,
    ) {
        match (key, value) {
            ("font", PropertyValue::String(font)) => high_score.font = font,
            ("font_size", PropertyValue::F32(size)) => high_score.font_size = size.max(1.0),
            ("items", PropertyValue::I32(items)) => high_score.items = items.max(1) as usize,
            ("gap", PropertyValue::F32(gap)) => high_score.gap = gap.max(1.0),
            ("score_digits", PropertyValue::I32(digits)) => {
                high_score.score_digits = digits.max(1) as usize
            }
            _ => apply_visual_property(
                &mut high_score.visual,
                property,
                line,
                "highscore",
                path,
                diagnostics,
            ),
        }
    }
}

fn apply_map_property(
    map: &mut AsciiMapNode,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some((key, value)) =
        parse_schema_value(MAP_SCHEMA, property, line, "map", path, diagnostics)
    {
        match (key, value) {
            ("origin", PropertyValue::Vec2(origin)) => map.origin = origin,
            ("cell", PropertyValue::Vec2(cell)) => {
                map.cell = [cell[0].max(1.0), cell[1].max(1.0)];
            }
            ("render", PropertyValue::String(render)) => match parse_terrain_render_mode(&render) {
                Some(mode) => map.render = mode,
                None => diagnostics.push(Diagnostic::warning_at(
                    "invalid map render mode".to_string(),
                    Some(path.to_path_buf()),
                    line,
                )),
            },
            ("cap_depth", PropertyValue::F32(value)) => {
                map.terrain_style.cap_depth = value.clamp(0.05, 1.5);
            }
            ("ramp_cap_depth", PropertyValue::F32(value)) => {
                map.terrain_style.ramp_cap_depth = value.clamp(0.05, 1.5);
            }
            ("join_cap_depth", PropertyValue::F32(value)) => {
                map.terrain_style.join_cap_depth = value.clamp(0.05, 1.5);
            }
            ("shoulder_width", PropertyValue::F32(value)) => {
                map.terrain_style.shoulder_width = value.clamp(0.0, 1.0);
            }
            ("surface_roughness", PropertyValue::F32(value)) => {
                map.terrain_style.surface_roughness = value.clamp(0.0, 0.25);
            }
            ("shoulder_shape", PropertyValue::String(shape)) => {
                match parse_terrain_shoulder_shape(&shape) {
                    Some(shape) => map.terrain_style.shoulder_shape = shape,
                    None => diagnostics.push(Diagnostic::warning_at(
                        "invalid terrain shoulder shape".to_string(),
                        Some(path.to_path_buf()),
                        line,
                    )),
                }
            }
            _ => {}
        }
    }
}

fn apply_map_legend_property(
    map: Option<&mut AsciiMapNode>,
    property: &Property<'_>,
    line: usize,
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(map) = map else {
        diagnostics.push(Diagnostic::warning_at(
            "legend entry outside map",
            Some(path.to_path_buf()),
            line,
        ));
        return;
    };

    let Some(symbol) = parse_symbol_ref(property.key) else {
        diagnostics.push(Diagnostic::warning_at(
            "invalid legend symbol",
            Some(path.to_path_buf()),
            line,
        ));
        return;
    };

    let Some(symbol) = symbol.chars().next() else {
        diagnostics.push(Diagnostic::warning_at(
            "invalid legend symbol",
            Some(path.to_path_buf()),
            line,
        ));
        return;
    };

    let Some(meaning) = parse_map_legend_meaning(property.raw) else {
        diagnostics.push(Diagnostic::warning_at(
            "invalid legend mapping",
            Some(path.to_path_buf()),
            line,
        ));
        return;
    };

    if let Some(existing) = map.legend.iter_mut().find(|entry| entry.symbol == symbol) {
        existing.meaning = meaning;
    } else {
        map.legend.push(MapLegendEntry { symbol, meaning });
    }
}

fn parse_map_legend_meaning(value: &str) -> Option<MapLegendMeaning> {
    let value = value.trim();
    if let Some(name) = parse_named_legend_call(value, "spawn") {
        return Some(MapLegendMeaning::Spawn(name));
    }
    if let Some(tile) = parse_tile_legend_call(value) {
        return Some(MapLegendMeaning::Tile(tile));
    }

    if matches!(value, "marker" | "spawn" | "entity") {
        return Some(MapLegendMeaning::Marker);
    }

    if let Some(color) = parse_color(value) {
        return Some(MapLegendMeaning::Color(color));
    }

    if let Some(texture) = parse_string(value) {
        return Some(MapLegendMeaning::Texture(texture));
    }

    parse_terrain_legend_entry(value).map(MapLegendMeaning::Terrain)
}

fn parse_tile_legend_call(value: &str) -> Option<MapTileEntry> {
    let inner = value.strip_prefix("tile")?.trim();
    let inner = inner.strip_prefix('(')?.strip_suffix(')')?.trim();
    let parts = split_top_level_args(inner)?;
    if parts.len() != 2 {
        return None;
    }
    Some(MapTileEntry {
        texture: parse_string(&parts[0])?,
        collision: parse_tile_collision(&parts[1])?,
    })
}

fn parse_tile_collision(value: &str) -> Option<MapTileCollision> {
    match value.trim() {
        "solid" => Some(MapTileCollision::Solid),
        "one_way" => Some(MapTileCollision::OneWay),
        "none" => Some(MapTileCollision::None),
        _ => None,
    }
}

fn parse_named_legend_call(value: &str, name: &str) -> Option<String> {
    let inner = value.strip_prefix(name)?.trim();
    let inner = inner.strip_prefix('(')?.strip_suffix(')')?.trim();
    if inner.is_empty() {
        return None;
    }
    Some(inner.to_string())
}

fn parse_terrain_render_mode(value: &str) -> Option<TerrainRenderMode> {
    match value.trim() {
        "debug" => Some(TerrainRenderMode::Debug),
        "basic" => Some(TerrainRenderMode::Basic),
        "synth" => Some(TerrainRenderMode::Synth),
        _ => None,
    }
}

fn parse_terrain_shoulder_shape(value: &str) -> Option<TerrainShoulderShape> {
    match value.trim() {
        "linear" => Some(TerrainShoulderShape::Linear),
        "bend" => Some(TerrainShoulderShape::Bend),
        _ => None,
    }
}

fn parse_terrain_legend_entry(value: &str) -> Option<MapTerrainEntry> {
    let value = value.trim();
    let (topology, material_raw) = match value.split_once(':') {
        Some((kind, material)) => (parse_terrain_topology(kind.trim())?, material.trim()),
        None => (TerrainTopology::Solid, value),
    };
    let material = parse_string(material_raw).unwrap_or_else(|| material_raw.trim().to_string());
    if material.is_empty() {
        return None;
    }
    let material_stack = material
        .split('>')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if material_stack.is_empty() {
        return None;
    }
    Some(MapTerrainEntry {
        topology,
        material,
        material_stack,
    })
}

fn parse_terrain_topology(value: &str) -> Option<TerrainTopology> {
    match value {
        "solid" => Some(TerrainTopology::Solid),
        "slope_up" => Some(TerrainTopology::SlopeUp),
        "slope_down" => Some(TerrainTopology::SlopeDown),
        _ => None,
    }
}

fn build_terrain_occupancy(
    map: &AsciiMapNode,
    legend: &std::collections::HashMap<char, &MapTerrainEntry>,
    width: usize,
    height: usize,
) -> Vec<Vec<bool>> {
    let mut occupancy = vec![vec![false; width]; height];
    for (row, line) in map.rows.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if legend.contains_key(&ch) {
                occupancy[row][col] = true;
            }
        }
    }
    occupancy
}

fn extract_terrain_regions(
    map: &AsciiMapNode,
    legend: &std::collections::HashMap<char, &MapTerrainEntry>,
    width: usize,
    height: usize,
) -> Vec<TerrainRegion> {
    let mut materials = vec![vec![None::<String>; width]; height];
    for (row, line) in map.rows.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if let Some(terrain) = legend.get(&ch) {
                materials[row][col] = Some(terrain.material.clone());
            }
        }
    }

    let mut visited = vec![vec![false; width]; height];
    let mut regions = Vec::new();
    let mut next_id = 1usize;

    for row in 0..height {
        for col in 0..width {
            let Some(material) = materials[row][col].as_ref() else {
                continue;
            };
            if visited[row][col] {
                continue;
            }

            let mut queue = std::collections::VecDeque::new();
            let mut cells = Vec::new();
            let mut boundary_cells = Vec::new();
            let mut min_row = row;
            let mut max_row = row;
            let mut min_col = col;
            let mut max_col = col;

            visited[row][col] = true;
            queue.push_back((row, col));

            while let Some((current_row, current_col)) = queue.pop_front() {
                cells.push((current_row, current_col));
                if terrain_exposed_sides_for_material(
                    &materials,
                    current_row,
                    current_col,
                    material,
                )
                .any()
                {
                    boundary_cells.push((current_row, current_col));
                }
                min_row = min_row.min(current_row);
                max_row = max_row.max(current_row);
                min_col = min_col.min(current_col);
                max_col = max_col.max(current_col);

                for (next_row, next_col) in
                    orthogonal_neighbors(current_row, current_col, width, height)
                {
                    if visited[next_row][next_col] {
                        continue;
                    }
                    if materials[next_row][next_col].as_deref() != Some(material.as_str()) {
                        continue;
                    }
                    visited[next_row][next_col] = true;
                    queue.push_back((next_row, next_col));
                }
            }

            let boundary_loop = order_boundary_cells(&boundary_cells);
            let max_boundary_distance =
                compute_region_boundary_distances_from_cells(&cells, &boundary_cells)
                    .into_values()
                    .max()
                    .unwrap_or(0);

            regions.push(TerrainRegion {
                id: next_id,
                material: material.clone(),
                min_row,
                min_col,
                max_row,
                max_col,
                cells,
                boundary_cells,
                boundary_loop,
                max_boundary_distance,
            });
            next_id += 1;
        }
    }

    regions
}

fn orthogonal_neighbors(
    row: usize,
    col: usize,
    width: usize,
    height: usize,
) -> impl Iterator<Item = (usize, usize)> {
    let mut neighbors = Vec::with_capacity(4);
    if row > 0 {
        neighbors.push((row - 1, col));
    }
    if row + 1 < height {
        neighbors.push((row + 1, col));
    }
    if col > 0 {
        neighbors.push((row, col - 1));
    }
    if col + 1 < width {
        neighbors.push((row, col + 1));
    }
    neighbors.into_iter()
}

fn terrain_exposed_sides(occupancy: &[Vec<bool>], row: usize, col: usize) -> TerrainExposedSides {
    let filled = |dy: isize, dx: isize| -> bool {
        let y = row as isize + dy;
        let x = col as isize + dx;
        if y < 0 || x < 0 {
            return false;
        }
        occupancy
            .get(y as usize)
            .and_then(|r| r.get(x as usize))
            .copied()
            .unwrap_or(false)
    };

    TerrainExposedSides {
        top: !filled(-1, 0),
        bottom: !filled(1, 0),
        left: !filled(0, -1),
        right: !filled(0, 1),
    }
}

fn terrain_exposed_sides_for_material(
    materials: &[Vec<Option<String>>],
    row: usize,
    col: usize,
    material: &str,
) -> TerrainExposedSides {
    let same = |dy: isize, dx: isize| -> bool {
        let y = row as isize + dy;
        let x = col as isize + dx;
        if y < 0 || x < 0 {
            return false;
        }
        materials
            .get(y as usize)
            .and_then(|r| r.get(x as usize))
            .and_then(|cell| cell.as_deref())
            == Some(material)
    };

    TerrainExposedSides {
        top: !same(-1, 0),
        bottom: !same(1, 0),
        left: !same(0, -1),
        right: !same(0, 1),
    }
}

fn classify_terrain_edge_style(topology: TerrainTopology, shape: TerrainShape) -> TerrainEdgeStyle {
    match topology {
        TerrainTopology::SlopeUp | TerrainTopology::SlopeDown => TerrainEdgeStyle::Diagonal,
        TerrainTopology::Solid => match shape {
            TerrainShape::TopLeftOuter
            | TerrainShape::TopRightOuter
            | TerrainShape::BottomLeftOuter
            | TerrainShape::BottomRightOuter => TerrainEdgeStyle::Round,
            _ => TerrainEdgeStyle::Square,
        },
    }
}

fn classify_terrain_contour(topology: TerrainTopology, shape: TerrainShape) -> TerrainContour {
    match topology {
        TerrainTopology::SlopeUp => TerrainContour::RampUpRight,
        TerrainTopology::SlopeDown => TerrainContour::RampUpLeft,
        TerrainTopology::Solid => match shape {
            TerrainShape::Top | TerrainShape::TopLeftOuter | TerrainShape::TopRightOuter => {
                TerrainContour::FlatTop
            }
            _ => TerrainContour::None,
        },
    }
}

fn classify_terrain_transition_role(
    row: usize,
    col: usize,
    contour: TerrainContour,
    contour_lookup: &std::collections::HashMap<(usize, usize), TerrainContour>,
) -> TerrainTransitionRole {
    match contour {
        TerrainContour::RampUpRight => TerrainTransitionRole::RampUpRight,
        TerrainContour::RampUpLeft => TerrainTransitionRole::RampUpLeft,
        TerrainContour::FlatTop => {
            let join_from_left = row
                .checked_add(1)
                .and_then(|r| col.checked_sub(1).map(|c| (r, c)))
                .and_then(|pos| contour_lookup.get(&pos).copied())
                == Some(TerrainContour::RampUpRight);
            let join_from_right = row
                .checked_add(1)
                .map(|r| (r, col + 1))
                .and_then(|pos| contour_lookup.get(&pos).copied())
                == Some(TerrainContour::RampUpLeft);

            match (join_from_left, join_from_right) {
                (true, true) => TerrainTransitionRole::JoinBoth,
                (true, false) => TerrainTransitionRole::JoinFromLeft,
                (false, true) => TerrainTransitionRole::JoinFromRight,
                (false, false) => TerrainTransitionRole::None,
            }
        }
        _ => TerrainTransitionRole::None,
    }
}

fn compute_transition_strengths(cells: &mut [ClassifiedMapCell]) {
    for cell in cells.iter_mut() {
        cell.transition_strength = match cell.transition_role {
            TerrainTransitionRole::RampUpRight | TerrainTransitionRole::RampUpLeft => 255,
            TerrainTransitionRole::JoinFromLeft
            | TerrainTransitionRole::JoinFromRight
            | TerrainTransitionRole::JoinBoth => 255,
            TerrainTransitionRole::None => 0,
        };
    }
}

fn classify_terrain_normal(exposed: TerrainExposedSides) -> TerrainNormal {
    match (exposed.top, exposed.bottom, exposed.left, exposed.right) {
        (false, false, false, false) => TerrainNormal::None,
        (true, false, false, false) => TerrainNormal::Up,
        (false, true, false, false) => TerrainNormal::Down,
        (false, false, true, false) => TerrainNormal::Left,
        (false, false, false, true) => TerrainNormal::Right,
        (true, false, true, false) => TerrainNormal::UpLeft,
        (true, false, false, true) => TerrainNormal::UpRight,
        (false, true, true, false) => TerrainNormal::DownLeft,
        (false, true, false, true) => TerrainNormal::DownRight,
        (true, true, false, false) => TerrainNormal::Up,
        (false, false, true, true) => TerrainNormal::Left,
        (true, true, true, false) => TerrainNormal::UpLeft,
        (true, true, false, true) => TerrainNormal::UpRight,
        (true, false, true, true) => TerrainNormal::UpLeft,
        (false, true, true, true) => TerrainNormal::DownLeft,
        (true, true, true, true) => TerrainNormal::UpLeft,
    }
}

fn classify_terrain_tangent(normal: TerrainNormal) -> TerrainTangent {
    match normal {
        TerrainNormal::None => TerrainTangent::None,
        TerrainNormal::Up => TerrainTangent::Right,
        TerrainNormal::Down => TerrainTangent::Left,
        TerrainNormal::Left => TerrainTangent::Up,
        TerrainNormal::Right => TerrainTangent::Down,
        TerrainNormal::UpLeft => TerrainTangent::UpRight,
        TerrainNormal::UpRight => TerrainTangent::DownRight,
        TerrainNormal::DownLeft => TerrainTangent::UpLeft,
        TerrainNormal::DownRight => TerrainTangent::DownLeft,
    }
}

fn classify_terrain_depth_band(boundary_distance: u32) -> TerrainDepthBand {
    match boundary_distance {
        0 => TerrainDepthBand::Edge,
        1 => TerrainDepthBand::NearSurface,
        2..=3 => TerrainDepthBand::Interior,
        _ => TerrainDepthBand::DeepInterior,
    }
}

fn terrain_material_for_depth_band<'a>(
    material_stack: &'a [String],
    depth_band: TerrainDepthBand,
    normal: TerrainNormal,
    style: TerrainEdgeStyle,
) -> &'a str {
    let index = match depth_band {
        TerrainDepthBand::Edge => match style {
            TerrainEdgeStyle::Diagonal => 0,
            TerrainEdgeStyle::Square | TerrainEdgeStyle::Round => match normal {
                TerrainNormal::Up | TerrainNormal::UpLeft | TerrainNormal::UpRight => 0,
                _ => 1,
            },
        },
        TerrainDepthBand::NearSurface => 1,
        TerrainDepthBand::Interior => 2,
        TerrainDepthBand::DeepInterior => material_stack.len().saturating_sub(1),
    }
    .min(material_stack.len().saturating_sub(1));
    material_stack.get(index).map(String::as_str).unwrap_or("")
}

fn compute_region_boundary_distances(
    region: &TerrainRegion,
) -> std::collections::HashMap<(usize, usize), u32> {
    compute_region_boundary_distances_from_cells(&region.cells, &region.boundary_cells)
}

fn compute_region_surface_coordinates(
    region: &TerrainRegion,
) -> std::collections::HashMap<(usize, usize), u32> {
    let cell_set: std::collections::HashSet<(usize, usize)> =
        region.cells.iter().copied().collect();
    let mut surface_u = std::collections::HashMap::new();
    let mut queue = std::collections::VecDeque::new();

    for (index, &cell) in region.boundary_loop.iter().enumerate() {
        if surface_u.insert(cell, index as u32).is_none() {
            queue.push_back(cell);
        }
    }

    while let Some((row, col)) = queue.pop_front() {
        let current_u = *surface_u.get(&(row, col)).unwrap_or(&0);
        for next in orthogonal_neighbors_unbounded(row, col) {
            if !cell_set.contains(&next) || surface_u.contains_key(&next) {
                continue;
            }
            surface_u.insert(next, current_u);
            queue.push_back(next);
        }
    }

    for &cell in &region.cells {
        surface_u.entry(cell).or_insert(0);
    }

    surface_u
}

fn compute_region_boundary_distances_from_cells(
    cells: &[(usize, usize)],
    boundary_cells: &[(usize, usize)],
) -> std::collections::HashMap<(usize, usize), u32> {
    let cell_set: std::collections::HashSet<(usize, usize)> = cells.iter().copied().collect();
    let mut distances = std::collections::HashMap::new();
    let mut queue = std::collections::VecDeque::new();

    for &cell in boundary_cells {
        distances.insert(cell, 0);
        queue.push_back(cell);
    }

    while let Some((row, col)) = queue.pop_front() {
        let current_distance = *distances.get(&(row, col)).unwrap_or(&0);
        for next in orthogonal_neighbors_unbounded(row, col) {
            if !cell_set.contains(&next) || distances.contains_key(&next) {
                continue;
            }
            distances.insert(next, current_distance + 1);
            queue.push_back(next);
        }
    }

    for &cell in cells {
        distances.entry(cell).or_insert(0);
    }

    distances
}

fn order_boundary_cells(boundary_cells: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if boundary_cells.is_empty() {
        return Vec::new();
    }

    let boundary_set: std::collections::HashSet<(usize, usize)> =
        boundary_cells.iter().copied().collect();
    let mut remaining = boundary_set.clone();
    let mut ordered = Vec::with_capacity(boundary_cells.len());
    let mut current = *boundary_cells
        .iter()
        .min()
        .expect("boundary cells are not empty");
    let mut previous_direction = (0isize, 1isize);

    while !remaining.is_empty() {
        ordered.push(current);
        remaining.remove(&current);

        let mut neighbors = boundary_neighbors(current, &remaining);
        if neighbors.is_empty() {
            if let Some(next) = remaining
                .iter()
                .min_by_key(|&&(row, col)| row.abs_diff(current.0) + col.abs_diff(current.1))
                .copied()
            {
                current = next;
                previous_direction = (0, 1);
                continue;
            }
            break;
        }

        neighbors.sort_by_key(|&(next_row, next_col)| {
            let direction = (
                next_row as isize - current.0 as isize,
                next_col as isize - current.1 as isize,
            );
            (
                boundary_turn_rank(previous_direction, direction),
                next_row.abs_diff(current.0) + next_col.abs_diff(current.1),
                next_row,
                next_col,
            )
        });

        let next = neighbors[0];
        previous_direction = (
            next.0 as isize - current.0 as isize,
            next.1 as isize - current.1 as isize,
        );
        current = next;
    }

    ordered
}

fn orthogonal_neighbors_unbounded(row: usize, col: usize) -> impl Iterator<Item = (usize, usize)> {
    let mut neighbors = Vec::with_capacity(4);
    if row > 0 {
        neighbors.push((row - 1, col));
    }
    neighbors.push((row + 1, col));
    if col > 0 {
        neighbors.push((row, col - 1));
    }
    neighbors.push((row, col + 1));
    neighbors.into_iter()
}

fn boundary_neighbors(
    current: (usize, usize),
    boundary_cells: &std::collections::HashSet<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let (row, col) = current;
    let mut neighbors = Vec::with_capacity(8);
    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            if dy == 0 && dx == 0 {
                continue;
            }
            let next_row = row as isize + dy;
            let next_col = col as isize + dx;
            if next_row < 0 || next_col < 0 {
                continue;
            }
            let next = (next_row as usize, next_col as usize);
            if boundary_cells.contains(&next) {
                neighbors.push(next);
            }
        }
    }
    neighbors
}

fn boundary_turn_rank(previous_direction: (isize, isize), direction: (isize, isize)) -> i32 {
    let directions = [
        (-1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
    ];
    let previous_index = directions
        .iter()
        .position(|&candidate| candidate == previous_direction)
        .unwrap_or(2) as i32;
    let current_index = directions
        .iter()
        .position(|&candidate| candidate == direction)
        .unwrap_or(2) as i32;
    (current_index - previous_index).rem_euclid(8)
}

fn classify_terrain_shape(occupancy: &[Vec<bool>], row: usize, col: usize) -> TerrainShape {
    if occupancy.is_empty() || occupancy[row][col] == false {
        return TerrainShape::Empty;
    }

    let filled = |dy: isize, dx: isize| -> bool {
        let y = row as isize + dy;
        let x = col as isize + dx;
        if y < 0 || x < 0 {
            return false;
        }
        occupancy
            .get(y as usize)
            .and_then(|r| r.get(x as usize))
            .copied()
            .unwrap_or(false)
    };

    let up = filled(-1, 0);
    let down = filled(1, 0);
    let left = filled(0, -1);
    let right = filled(0, 1);

    let top_open = !up;
    let bottom_open = !down;
    let left_open = !left;
    let right_open = !right;

    if top_open && left_open && !bottom_open && !right_open {
        return TerrainShape::TopLeftOuter;
    }
    if top_open && right_open && !bottom_open && !left_open {
        return TerrainShape::TopRightOuter;
    }
    if bottom_open && left_open && !top_open && !right_open {
        return TerrainShape::BottomLeftOuter;
    }
    if bottom_open && right_open && !top_open && !left_open {
        return TerrainShape::BottomRightOuter;
    }

    let up_left = filled(-1, -1);
    let up_right = filled(-1, 1);
    let down_left = filled(1, -1);
    let down_right = filled(1, 1);

    if !top_open && !left_open && !up_left {
        return TerrainShape::TopLeftInner;
    }
    if !top_open && !right_open && !up_right {
        return TerrainShape::TopRightInner;
    }
    if !bottom_open && !left_open && !down_left {
        return TerrainShape::BottomLeftInner;
    }
    if !bottom_open && !right_open && !down_right {
        return TerrainShape::BottomRightInner;
    }

    match (top_open, bottom_open, left_open, right_open) {
        (false, false, false, false) => TerrainShape::Interior,
        (true, false, false, false) => TerrainShape::Top,
        (false, true, false, false) => TerrainShape::Bottom,
        (false, false, true, false) => TerrainShape::Left,
        (false, false, false, true) => TerrainShape::Right,
        _ => TerrainShape::Isolated,
    }
}

fn terrain_shape_debug_color(shape: TerrainShape) -> [f32; 4] {
    match shape {
        TerrainShape::Empty => [0.0, 0.0, 0.0, 0.0],
        TerrainShape::Isolated => [0.95, 0.32, 0.32, 1.0],
        TerrainShape::Interior => [0.14, 0.26, 0.74, 1.0],
        TerrainShape::Top => [0.35, 0.88, 0.42, 1.0],
        TerrainShape::Bottom => [0.72, 0.33, 0.84, 1.0],
        TerrainShape::Left => [0.98, 0.72, 0.26, 1.0],
        TerrainShape::Right => [0.98, 0.56, 0.18, 1.0],
        TerrainShape::TopLeftOuter => [0.26, 0.92, 0.92, 1.0],
        TerrainShape::TopRightOuter => [0.19, 0.82, 0.98, 1.0],
        TerrainShape::BottomLeftOuter => [0.88, 0.41, 0.81, 1.0],
        TerrainShape::BottomRightOuter => [0.75, 0.33, 0.95, 1.0],
        TerrainShape::TopLeftInner => [0.62, 0.94, 0.62, 1.0],
        TerrainShape::TopRightInner => [0.55, 0.88, 0.55, 1.0],
        TerrainShape::BottomLeftInner => [0.93, 0.58, 0.58, 1.0],
        TerrainShape::BottomRightInner => [0.86, 0.49, 0.49, 1.0],
    }
}

fn normalize_ascii_map(mut map: AsciiMapNode) -> AsciiMapNode {
    let min_indent = map
        .rows
        .iter()
        .filter(|row| !row.trim().is_empty())
        .map(|row| {
            row.chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .count()
        })
        .min()
        .unwrap_or(0);

    map.rows = map
        .rows
        .into_iter()
        .map(|row| {
            let mut skip = min_indent;
            let mut start = 0usize;
            for (index, ch) in row.char_indices() {
                if skip == 0 {
                    start = index;
                    break;
                }
                if ch == ' ' || ch == '\t' {
                    skip -= 1;
                    start = index + ch.len_utf8();
                } else {
                    start = index;
                    break;
                }
            }
            row[start..].trim_end().to_string()
        })
        .collect();

    map
}

fn parse_color(value: &str) -> Option<[f32; 4]> {
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    parse_f32_tuple::<4>(value)
}

fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ])
        }
        _ => None,
    }
}

fn parse_f32_tuple<const N: usize>(value: &str) -> Option<[f32; N]> {
    let value = value.strip_prefix('(')?.strip_suffix(')')?;
    let mut out = [0.0; N];
    let mut parts = value.split(',').map(str::trim);
    for item in &mut out {
        *item = parts.next()?.parse().ok()?;
    }
    if parts.next().is_some() {
        return None;
    }
    Some(out)
}

fn parse_call_vec2(argument: &str) -> Option<[f32; 2]> {
    let inner = argument.trim().strip_suffix(')')?.trim();
    let wrapped = format!("({inner})");
    parse_f32_tuple::<2>(&wrapped)
}

fn parse_call_color(argument: &str) -> Option<[f32; 4]> {
    let inner = argument.trim().strip_suffix(')')?.trim();
    parse_color(inner)
}

fn split_top_level_args(inner: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut depth = 0i32;

    for ch in inner.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' if !in_string => {
                depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                current.push(ch);
            }
            ',' if !in_string && depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if in_string || depth != 0 {
        return None;
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    Some(parts)
}

fn parse_targeted_call_vec2(argument: &str) -> Option<(String, [f32; 2])> {
    let parts = split_top_level_args(argument.trim().strip_suffix(')')?.trim())?;
    if parts.len() != 3 {
        return None;
    }

    let target = parse_string(&parts[0])?;
    let wrapped = format!("({}, {})", parts[1], parts[2]);
    let delta = parse_f32_tuple::<2>(&wrapped)?;
    Some((target, delta))
}

fn parse_targeted_call_color(argument: &str) -> Option<(String, [f32; 4])> {
    let parts = split_top_level_args(argument.trim().strip_suffix(')')?.trim())?;
    if parts.len() != 2 {
        return None;
    }

    let target = parse_string(&parts[0])?;
    let color = parse_color(&parts[1])?;
    Some((target, color))
}

fn parse_call_range(argument: &str) -> Option<[f32; 2]> {
    let inner = argument.trim().strip_suffix(')')?.trim();
    let wrapped = format!("({inner})");
    parse_f32_tuple::<2>(&wrapped)
}

fn parse_call_target(argument: &str) -> Option<String> {
    let parts = split_top_level_args(argument.trim().strip_suffix(')')?.trim())?;
    if parts.len() != 1 {
        return None;
    }
    parse_string(&parts[0])
}

fn count_ops(ops: &[BytecodeOp]) -> usize {
    ops.iter()
        .map(|op| match &op.op {
            OpCode::If(_, body, else_body) => 1 + count_ops(body) + count_ops(else_body),
            _ => 1,
        })
        .sum()
}

fn compile_scripts(
    scripts: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<BytecodeScript> {
    scripts
        .iter()
        .map(|script| compile_script(script, diagnostics))
        .collect()
}

fn compile_script(script: &SourceFile, diagnostics: &mut Vec<Diagnostic>) -> BytecodeScript {
    let mut lines = Vec::new();
    for (index, raw) in script.contents.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed == "} else {" {
            lines.push((line_no, "}".to_string()));
            lines.push((line_no, "else {".to_string()));
        } else if let Some(rest) = trimmed.strip_prefix("} else if ") {
            lines.push((line_no, "}".to_string()));
            lines.push((line_no, format!("else if {rest}")));
        } else {
            lines.push((line_no, line.to_string()));
        }
    }
    let mut handlers = Vec::new();
    let mut functions = Vec::new();
    let mut state = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let (line_no, raw_line) = &lines[index];
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") {
            index += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("on ") {
            let Some((event, params)) = parse_callable_signature(rest) else {
                diagnostics.push(Diagnostic::warning_at(
                    "invalid event handler signature",
                    Some(script.relative_path.clone()),
                    *line_no,
                ));
                index += 1;
                continue;
            };
            index += 1;
            let ops = compile_script_block(&lines, &mut index, script, diagnostics);
            handlers.push(BytecodeHandler { event, params, ops });
            continue;
        }

        if let Some(rest) = line.strip_prefix("fn ") {
            let Some((name, params)) = parse_callable_signature(rest) else {
                diagnostics.push(Diagnostic::warning_at(
                    "invalid function signature",
                    Some(script.relative_path.clone()),
                    *line_no,
                ));
                index += 1;
                continue;
            };
            index += 1;
            let ops = compile_script_block(&lines, &mut index, script, diagnostics);
            functions.push(BytecodeFunction { name, params, ops });
            continue;
        }

        if let Some(rest) = line.strip_prefix("state ") {
            let Some((name, init)) = parse_state_declaration(rest) else {
                diagnostics.push(Diagnostic::warning_at(
                    "invalid state declaration",
                    Some(script.relative_path.clone()),
                    *line_no,
                ));
                index += 1;
                continue;
            };
            state.push(BytecodeState {
                name,
                init,
                line: *line_no,
            });
            index += 1;
            continue;
        }

        diagnostics.push(Diagnostic::warning_at(
            "script statement is outside of an event block",
            Some(script.relative_path.clone()),
            *line_no,
        ));
        index += 1;
    }

    if handlers.is_empty() {
        diagnostics.push(Diagnostic::warning(
            "script does not define any event handlers",
            Some(script.relative_path.clone()),
        ));
    }

    BytecodeScript {
        path: script.relative_path.clone(),
        state,
        handlers,
        functions,
    }
}

fn compile_script_block(
    lines: &[(usize, String)],
    index: &mut usize,
    script: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<BytecodeOp> {
    let mut ops = Vec::new();

    while *index < lines.len() {
        let (line_no, raw_line) = &lines[*index];
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with("//") {
            *index += 1;
            continue;
        }

        if line == "}" {
            *index += 1;
            break;
        }

        if let Some(condition_text) = line
            .strip_prefix("if ")
            .and_then(|rest| rest.strip_suffix('{'))
        {
            *index += 1;
            if let Some(condition) = parse_condition(condition_text.trim()) {
                let body = compile_script_block(lines, index, script, diagnostics);
                let else_body = compile_else_branch(lines, index, script, diagnostics);
                ops.push(BytecodeOp {
                    line: *line_no,
                    op: OpCode::If(condition, body, else_body),
                });
            } else {
                diagnostics.push(Diagnostic::warning_at(
                    "invalid if condition",
                    Some(script.relative_path.clone()),
                    *line_no,
                ));
                let _ = compile_script_block(lines, index, script, diagnostics);
            }
            continue;
        }

        if let Some(op) = compile_statement(line) {
            ops.push(BytecodeOp { line: *line_no, op });
        } else {
            diagnostics.push(Diagnostic::warning_at(
                "unsupported script statement",
                Some(script.relative_path.clone()),
                *line_no,
            ));
        }
        *index += 1;
    }

    ops
}

fn compile_else_branch(
    lines: &[(usize, String)],
    index: &mut usize,
    script: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<BytecodeOp> {
    if *index >= lines.len() {
        return Vec::new();
    }

    let (line_no, raw_line) = &lines[*index];
    let line = raw_line.trim();

    if line == "else {" {
        *index += 1;
        return compile_script_block(lines, index, script, diagnostics);
    }

    if let Some(condition_text) = line
        .strip_prefix("else if ")
        .and_then(|rest| rest.strip_suffix('{'))
    {
        *index += 1;
        if let Some(condition) = parse_condition(condition_text.trim()) {
            let body = compile_script_block(lines, index, script, diagnostics);
            let else_body = compile_else_branch(lines, index, script, diagnostics);
            return vec![BytecodeOp {
                line: *line_no,
                op: OpCode::If(condition, body, else_body),
            }];
        }

        diagnostics.push(Diagnostic::warning_at(
            "invalid else-if condition",
            Some(script.relative_path.clone()),
            *line_no,
        ));
        let _ = compile_script_block(lines, index, script, diagnostics);
    }

    Vec::new()
}

fn compile_statement(line: &str) -> Option<OpCode> {
    if let Some(rest) = line.strip_prefix("spawn(") {
        let (template, name, x, y) = parse_spawn_statement(rest)?;
        return Some(OpCode::Spawn(template, name, x, y));
    }

    if let Some(rest) = line.strip_prefix("destroy(") {
        return parse_destroy_statement(rest);
    }

    if let Some(rest) = line.strip_prefix("return ") {
        return Some(OpCode::Return(parse_expr(rest.trim())?));
    }

    if let Some(rest) = line.strip_prefix("call ") {
        let (name, args) = parse_call_statement(rest)?;
        return Some(OpCode::Call(name, args));
    }

    if let Some((name, args)) = parse_call_statement(line) {
        return Some(OpCode::Call(name, args));
    }

    if let Some(rest) = line.strip_prefix("let ") {
        let (name, right) = rest.split_once('=')?;
        let name = name.trim();
        let right = right.trim();
        if name == "_" {
            return Some(compile_op(line));
        }
        if name.is_empty() {
            return None;
        }
        return Some(OpCode::Let(name.to_string(), parse_expr(right)?));
    }

    if let Some((left, right)) = line.split_once('=') {
        let left = left.trim();
        let right = right.trim();
        if is_state_identifier(left) {
            return Some(OpCode::StateSet(left.to_string(), parse_expr(right)?));
        }
    }

    if let Some((left, right)) = line.split_once('=') {
        let left = left.trim();
        let right = right.trim();
        if let (Some(target), Some(expr)) = (parse_script_target(left), parse_expr(right)) {
            return Some(OpCode::Assign(target, expr));
        }
    }

    Some(compile_op(line))
}

fn parse_state_declaration(value: &str) -> Option<(String, Expr)> {
    let (name, init) = value.split_once('=')?;
    let name = name.trim();
    if !is_state_identifier(name) {
        return None;
    }
    Some((name.to_string(), parse_expr(init.trim())?))
}

fn is_state_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn parse_spawn_statement(value: &str) -> Option<(String, Option<String>, Expr, Expr)> {
    let inner = value.trim().strip_suffix(')')?.trim();
    let parts = split_top_level_args(inner)?;
    match parts.len() {
        3 => Some((
            parse_string(parts[0].trim())?,
            None,
            parse_expr(parts[1].trim())?,
            parse_expr(parts[2].trim())?,
        )),
        4 => Some((
            parse_string(parts[0].trim())?,
            Some(parse_string(parts[1].trim())?),
            parse_expr(parts[2].trim())?,
            parse_expr(parts[3].trim())?,
        )),
        _ => None,
    }
}

fn parse_destroy_statement(value: &str) -> Option<OpCode> {
    let inner = value.trim().strip_suffix(')')?.trim();
    if inner == "self" {
        return Some(OpCode::Destroy(DestroyTarget::SelfEntity));
    }
    if let Some(name) = parse_string(inner) {
        return Some(OpCode::Destroy(DestroyTarget::Named(name)));
    }
    Some(OpCode::DestroyExpr(parse_expr(inner)?))
}

fn parse_callable_signature(value: &str) -> Option<(String, Vec<String>)> {
    let value = value.trim();
    let value = value.strip_suffix('{')?.trim();
    let (name, params) = value.split_once('(')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let params = params.strip_suffix(')')?.trim();
    let mut out = Vec::new();
    if !params.is_empty() {
        for param in split_top_level_args(params)? {
            let param = param.trim();
            if param.is_empty() {
                return None;
            }
            out.push(param.to_string());
        }
    }
    Some((name.to_string(), out))
}

fn parse_call_statement(value: &str) -> Option<(String, Vec<Expr>)> {
    let value = value.trim();
    let (name, args) = value.split_once('(')?;
    let name = name.trim();
    if !is_state_identifier(name) {
        return None;
    }
    let args = args.strip_suffix(')')?.trim();
    let mut out = Vec::new();
    if !args.is_empty() {
        for arg in split_top_level_args(args)? {
            out.push(parse_expr(arg.trim())?);
        }
    }
    Some((name.to_string(), out))
}

fn parse_script_target(value: &str) -> Option<ScriptTarget> {
    let (entity, property) = value.split_once('.')?;
    let property = match property.trim() {
        "x" => ScriptProperty::X,
        "y" => ScriptProperty::Y,
        "width" => ScriptProperty::Width,
        "height" => ScriptProperty::Height,
        "pos" => ScriptProperty::Pos,
        "size" => ScriptProperty::Size,
        "rotation" => ScriptProperty::Rotation,
        "color" => ScriptProperty::Color,
        "texture" => ScriptProperty::Texture,
        "animation" => ScriptProperty::Animation,
        "flip_x" => ScriptProperty::FlipX,
        "flip_y" => ScriptProperty::FlipY,
        "vx" => ScriptProperty::Vx,
        "vy" => ScriptProperty::Vy,
        "move_x" => ScriptProperty::MoveX,
        "jump" => ScriptProperty::Jump,
        "grounded" => ScriptProperty::Grounded,
        "text" => ScriptProperty::Text,
        other if is_state_identifier(other) => ScriptProperty::State(other.to_string()),
        _ => return None,
    };

    let entity = entity.trim();
    if entity == "self" {
        Some(ScriptTarget::SelfEntity(property))
    } else {
        Some(ScriptTarget::NamedEntity(entity.to_string(), property))
    }
}

fn parse_condition(value: &str) -> Option<Condition> {
    let value = strip_outer_condition_parens(value.trim());

    if let Some((left, right)) = split_top_level_condition(value, "||") {
        return Some(Condition::Or(
            Box::new(parse_condition(left)?),
            Box::new(parse_condition(right)?),
        ));
    }

    if let Some((left, right)) = split_top_level_condition(value, "&&") {
        return Some(Condition::And(
            Box::new(parse_condition(left)?),
            Box::new(parse_condition(right)?),
        ));
    }

    if let Some(rest) = value.strip_prefix('!') {
        return Some(Condition::Not(Box::new(parse_condition(rest.trim())?)));
    }

    for (symbol, op) in [
        ("<=", CompareOp::LessEqual),
        (">=", CompareOp::GreaterEqual),
        ("==", CompareOp::Equal),
        ("!=", CompareOp::NotEqual),
        ("<", CompareOp::Less),
        (">", CompareOp::Greater),
    ] {
        if let Some((left, right)) = split_top_level_condition(value, symbol) {
            return Some(Condition::Compare {
                left: parse_expr(left)?,
                op,
                right: parse_expr(right)?,
            });
        }
    }

    Some(Condition::Compare {
        left: parse_expr(value)?,
        op: CompareOp::NotEqual,
        right: Expr::Number(0.0),
    })
}

fn strip_outer_condition_parens(mut value: &str) -> &str {
    loop {
        let trimmed = value.trim();
        if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
            return trimmed;
        }
        let mut depth = 0i32;
        let mut wraps = true;
        for (index, ch) in trimmed.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && index != trimmed.len() - 1 {
                        wraps = false;
                        break;
                    }
                }
                _ => {}
            }
            if depth < 0 {
                wraps = false;
                break;
            }
        }
        if wraps {
            value = &trimmed[1..trimmed.len() - 1];
        } else {
            return trimmed;
        }
    }
}

fn split_top_level_condition<'a>(value: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0i32;
    let mut in_string = false;
    for (index, ch) in value.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => depth -= 1,
            _ => {}
        }
        if depth == 0 && !in_string && value[index..].starts_with(needle) {
            let left = value[..index].trim();
            let right = value[index + needle.len()..].trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, right));
            }
        }
    }
    None
}

#[derive(Clone)]
enum ExprToken {
    Number(f32),
    Dt,
    String(String),
    Variable(String),
    Target(ScriptTarget),
    Color([f32; 4]),
    Clamp,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
}

fn parse_expr(value: &str) -> Option<Expr> {
    let tokens = tokenize_expr(value)?;
    let mut index = 0usize;
    let expr = parse_expr_bp(&tokens, &mut index, 0)?;
    if index != tokens.len() {
        return None;
    }
    Some(expr)
}

fn tokenize_expr(value: &str) -> Option<Vec<ExprToken>> {
    let mut chars = value.char_indices().peekable();
    let mut tokens = Vec::new();

    while let Some((index, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        match ch {
            '+' => tokens.push(ExprToken::Plus),
            '-' => {
                let unary_context = matches!(
                    tokens.last(),
                    None | Some(
                        ExprToken::Plus
                            | ExprToken::Minus
                            | ExprToken::Star
                            | ExprToken::Slash
                            | ExprToken::LParen
                            | ExprToken::Comma
                    )
                );
                let next_is_number = chars
                    .peek()
                    .map(|(_, next_ch)| next_ch.is_ascii_digit() || *next_ch == '.')
                    .unwrap_or(false);
                if unary_context && next_is_number {
                    let start = index;
                    let mut end = index + ch.len_utf8();
                    while let Some((next_index, next_ch)) = chars.peek().copied() {
                        if next_ch.is_ascii_digit() || next_ch == '.' {
                            end = next_index + next_ch.len_utf8();
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(ExprToken::Number(value[start..end].parse().ok()?));
                } else {
                    tokens.push(ExprToken::Minus);
                }
            }
            '*' => tokens.push(ExprToken::Star),
            '/' => tokens.push(ExprToken::Slash),
            '(' => tokens.push(ExprToken::LParen),
            ')' => tokens.push(ExprToken::RParen),
            ',' => tokens.push(ExprToken::Comma),
            '"' => {
                let start = index + ch.len_utf8();
                let mut end = start;
                let mut closed = false;
                while let Some((next_index, next_ch)) = chars.next() {
                    if next_ch == '"' {
                        end = next_index;
                        closed = true;
                        break;
                    }
                }
                if !closed {
                    return None;
                }
                tokens.push(ExprToken::String(value[start..end].to_string()));
            }
            '#' => {
                let start = index;
                let mut end = index + ch.len_utf8();
                while let Some((next_index, next_ch)) = chars.peek().copied() {
                    if next_ch.is_ascii_hexdigit() {
                        end = next_index + next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(ExprToken::Color(parse_color(&value[start..end])?));
            }
            '0'..='9' => {
                let start = index;
                let mut end = index + ch.len_utf8();
                while let Some((next_index, next_ch)) = chars.peek().copied() {
                    if next_ch.is_ascii_digit() || next_ch == '.' {
                        end = next_index + next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(ExprToken::Number(value[start..end].parse().ok()?));
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                let start = index;
                let mut end = index + ch.len_utf8();
                while let Some((next_index, next_ch)) = chars.peek().copied() {
                    if next_ch.is_ascii_alphanumeric() || next_ch == '_' || next_ch == '.' {
                        end = next_index + next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }
                let ident = &value[start..end];
                match ident {
                    "dt" => tokens.push(ExprToken::Dt),
                    "clamp" => tokens.push(ExprToken::Clamp),
                    _ => match parse_script_target(ident) {
                        Some(target) => tokens.push(ExprToken::Target(target)),
                        None => tokens.push(ExprToken::Variable(ident.to_string())),
                    },
                }
            }
            _ => return None,
        }
    }

    Some(tokens)
}

fn parse_expr_bp(tokens: &[ExprToken], index: &mut usize, min_bp: u8) -> Option<Expr> {
    let mut lhs = match tokens.get(*index)? {
        ExprToken::Number(value) => {
            *index += 1;
            Expr::Number(*value)
        }
        ExprToken::Minus => {
            *index += 1;
            let rhs = parse_expr_bp(tokens, index, 5)?;
            Expr::Binary(Box::new(Expr::Number(0.0)), BinaryOp::Sub, Box::new(rhs))
        }
        ExprToken::Dt => {
            *index += 1;
            Expr::Dt
        }
        ExprToken::String(value) => {
            *index += 1;
            Expr::String(value.clone())
        }
        ExprToken::Variable(name) => {
            *index += 1;
            Expr::Variable(name.clone())
        }
        ExprToken::Target(target) => {
            *index += 1;
            Expr::Target(target.clone())
        }
        ExprToken::Color(color) => {
            *index += 1;
            Expr::Color(*color)
        }
        ExprToken::LParen => {
            *index += 1;
            let expr = parse_expr_bp(tokens, index, 0)?;
            match tokens.get(*index)? {
                ExprToken::RParen => {
                    *index += 1;
                    expr
                }
                _ => return None,
            }
        }
        ExprToken::Clamp => {
            *index += 1;
            match tokens.get(*index)? {
                ExprToken::LParen => *index += 1,
                _ => return None,
            }
            let value = parse_expr_bp(tokens, index, 0)?;
            match tokens.get(*index)? {
                ExprToken::Comma => *index += 1,
                _ => return None,
            }
            let min = parse_expr_bp(tokens, index, 0)?;
            match tokens.get(*index)? {
                ExprToken::Comma => *index += 1,
                _ => return None,
            }
            let max = parse_expr_bp(tokens, index, 0)?;
            match tokens.get(*index)? {
                ExprToken::RParen => *index += 1,
                _ => return None,
            }
            Expr::Clamp(Box::new(value), Box::new(min), Box::new(max))
        }
        _ => return None,
    };

    loop {
        if matches!(tokens.get(*index), Some(ExprToken::LParen)) {
            let function_name = match &lhs {
                Expr::Variable(name) => name.clone(),
                _ => break,
            };
            *index += 1;
            let mut args = Vec::new();
            if !matches!(tokens.get(*index), Some(ExprToken::RParen)) {
                loop {
                    args.push(parse_expr_bp(tokens, index, 0)?);
                    match tokens.get(*index)? {
                        ExprToken::Comma => *index += 1,
                        ExprToken::RParen => break,
                        _ => return None,
                    }
                }
            }
            match tokens.get(*index)? {
                ExprToken::RParen => *index += 1,
                _ => return None,
            }
            lhs = Expr::Call(function_name, args);
            continue;
        }

        let (l_bp, r_bp, op) = match tokens.get(*index) {
            Some(ExprToken::Plus) => (1, 2, BinaryOp::Add),
            Some(ExprToken::Minus) => (1, 2, BinaryOp::Sub),
            Some(ExprToken::Star) => (3, 4, BinaryOp::Mul),
            Some(ExprToken::Slash) => (3, 4, BinaryOp::Div),
            _ => break,
        };

        if l_bp < min_bp {
            break;
        }
        *index += 1;
        let rhs = parse_expr_bp(tokens, index, r_bp)?;
        lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
    }

    Some(lhs)
}

fn compile_op(line: &str) -> OpCode {
    if let Some(argument) = line.strip_prefix("log(") {
        let message = argument
            .trim_end_matches(')')
            .trim()
            .trim_matches('"')
            .to_string();
        return OpCode::Log(message);
    }

    if let Some(argument) = line.strip_prefix("let _ =") {
        return OpCode::IgnoreValue(argument.trim().to_string());
    }

    if let Some(argument) = line.strip_prefix("move_by_dt(") {
        if let Some((target, delta)) = parse_targeted_call_vec2(argument) {
            return OpCode::MoveByDtTarget(target, delta);
        }
        if let Some(delta) = parse_call_vec2(argument) {
            return OpCode::MoveByDt(delta);
        }
    }

    if let Some(argument) = line.strip_prefix("move_by(") {
        if let Some((target, delta)) = parse_targeted_call_vec2(argument) {
            return OpCode::MoveByTarget(target, delta);
        }
        if let Some(delta) = parse_call_vec2(argument) {
            return OpCode::MoveBy(delta);
        }
    }

    if let Some(argument) = line.strip_prefix("set_pos(") {
        if let Some((target, pos)) = parse_targeted_call_vec2(argument) {
            return OpCode::SetPosTarget(target, pos);
        }
        if let Some(pos) = parse_call_vec2(argument) {
            return OpCode::SetPos(pos);
        }
    }

    if let Some(argument) = line.strip_prefix("set_color(") {
        if let Some((target, color)) = parse_targeted_call_color(argument) {
            return OpCode::SetColorTarget(target, color);
        }
        if let Some(color) = parse_call_color(argument) {
            return OpCode::SetColor(color);
        }
    }

    if let Some(argument) = line.strip_prefix("copy_pos(") {
        if let Some(target) = parse_call_target(argument) {
            return OpCode::CopyPos(target);
        }
    }

    if let Some(argument) = line.strip_prefix("clamp_x(") {
        if let Some(range) = parse_call_range(argument) {
            return OpCode::ClampX(range);
        }
    }

    if let Some(argument) = line.strip_prefix("clamp_y(") {
        if let Some(range) = parse_call_range(argument) {
            return OpCode::ClampY(range);
        }
    }

    OpCode::Raw(line.to_string())
}

fn max_system_time(a: Option<SystemTime>, b: Option<SystemTime>) -> Option<SystemTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_start_scene() -> String {
    "Main".to_string()
}

fn default_window_width() -> u32 {
    1280
}

fn default_window_height() -> u32 {
    720
}

fn default_window_scale() -> f32 {
    1.0
}

fn default_scene(name: &str) -> String {
    format!(
        "scene Main {{\n    meta {{\n        title = \"{}\"\n    }}\n\n    camera MainCamera {{\n        pos = (320, 220)\n        zoom = 1.0\n        background = (0.07, 0.08, 0.12, 1.0)\n    }}\n\n    map Terrain {{\n        origin = (80, 288)\n        cell = (48, 48)\n\n        legend {{\n            x = marker\n            # = #ffb224\n            - = #3d7cff\n        }}\n\n        ascii {{\n            x      \n            ###----\n        }}\n    }}\n\n    rect Hero {{\n        layer = 0\n        z = 10\n        pos = (96, 96)\n        size = (240, 140)\n        color = (0.94, 0.42, 0.18, 1.0)\n        script = \"main.rpu\"\n    }}\n\n    sprite Mascot {{\n        layer = 1\n        z = 20\n        symbol = x\n        size = (192, 192)\n        color = (0.94, 0.92, 0.26, 1.0)\n    }}\n\n    rect Accent {{\n        layer = 0\n        z = 30\n        pos = (380, 180)\n        size = (180, 220)\n        color = (0.14, 0.72, 0.88, 1.0)\n    }}\n}}\n",
        name
    )
}

fn default_script() -> &'static str {
    "on ready() {\n    log(\"RPU project booted\")\n    set_color(#ffd447)\n}\n\non update(dt) {\n    let _ = dt\n    move_by_dt(16.0, 0.0)\n}\n"
}

fn default_gitignore() -> &'static str {
    "/build/\n/.rpu/\n"
}

#[cfg(test)]
mod tests;
