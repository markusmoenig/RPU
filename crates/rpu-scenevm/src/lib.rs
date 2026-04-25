use anyhow::{Context as AnyhowContext, Result, anyhow};
#[cfg(target_arch = "wasm32")]
use base64::Engine;
use bytemuck::{Pod, Zeroable};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use rpu_core::{Anchor, TextAlign};
use std::collections::{HashMap, HashSet};
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "tvos", target_os = "ios")
))]
use std::ffi::c_void;
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
use std::ffi::CString;
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
use std::{fs::File, io::Write, path::PathBuf};
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlAudioElement, HtmlCanvasElement, KeyboardEvent, MouseEvent, WheelEvent};
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_ASSETS: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
}

#[derive(Clone)]
struct GeneratedTextureAsset {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

static GENERATED_TEXTURES: OnceLock<Mutex<HashMap<String, GeneratedTextureAsset>>> =
    OnceLock::new();

fn generated_textures() -> &'static Mutex<HashMap<String, GeneratedTextureAsset>> {
    GENERATED_TEXTURES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(target_arch = "wasm32")]
pub fn register_web_asset(path: &str, bytes: &[u8]) {
    WEB_ASSETS.with(|assets| {
        assets
            .borrow_mut()
            .insert(path.trim_start_matches('/').to_string(), bytes.to_vec());
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_web_asset(_path: &str, _bytes: &[u8]) {}

pub fn register_generated_rgba_texture(path: &str, width: u32, height: u32, rgba: &[u8]) {
    if width == 0 || height == 0 {
        return;
    }
    let expected = width.saturating_mul(height).saturating_mul(4) as usize;
    if rgba.len() != expected {
        return;
    }
    if let Ok(mut textures) = generated_textures().lock() {
        textures.insert(
            path.to_string(),
            GeneratedTextureAsset {
                width,
                height,
                rgba: rgba.to_vec(),
            },
        );
    }
}

pub trait RpuSceneApp {
    fn initial_window_size(&self) -> Option<(u32, u32)> {
        Some((1280, 720))
    }

    fn window_title(&self) -> Option<String> {
        Some("RPU".to_string())
    }

    fn target_fps(&self) -> Option<f32> {
        Some(60.0)
    }

    fn init(&mut self, _ctx: &mut RuntimeContext) {}

    fn update(&mut self, _ctx: &mut RuntimeContext) {}

    fn render(&mut self, _ctx: &mut RuntimeContext, _frame: &mut SceneFrame) {}

    fn needs_update(&mut self, _ctx: &RuntimeContext) -> bool {
        true
    }

    fn resize(&mut self, _ctx: &mut RuntimeContext, _size: (u32, u32)) {}

    fn set_scale(&mut self, _scale: f32) {}

    fn set_native_mode(&mut self, _is_native: bool) {}

    fn mouse_down(&mut self, _ctx: &mut RuntimeContext, _x: f32, _y: f32) {}

    fn mouse_up(&mut self, _ctx: &mut RuntimeContext, _x: f32, _y: f32) {}

    fn mouse_move(&mut self, _ctx: &mut RuntimeContext, _x: f32, _y: f32) {}

    fn scroll(&mut self, _ctx: &mut RuntimeContext, _dx: f32, _dy: f32) {}
}

pub struct RuntimeContext {
    window_size: (u32, u32),
    scale_factor: f32,
    pressed_keys: HashSet<String>,
    audio: AudioState,
}

impl RuntimeContext {
    pub fn new(window_size: (u32, u32), scale_factor: f32) -> Self {
        Self {
            window_size,
            scale_factor,
            pressed_keys: HashSet::new(),
            audio: AudioState::new(),
        }
    }

    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn is_key_pressed(&self, key: &str) -> bool {
        self.pressed_keys.contains(&normalize_key_name(key))
    }

    pub fn input_left(&self) -> bool {
        self.is_key_pressed("ArrowLeft") || self.is_key_pressed("A")
    }

    pub fn input_right(&self) -> bool {
        self.is_key_pressed("ArrowRight") || self.is_key_pressed("D")
    }

    pub fn input_up(&self) -> bool {
        self.is_key_pressed("ArrowUp") || self.is_key_pressed("W")
    }

    pub fn input_down(&self) -> bool {
        self.is_key_pressed("ArrowDown") || self.is_key_pressed("S")
    }

    pub fn input_action(&self) -> bool {
        self.is_key_pressed("Space")
            || self.is_key_pressed("Enter")
            || self.is_key_pressed("Z")
            || self.is_key_pressed("X")
    }

    pub fn pressed_keys(&self) -> HashSet<String> {
        self.pressed_keys.clone()
    }

    pub fn play_sound(&mut self, asset_path: &str) {
        self.audio.play_sound(asset_path);
    }

    pub fn play_music(&mut self, asset_path: &str) {
        self.audio.play_music(asset_path);
    }

    pub fn stop_music(&mut self) {
        self.audio.stop_music();
    }

    pub fn set_window_size(&mut self, window_size: (u32, u32)) {
        self.window_size = window_size;
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    pub fn set_key_pressed(&mut self, key: &str, pressed: bool) {
        let key = normalize_key_name(key);
        if pressed {
            self.pressed_keys.insert(key);
            self.audio.activate();
        } else {
            self.pressed_keys.remove(&key);
        }
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
struct AudioState {
    output_stream: Option<OutputStream>,
    output_handle: Option<OutputStreamHandle>,
    music_sink: Option<Sink>,
    current_music: Option<String>,
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
impl AudioState {
    fn new() -> Self {
        match OutputStream::try_default() {
            Ok((output_stream, output_handle)) => Self {
                output_stream: Some(output_stream),
                output_handle: Some(output_handle),
                music_sink: None,
                current_music: None,
            },
            Err(_) => Self {
                output_stream: None,
                output_handle: None,
                music_sink: None,
                current_music: None,
            },
        }
    }

    fn play_sound(&mut self, asset_path: &str) {
        let Some(bytes) = std::fs::read(asset_path).ok() else {
            return;
        };
        let Some(handle) = self.output_handle.as_ref() else {
            return;
        };
        let Ok(decoder) = Decoder::new(Cursor::new(bytes)) else {
            return;
        };
        let Ok(sink) = Sink::try_new(handle) else {
            return;
        };
        sink.append(decoder);
        sink.detach();
    }

    fn play_music(&mut self, asset_path: &str) {
        let normalized = asset_path.trim_start_matches('/').to_string();
        if self.current_music.as_deref() == Some(normalized.as_str()) {
            return;
        }
        let Some(bytes) = std::fs::read(asset_path).ok() else {
            return;
        };
        let Some(handle) = self.output_handle.as_ref() else {
            return;
        };
        let Ok(decoder) = Decoder::new(Cursor::new(bytes)) else {
            return;
        };
        let Ok(sink) = Sink::try_new(handle) else {
            return;
        };
        sink.append(decoder.repeat_infinite());
        self.stop_music();
        self.music_sink = Some(sink);
        self.current_music = Some(normalized);
    }

    fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
        self.current_music = None;
    }

    fn activate(&mut self) {
        let _ = self.output_stream.as_ref();
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
struct AudioState;

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
unsafe extern "C" {
    fn rpu_apple_play_sound(asset_path: *const std::os::raw::c_char);
    fn rpu_apple_play_music(asset_path: *const std::os::raw::c_char);
    fn rpu_apple_stop_music();
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
impl AudioState {
    fn new() -> Self {
        Self
    }

    fn play_sound(&mut self, asset_path: &str) {
        let playable_path = apple_playable_audio_path(asset_path);
        let Ok(asset_path) = CString::new(playable_path.as_str()) else {
            return;
        };
        unsafe { rpu_apple_play_sound(asset_path.as_ptr()) };
    }

    fn play_music(&mut self, asset_path: &str) {
        let playable_path = apple_playable_audio_path(asset_path);
        let Ok(asset_path) = CString::new(playable_path.as_str()) else {
            return;
        };
        unsafe { rpu_apple_play_music(asset_path.as_ptr()) };
    }

    fn stop_music(&mut self) {
        unsafe { rpu_apple_stop_music() };
    }

    fn activate(&mut self) {}
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
fn apple_playable_audio_path(asset_path: &str) -> String {
    if !asset_path.to_ascii_lowercase().ends_with(".ogg") {
        return asset_path.to_string();
    }
    convert_ogg_to_wav(asset_path).unwrap_or_else(|| asset_path.to_string())
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
fn convert_ogg_to_wav(asset_path: &str) -> Option<String> {
    let src = PathBuf::from(asset_path);
    let metadata = std::fs::metadata(&src).ok()?;
    let modified = metadata.modified().ok()?;
    let modified = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let stem = src.file_stem()?.to_string_lossy();
    let out_dir = std::env::temp_dir().join("rpu-audio-cache");
    std::fs::create_dir_all(&out_dir).ok()?;
    let out_path = out_dir.join(format!("{stem}-{modified}.wav"));
    if out_path.exists() {
        return Some(out_path.to_string_lossy().to_string());
    }

    let file = File::open(&src).ok()?;
    let mut reader = lewton::inside_ogg::OggStreamReader::new(file).ok()?;
    let channels = reader.ident_hdr.audio_channels as u16;
    let sample_rate = reader.ident_hdr.audio_sample_rate;
    let mut pcm = Vec::new();
    while let Some(packet) = reader.read_dec_packet_itl().ok()? {
        pcm.extend(packet);
    }
    write_pcm16_wav(&out_path, channels, sample_rate, &pcm).ok()?;
    Some(out_path.to_string_lossy().to_string())
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
fn write_pcm16_wav(path: &PathBuf, channels: u16, sample_rate: u32, samples: &[i16]) -> std::io::Result<()> {
    let data_len = samples.len().saturating_mul(2) as u32;
    let byte_rate = sample_rate
        .saturating_mul(channels as u32)
        .saturating_mul(2);
    let block_align = channels.saturating_mul(2);
    let mut file = File::create(path)?;
    file.write_all(b"RIFF")?;
    file.write_all(&(36u32.saturating_add(data_len)).to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_len.to_le_bytes())?;
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
struct AudioState {
    music: Option<HtmlAudioElement>,
    cached_data_urls: HashMap<String, String>,
    pending_music: Option<String>,
    current_music: Option<String>,
}

#[cfg(target_arch = "wasm32")]
impl AudioState {
    fn new() -> Self {
        Self {
            music: None,
            cached_data_urls: HashMap::new(),
            pending_music: None,
            current_music: None,
        }
    }

    fn play_sound(&mut self, asset_path: &str) {
        let Some(src) = self.asset_data_url(asset_path) else {
            return;
        };
        let Ok(audio) = HtmlAudioElement::new_with_src(&src) else {
            return;
        };
        audio.set_loop(false);
        let _ = audio.play();
    }

    fn play_music(&mut self, asset_path: &str) {
        let normalized = asset_path.trim_start_matches('/').to_string();
        if self.current_music.as_deref() == Some(normalized.as_str())
            || self.pending_music.as_deref() == Some(normalized.as_str())
        {
            return;
        }
        self.pending_music = Some(normalized);
        self.try_start_pending_music();
    }

    fn stop_music(&mut self) {
        self.pending_music = None;
        if let Some(audio) = self.music.take() {
            audio.pause().ok();
            audio.set_current_time(0.0);
        }
        self.current_music = None;
    }

    fn activate(&mut self) {
        self.try_start_pending_music();
    }

    fn try_start_pending_music(&mut self) {
        let Some(asset_path) = self.pending_music.clone() else {
            return;
        };
        let Some(src) = self.asset_data_url(&asset_path) else {
            return;
        };
        let Ok(audio) = HtmlAudioElement::new_with_src(&src) else {
            return;
        };
        audio.set_loop(true);
        match audio.play() {
            Ok(_) => {
                if let Some(old_audio) = self.music.replace(audio) {
                    old_audio.pause().ok();
                }
                self.current_music = Some(asset_path);
                self.pending_music = None;
            }
            Err(_) => {}
        }
    }

    fn asset_data_url(&mut self, asset_path: &str) -> Option<String> {
        let key = asset_path.trim_start_matches('/').to_string();
        if let Some(url) = self.cached_data_urls.get(&key) {
            return Some(url.clone());
        }
        let bytes = WEB_ASSETS.with(|assets| assets.borrow().get(&key).cloned())?;
        let mime = audio_mime_type_for(&key);
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        let url = format!("data:{mime};base64,{encoded}");
        self.cached_data_urls.insert(key, url.clone());
        Some(url)
    }
}

#[cfg(target_arch = "wasm32")]
fn audio_mime_type_for(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".ogg") {
        "audio/ogg"
    } else if lower.ends_with(".mp3") {
        "audio/mpeg"
    } else if lower.ends_with(".wav") {
        "audio/wav"
    } else {
        "application/octet-stream"
    }
}

pub struct SceneFrame {
    size: (u32, u32),
    pub clear_color: [f32; 4],
    pub items: Vec<RenderItem>,
}

impl SceneFrame {
    fn new(size: (u32, u32)) -> Self {
        Self {
            size,
            clear_color: [0.04, 0.05, 0.08, 1.0],
            items: Vec::new(),
        }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn clear_color(&mut self, rgba: [f32; 4]) {
        self.clear_color = rgba;
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        self.push_rect(0, self.items.len() as i32, x, y, width, height, color);
    }

    pub fn push_rect(
        &mut self,
        layer: i32,
        order: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) {
        self.items.push(RenderItem::Rect(RenderRect {
            layer,
            order,
            x,
            y,
            width,
            height,
            color,
        }));
    }

    pub fn draw_sprite(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        color: [f32; 4],
        flip_x: bool,
        flip_y: bool,
        texture: Option<&str>,
    ) {
        self.push_sprite(
            0,
            self.items.len() as i32,
            x,
            y,
            width,
            height,
            rotation,
            color,
            flip_x,
            flip_y,
            texture,
        );
    }

    pub fn push_sprite(
        &mut self,
        layer: i32,
        order: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rotation: f32,
        color: [f32; 4],
        flip_x: bool,
        flip_y: bool,
        texture: Option<&str>,
    ) {
        self.items.push(RenderItem::Sprite(RenderSprite {
            layer,
            order,
            x,
            y,
            width,
            height,
            rotation,
            color,
            flip_x,
            flip_y,
            texture_path: texture.map(ToOwned::to_owned),
        }));
    }

    pub fn push_text(
        &mut self,
        layer: i32,
        order: i32,
        x: f32,
        y: f32,
        value: &str,
        font: &str,
        font_size: f32,
        color: [f32; 4],
        anchor: Anchor,
        align: TextAlign,
    ) {
        self.items.push(RenderItem::Text(RenderText {
            layer,
            order,
            x,
            y,
            value: value.to_string(),
            font_path: font.to_string(),
            font_size,
            color,
            anchor,
            align,
        }));
    }
}

#[derive(Clone)]
pub enum RenderItem {
    Rect(RenderRect),
    Sprite(RenderSprite),
    Text(RenderText),
}

#[derive(Clone)]
pub struct RenderRect {
    pub layer: i32,
    pub order: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

#[derive(Clone)]
pub struct RenderSprite {
    pub layer: i32,
    pub order: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
    pub texture_path: Option<String>,
}

#[derive(Clone)]
pub struct RenderText {
    pub layer: i32,
    pub order: i32,
    pub x: f32,
    pub y: f32,
    pub value: String,
    pub font_path: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub anchor: Anchor,
    pub align: TextAlign,
}

#[cfg(target_arch = "wasm32")]
struct WebRunner<A: RpuSceneApp> {
    app: A,
    gpu: GpuState,
    ctx: RuntimeContext,
    canvas: HtmlCanvasElement,
    target_frame_ms: Option<f64>,
    last_frame_ms: Option<f64>,
    last_cursor: (f32, f32),
}

#[cfg(target_arch = "wasm32")]
impl<A: RpuSceneApp> WebRunner<A> {
    fn update_and_render(&mut self) {
        self.app.update(&mut self.ctx);
        let mut frame = SceneFrame::new(self.ctx.window_size());
        self.app.render(&mut self.ctx, &mut frame);
        if let Err(error) = self.gpu.render(&frame) {
            web_sys::console::error_1(&format!("rpu-scenevm web render failed: {error:?}").into());
        }
    }

    fn mouse_move(&mut self, x: f32, y: f32) {
        self.last_cursor = (x, y);
        self.app.mouse_move(&mut self.ctx, x, y);
    }

    fn mouse_down(&mut self) {
        let (x, y) = self.last_cursor;
        self.app.mouse_down(&mut self.ctx, x, y);
    }

    fn mouse_up(&mut self) {
        let (x, y) = self.last_cursor;
        self.app.mouse_up(&mut self.ctx, x, y);
    }

    fn scroll(&mut self, dx: f32, dy: f32) {
        self.app.scroll(&mut self.ctx, dx, dy);
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
pub fn run_app<A: RpuSceneApp + 'static>(app: A) -> Result<()> {
    use pollster::block_on;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use winit::application::ApplicationHandler;
    use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
    use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
    use winit::window::{Window, WindowAttributes, WindowId};

    struct NativeRunner<A: RpuSceneApp> {
        app: A,
        window: Option<Arc<Window>>,
        window_id: Option<WindowId>,
        gpu: Option<GpuState>,
        ctx: Option<RuntimeContext>,
        cursor_pos: PhysicalPosition<f64>,
        frame_interval: Option<Duration>,
        last_frame: Instant,
    }

    impl<A: RpuSceneApp> ApplicationHandler for NativeRunner<A> {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }

            let mut attrs = WindowAttributes::default()
                .with_title(self.app.window_title().unwrap_or_else(|| "RPU".to_string()));
            if let Some((w, h)) = self.app.initial_window_size() {
                attrs = attrs.with_inner_size(LogicalSize::new(w as f64, h as f64));
            }

            let window = Arc::new(
                event_loop
                    .create_window(attrs)
                    .expect("failed to create window"),
            );
            let scale_factor = window.scale_factor();
            let size = logical_size(&window);

            let gpu = block_on(GpuState::new(window.clone())).expect("failed to initialize GPU");

            self.app.set_native_mode(true);
            self.app.set_scale(scale_factor as f32);

            let mut ctx = RuntimeContext::new(size, scale_factor as f32);
            self.app.init(&mut ctx);

            self.window_id = Some(window.id());
            self.ctx = Some(ctx);
            self.gpu = Some(gpu);
            self.window = Some(window);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if Some(window_id) != self.window_id {
                return;
            }

            let (Some(window), Some(ctx), Some(gpu)) =
                (self.window.as_ref(), self.ctx.as_mut(), self.gpu.as_mut())
            else {
                return;
            };

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    ctx.set_scale_factor(scale_factor as f32);
                    self.app.set_scale(scale_factor as f32);
                    let size = logical_size(window);
                    ctx.set_window_size(size);
                    gpu.resize(window.inner_size().width, window.inner_size().height);
                    self.app.resize(ctx, size);
                }
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        let logical = logical_size(window);
                        ctx.set_window_size(logical);
                        gpu.resize(size.width, size.height);
                        self.app.resize(ctx, logical);
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.cursor_pos = position;
                    let logical = position.to_logical::<f32>(window.scale_factor());
                    self.app.mouse_move(ctx, logical.x, logical.y);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        let logical = self.cursor_pos.to_logical::<f32>(window.scale_factor());
                        match state {
                            ElementState::Pressed => self.app.mouse_down(ctx, logical.x, logical.y),
                            ElementState::Released => self.app.mouse_up(ctx, logical.x, logical.y),
                        }
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let (dx, dy) = match delta {
                        MouseScrollDelta::LineDelta(dx, dy) => (dx, dy),
                        MouseScrollDelta::PixelDelta(pos) => {
                            let LogicalPosition { x, y } =
                                pos.to_logical::<f32>(window.scale_factor());
                            (x, y)
                        }
                    };
                    self.app.scroll(ctx, dx, dy);
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if let Some(key_name) = key_name_from_event(&event.logical_key) {
                        ctx.set_key_pressed(&key_name, event.state == ElementState::Pressed);
                    }
                }
                WindowEvent::RedrawRequested => {
                    self.app.update(ctx);
                    let mut frame = SceneFrame::new(ctx.window_size());
                    self.app.render(ctx, &mut frame);
                    if let Err(error) = gpu.render(&frame) {
                        match error {
                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                                gpu.resize(window.inner_size().width, window.inner_size().height);
                            }
                            wgpu::SurfaceError::OutOfMemory => {
                                eprintln!("rpu-scenevm: surface out of memory");
                                event_loop.exit();
                            }
                            wgpu::SurfaceError::Timeout => {}
                            wgpu::SurfaceError::Other => {}
                        }
                    }
                }
                _ => {}
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            let Some(window) = self.window.as_ref() else {
                return;
            };
            let Some(ctx) = self.ctx.as_ref() else {
                return;
            };

            if !self.app.needs_update(ctx) {
                event_loop.set_control_flow(ControlFlow::Wait);
                return;
            }

            if let Some(frame_interval) = self.frame_interval {
                let now = Instant::now();
                let elapsed = now.saturating_duration_since(self.last_frame);
                if elapsed >= frame_interval {
                    self.last_frame = now;
                    window.request_redraw();
                    event_loop.set_control_flow(ControlFlow::WaitUntil(now + frame_interval));
                } else {
                    event_loop
                        .set_control_flow(ControlFlow::WaitUntil(self.last_frame + frame_interval));
                }
            } else {
                window.request_redraw();
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        }
    }

    let frame_interval = app.target_fps().and_then(|fps| {
        if fps > 0.0 {
            Some(Duration::from_secs_f32(1.0 / fps))
        } else {
            None
        }
    });

    let event_loop = EventLoop::new()?;
    let mut runner = NativeRunner {
        app,
        window: None,
        window_id: None,
        gpu: None,
        ctx: None,
        cursor_pos: PhysicalPosition::new(0.0, 0.0),
        frame_interval,
        last_frame: Instant::now(),
    };
    event_loop.run_app(&mut runner)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
fn key_name_from_event(key: &winit::keyboard::Key) -> Option<String> {
    match key {
        winit::keyboard::Key::Character(value) => Some(value.to_uppercase()),
        winit::keyboard::Key::Named(named) => match named {
            winit::keyboard::NamedKey::ArrowLeft => Some("ArrowLeft".to_string()),
            winit::keyboard::NamedKey::ArrowRight => Some("ArrowRight".to_string()),
            winit::keyboard::NamedKey::ArrowUp => Some("ArrowUp".to_string()),
            winit::keyboard::NamedKey::ArrowDown => Some("ArrowDown".to_string()),
            winit::keyboard::NamedKey::Space => Some("Space".to_string()),
            winit::keyboard::NamedKey::Enter => Some("Enter".to_string()),
            winit::keyboard::NamedKey::Shift => Some("Shift".to_string()),
            winit::keyboard::NamedKey::Control => Some("Control".to_string()),
            winit::keyboard::NamedKey::Alt => Some("Alt".to_string()),
            winit::keyboard::NamedKey::Escape => Some("Escape".to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn normalize_key_name(key: &str) -> String {
    match key.trim() {
        "Left" => "ArrowLeft".to_string(),
        "Right" => "ArrowRight".to_string(),
        "Up" => "ArrowUp".to_string(),
        "Down" => "ArrowDown".to_string(),
        "" | " " | "Spacebar" => "Space".to_string(),
        other => other
            .to_uppercase()
            .replace("ARROWLEFT", "ArrowLeft")
            .replace("ARROWRIGHT", "ArrowRight")
            .replace("ARROWUP", "ArrowUp")
            .replace("ARROWDOWN", "ArrowDown")
            .replace("SPACE", "Space")
            .replace("ENTER", "Enter")
            .replace("SHIFT", "Shift")
            .replace("CONTROL", "Control")
            .replace("ALT", "Alt")
            .replace("ESCAPE", "Escape"),
    }
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    #[allow(dead_code)]
    config: wgpu::SurfaceConfiguration,
    surface_is_srgb: bool,
    quad_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    white_texture: GpuTexture,
    texture_cache: HashMap<String, GpuTexture>,
    font_cache: HashMap<String, fontdue::Font>,
}

impl GpuState {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "tvos"),
        not(target_os = "ios")
    ))]
    async fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|error| anyhow!("failed to create surface: {error}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("failed to request GPU adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("rpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::default(),
            })
            .await
            .context("failed to request GPU device")?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| caps.formats.first().copied())
            .ok_or_else(|| anyhow!("surface does not expose any formats"))?;
        let present_mode = caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .or_else(|| caps.present_modes.first().copied())
            .ok_or_else(|| anyhow!("surface does not expose any present modes"))?;
        let alpha_mode = caps
            .alpha_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::CompositeAlphaMode::Opaque)
            .or_else(|| caps.alpha_modes.first().copied())
            .unwrap_or(wgpu::CompositeAlphaMode::Opaque);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let surface_is_srgb = config.format.is_srgb();
        let (quad_pipeline, bind_group_layout) = create_quad_pipeline(&device, config.format);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rpu-texture-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let white_texture = GpuTexture::from_rgba(
            &device,
            &queue,
            &sampler,
            &bind_group_layout,
            1,
            1,
            &[255; 4],
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            surface_is_srgb,
            quad_pipeline,
            sampler,
            bind_group_layout,
            white_texture,
            texture_cache: HashMap::new(),
            font_cache: HashMap::new(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    async fn new_canvas(canvas: HtmlCanvasElement) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|error| anyhow!("failed to create web surface: {error}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("failed to request GPU adapter")?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("rpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::default(),
            })
            .await
            .context("failed to request GPU device")?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| caps.formats.first().copied())
            .ok_or_else(|| anyhow!("surface does not expose any formats"))?;
        let present_mode = caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .or_else(|| caps.present_modes.first().copied())
            .ok_or_else(|| anyhow!("surface does not expose any present modes"))?;
        let alpha_mode = caps
            .alpha_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::CompositeAlphaMode::Opaque)
            .or_else(|| caps.alpha_modes.first().copied())
            .unwrap_or(wgpu::CompositeAlphaMode::Opaque);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: canvas.width().max(1),
            height: canvas.height().max(1),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let surface_is_srgb = config.format.is_srgb();
        let (quad_pipeline, bind_group_layout) = create_quad_pipeline(&device, config.format);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rpu-texture-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let white_texture = GpuTexture::from_rgba(
            &device,
            &queue,
            &sampler,
            &bind_group_layout,
            1,
            1,
            &[255; 4],
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            surface_is_srgb,
            quad_pipeline,
            sampler,
            bind_group_layout,
            white_texture,
            texture_cache: HashMap::new(),
            font_cache: HashMap::new(),
        })
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        any(target_os = "macos", target_os = "tvos", target_os = "ios")
    ))]
    fn new_core_animation_layer(layer_ptr: *mut c_void, width: u32, height: u32) -> Result<Self> {
        let width = width.max(1);
        let height = height.max(1);
        let instance = wgpu::Instance::default();
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr))
        }
        .map_err(|error| anyhow!("failed to create surface for CAMetalLayer: {error}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .context("failed to request GPU adapter")?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("rpu-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::default(),
        }))
        .context("failed to request GPU device")?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| caps.formats.first().copied())
            .ok_or_else(|| anyhow!("surface does not expose any formats"))?;
        let present_mode = caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .or_else(|| caps.present_modes.first().copied())
            .ok_or_else(|| anyhow!("surface does not expose any present modes"))?;
        let alpha_mode = caps
            .alpha_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::CompositeAlphaMode::Opaque)
            .or_else(|| caps.alpha_modes.first().copied())
            .unwrap_or(wgpu::CompositeAlphaMode::Opaque);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let surface_is_srgb = config.format.is_srgb();
        let (quad_pipeline, bind_group_layout) = create_quad_pipeline(&device, config.format);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rpu-texture-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let white_texture = GpuTexture::from_rgba(
            &device,
            &queue,
            &sampler,
            &bind_group_layout,
            1,
            1,
            &[255; 4],
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            surface_is_srgb,
            quad_pipeline,
            sampler,
            bind_group_layout,
            white_texture,
            texture_cache: HashMap::new(),
            font_cache: HashMap::new(),
        })
    }

    #[allow(dead_code)]
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self, frame_ctx: &SceneFrame) -> std::result::Result<(), wgpu::SurfaceError> {
        for item in &frame_ctx.items {
            self.ensure_item_texture(item);
        }
        let batches = build_batches(frame_ctx.size, &frame_ctx.items, &self.texture_cache);

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rpu-render-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rpu-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color_for_surface(
                            frame_ctx.clear_color,
                            self.surface_is_srgb,
                        )),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if !batches.is_empty() {
                pass.set_pipeline(&self.quad_pipeline);
                for batch in &batches {
                    let vertex_buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("rpu-quad-vertices"),
                                contents: bytemuck::cast_slice(&batch.vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                    let texture = self.texture_for_key(batch.texture_path.as_deref());
                    pass.set_bind_group(0, &texture.bind_group, &[]);
                    pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    pass.draw(0..batch.vertices.len() as u32, 0..1);
                }
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    fn ensure_texture(&mut self, path: &str) {
        if !self.texture_cache.contains_key(path) {
            if let Ok(textures) = generated_textures().lock() {
                if let Some(asset) = textures.get(path) {
                    let texture = GpuTexture::from_rgba(
                        &self.device,
                        &self.queue,
                        &self.sampler,
                        &self.bind_group_layout,
                        asset.width,
                        asset.height,
                        &asset.rgba,
                    );
                    self.texture_cache.insert(path.to_string(), texture);
                    return;
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            let image = std::fs::read(path)
                .ok()
                .and_then(|bytes| image::load_from_memory(&bytes).ok());
            #[cfg(target_arch = "wasm32")]
            let image = WEB_ASSETS.with(|assets| {
                assets
                    .borrow()
                    .get(path.trim_start_matches('/'))
                    .cloned()
                    .and_then(|bytes| image::load_from_memory(&bytes).ok())
            });
            match image {
                Some(image) => {
                    let rgba = image.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    let texture = GpuTexture::from_rgba(
                        &self.device,
                        &self.queue,
                        &self.sampler,
                        &self.bind_group_layout,
                        width,
                        height,
                        rgba.as_raw(),
                    );
                    self.texture_cache.insert(path.to_string(), texture);
                }
                None => {
                    self.texture_cache
                        .insert(path.to_string(), self.white_texture.clone());
                }
            }
        }
    }

    fn ensure_item_texture(&mut self, item: &RenderItem) {
        match item {
            RenderItem::Rect(_) => {}
            RenderItem::Sprite(sprite) => {
                if let Some(path) = sprite.texture_path.as_deref() {
                    self.ensure_texture(path);
                }
            }
            RenderItem::Text(text) => {
                let key = text.texture_key();
                if self.texture_cache.contains_key(&key) {
                    return;
                }
                match self.rasterize_text(text) {
                    Some((width, height, rgba)) => {
                        let texture = GpuTexture::from_rgba(
                            &self.device,
                            &self.queue,
                            &self.sampler,
                            &self.bind_group_layout,
                            width,
                            height,
                            &rgba,
                        );
                        self.texture_cache.insert(key, texture);
                    }
                    None => {
                        self.texture_cache.insert(key, self.white_texture.clone());
                    }
                }
            }
        }
    }

    fn rasterize_text(&mut self, text: &RenderText) -> Option<(u32, u32, Vec<u8>)> {
        let font = self.load_font(&text.font_path)?;
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        layout.append(&[font], &TextStyle::new(&text.value, text.font_size, 0));

        let glyphs = layout.glyphs();
        if glyphs.is_empty() {
            return Some((1, 1, vec![255, 255, 255, 0]));
        }

        let mut width = 0usize;
        let mut height = 0usize;
        for glyph in glyphs {
            width = width.max((glyph.x + glyph.width as f32).ceil().max(0.0) as usize);
            height = height.max((glyph.y + glyph.height as f32).ceil().max(0.0) as usize);
        }
        width = width.max(1);
        height = height.max(1);

        let mut rgba = vec![0u8; width * height * 4];
        for glyph in glyphs {
            let (metrics, bitmap) = font.rasterize_config(glyph.key);
            let gx = glyph.x.round() as i32;
            let gy = glyph.y.round() as i32;
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let dst_x = gx + col as i32;
                    let dst_y = gy + row as i32;
                    if dst_x < 0 || dst_y < 0 {
                        continue;
                    }
                    let dst_x = dst_x as usize;
                    let dst_y = dst_y as usize;
                    if dst_x >= width || dst_y >= height {
                        continue;
                    }
                    let alpha = bitmap[row * metrics.width + col];
                    let idx = (dst_y * width + dst_x) * 4;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 255;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = alpha;
                }
            }
        }

        Some((width as u32, height as u32, rgba))
    }

    fn load_font(&mut self, path: &str) -> Option<&fontdue::Font> {
        if !self.font_cache.contains_key(path) {
            #[cfg(not(target_arch = "wasm32"))]
            let bytes = std::fs::read(path).ok()?;
            #[cfg(target_arch = "wasm32")]
            let bytes = WEB_ASSETS
                .with(|assets| assets.borrow().get(path.trim_start_matches('/')).cloned())?;
            let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).ok()?;
            self.font_cache.insert(path.to_string(), font);
        }
        self.font_cache.get(path)
    }

    fn texture_for_key(&self, path: Option<&str>) -> &GpuTexture {
        let Some(path) = path else {
            return &self.white_texture;
        };
        self.texture_cache.get(path).unwrap_or(&self.white_texture)
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "tvos", target_os = "ios")
))]
pub struct MetalLayerRunner<A: RpuSceneApp> {
    app: A,
    gpu: GpuState,
    ctx: RuntimeContext,
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "tvos", target_os = "ios")
))]
impl<A: RpuSceneApp> MetalLayerRunner<A> {
    pub fn new(
        mut app: A,
        layer_ptr: *mut c_void,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Result<Self> {
        let gpu = GpuState::new_core_animation_layer(layer_ptr, width, height)?;
        let mut ctx = RuntimeContext::new((width.max(1), height.max(1)), scale_factor);
        app.set_native_mode(false);
        app.set_scale(scale_factor);
        app.init(&mut ctx);
        Ok(Self { app, gpu, ctx })
    }

    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        let size = (width.max(1), height.max(1));
        self.ctx.set_window_size(size);
        self.ctx.set_scale_factor(scale_factor);
        self.app.set_scale(scale_factor);
        self.app.resize(&mut self.ctx, size);
        self.gpu.resize(size.0, size.1);
    }

    pub fn key_down(&mut self, key: &str) {
        self.ctx.set_key_pressed(key, true);
    }

    pub fn key_up(&mut self, key: &str) {
        self.ctx.set_key_pressed(key, false);
    }

    pub fn mouse_down(&mut self, x: f32, y: f32) {
        self.app.mouse_down(&mut self.ctx, x, y);
    }

    pub fn mouse_up(&mut self, x: f32, y: f32) {
        self.app.mouse_up(&mut self.ctx, x, y);
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.app.mouse_move(&mut self.ctx, x, y);
    }

    pub fn scroll(&mut self, dx: f32, dy: f32) {
        self.app.scroll(&mut self.ctx, dx, dy);
    }

    pub fn render(&mut self) -> Result<()> {
        self.app.update(&mut self.ctx);
        let mut frame = SceneFrame::new(self.ctx.window_size());
        self.app.render(&mut self.ctx, &mut frame);
        match self.gpu.render(&frame) {
            Ok(()) => Ok(()),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let (w, h) = self.ctx.window_size();
                self.gpu.resize(w.max(1), h.max(1));
                Ok(())
            }
            Err(wgpu::SurfaceError::OutOfMemory) => Err(anyhow!("GPU surface out of memory")),
            Err(wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Other) => Ok(()),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn run_app<A: RpuSceneApp + 'static>(app: A) -> Result<()> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().ok_or_else(|| anyhow!("missing browser window"))?;
    let document = window
        .document()
        .ok_or_else(|| anyhow!("missing browser document"))?;
    let canvas = document
        .create_element("canvas")
        .map_err(|error| anyhow!("failed to create canvas: {error:?}"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| anyhow!("failed to cast canvas element"))?;
    let body = document
        .body()
        .ok_or_else(|| anyhow!("missing browser body"))?;

    let initial_size = app.initial_window_size().unwrap_or((1280, 720));
    let device_scale = fit_canvas_to_viewport(&window, &canvas, initial_size)?;
    body.append_child(&canvas)
        .map_err(|error| anyhow!("failed to attach canvas: {error:?}"))?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = start_web_app(app, canvas, initial_size, device_scale).await {
            web_sys::console::error_1(&format!("rpu-scenevm web startup failed: {error:#}").into());
        }
    });
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn start_web_app<A: RpuSceneApp + 'static>(
    mut app: A,
    canvas: HtmlCanvasElement,
    logical_size: (u32, u32),
    device_scale: f32,
) -> Result<()> {
    let gpu = GpuState::new_canvas(canvas.clone()).await?;
    let mut ctx = RuntimeContext::new(
        (canvas.width().max(1), canvas.height().max(1)),
        device_scale,
    );
    app.set_native_mode(false);
    app.set_scale(device_scale);
    app.init(&mut ctx);

    let runner = Rc::new(RefCell::new(WebRunner {
        app,
        gpu,
        ctx,
        canvas: canvas.clone(),
        target_frame_ms: None,
        last_frame_ms: None,
        last_cursor: (0.0, 0.0),
    }));

    {
        let mut runner_ref = runner.borrow_mut();
        runner_ref.target_frame_ms = runner_ref
            .app
            .target_fps()
            .and_then(|fps| (fps > 0.0).then_some(1000.0 / fps as f64));
    }

    register_web_resize_handler(&runner, logical_size)?;
    register_web_input_handlers(&runner)?;

    let animation = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let animation_clone = animation.clone();
    let runner_clone = runner.clone();
    *animation.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp_ms: f64| {
        {
            let mut runner = runner_clone.borrow_mut();
            let should_draw = match runner.target_frame_ms {
                Some(interval) => match runner.last_frame_ms {
                    Some(last) => timestamp_ms - last >= interval,
                    None => true,
                },
                None => true,
            };
            if should_draw {
                runner.last_frame_ms = Some(timestamp_ms);
                runner.update_and_render();
            }
        }
        let window = web_sys::window().expect("window");
        let _ = window.request_animation_frame(
            animation_clone
                .borrow()
                .as_ref()
                .expect("animation closure")
                .as_ref()
                .unchecked_ref(),
        );
    }) as Box<dyn FnMut(f64)>));

    let window = web_sys::window().ok_or_else(|| anyhow!("missing browser window"))?;
    window
        .request_animation_frame(
            animation
                .borrow()
                .as_ref()
                .expect("animation closure")
                .as_ref()
                .unchecked_ref(),
        )
        .map_err(|error| anyhow!("requestAnimationFrame failed: {error:?}"))?;

    std::mem::forget(animation);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn configure_canvas(
    canvas: &HtmlCanvasElement,
    logical_size: (u32, u32),
    scale: f32,
) -> Result<()> {
    let style = canvas.style();
    style
        .set_property("width", &format!("{}px", logical_size.0))
        .map_err(|error| anyhow!("failed to style canvas width: {error:?}"))?;
    style
        .set_property("height", &format!("{}px", logical_size.1))
        .map_err(|error| anyhow!("failed to style canvas height: {error:?}"))?;
    style
        .set_property("display", "block")
        .map_err(|error| anyhow!("failed to style canvas display: {error:?}"))?;
    canvas.set_width((logical_size.0 as f32 * scale).round().max(1.0) as u32);
    canvas.set_height((logical_size.1 as f32 * scale).round().max(1.0) as u32);
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn fit_canvas_to_viewport(
    window: &web_sys::Window,
    canvas: &HtmlCanvasElement,
    logical_size: (u32, u32),
) -> Result<f32> {
    let viewport_w = window
        .inner_width()
        .map_err(|error| anyhow!("failed to read window innerWidth: {error:?}"))?
        .as_f64()
        .unwrap_or(logical_size.0 as f64)
        .max(1.0);
    let viewport_h = window
        .inner_height()
        .map_err(|error| anyhow!("failed to read window innerHeight: {error:?}"))?
        .as_f64()
        .unwrap_or(logical_size.1 as f64)
        .max(1.0);
    let logical_w = logical_size.0.max(1) as f64;
    let logical_h = logical_size.1.max(1) as f64;
    let fit_scale = (viewport_w / logical_w)
        .min(viewport_h / logical_h)
        .max(0.1);
    let css_w = (logical_w * fit_scale).round().max(1.0) as u32;
    let css_h = (logical_h * fit_scale).round().max(1.0) as u32;
    let device_scale = window.device_pixel_ratio().max(1.0) as f32;
    configure_canvas(canvas, (css_w, css_h), device_scale)?;
    Ok(device_scale)
}

#[cfg(target_arch = "wasm32")]
fn register_web_resize_handler<A: RpuSceneApp + 'static>(
    runner: &Rc<RefCell<WebRunner<A>>>,
    logical_size: (u32, u32),
) -> Result<()> {
    let window = web_sys::window().ok_or_else(|| anyhow!("missing browser window"))?;
    let runner = runner.clone();
    let resize = Closure::wrap(Box::new(move || {
        let Some(window) = web_sys::window() else {
            return;
        };
        let canvas = runner.borrow().canvas.clone();
        let Ok(device_scale) = fit_canvas_to_viewport(&window, &canvas, logical_size) else {
            return;
        };
        let mut runner = runner.borrow_mut();
        let pixel_size = (canvas.width().max(1), canvas.height().max(1));
        let WebRunner { app, gpu, ctx, .. } = &mut *runner;
        ctx.set_window_size(pixel_size);
        ctx.set_scale_factor(device_scale);
        app.set_scale(device_scale);
        app.resize(ctx, pixel_size);
        gpu.resize(pixel_size.0, pixel_size.1);
    }) as Box<dyn FnMut()>);
    window
        .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())
        .map_err(|error| anyhow!("failed to register resize: {error:?}"))?;
    resize.forget();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn register_web_input_handlers<A: RpuSceneApp + 'static>(
    runner: &Rc<RefCell<WebRunner<A>>>,
) -> Result<()> {
    let window = web_sys::window().ok_or_else(|| anyhow!("missing browser window"))?;

    {
        let runner = runner.clone();
        let keydown = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();
            if matches!(
                key.as_str(),
                " " | "Spacebar"
                    | "Space"
                    | "ArrowUp"
                    | "ArrowDown"
                    | "ArrowLeft"
                    | "ArrowRight"
                    | "Enter"
            ) {
                event.prevent_default();
            }
            runner.borrow_mut().ctx.set_key_pressed(&key, true);
        }) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
            .map_err(|error| anyhow!("failed to register keydown: {error:?}"))?;
        keydown.forget();
    }

    {
        let runner = runner.clone();
        let keyup = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();
            if matches!(
                key.as_str(),
                " " | "Spacebar"
                    | "Space"
                    | "ArrowUp"
                    | "ArrowDown"
                    | "ArrowLeft"
                    | "ArrowRight"
                    | "Enter"
            ) {
                event.prevent_default();
            }
            runner.borrow_mut().ctx.set_key_pressed(&key, false);
        }) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
            .map_err(|error| anyhow!("failed to register keyup: {error:?}"))?;
        keyup.forget();
    }

    {
        let runner_for_listener = runner.clone();
        let runner = runner.clone();
        let mousemove = Closure::wrap(Box::new(move |event: MouseEvent| {
            let mut runner = runner.borrow_mut();
            let rect = runner.canvas.get_bounding_client_rect();
            let x = ((event.client_x() as f64 - rect.left()) as f32).max(0.0);
            let y = ((event.client_y() as f64 - rect.top()) as f32).max(0.0);
            runner.mouse_move(x, y);
        }) as Box<dyn FnMut(_)>);
        runner_for_listener
            .borrow()
            .canvas
            .add_event_listener_with_callback("mousemove", mousemove.as_ref().unchecked_ref())
            .map_err(|error| anyhow!("failed to register mousemove: {error:?}"))?;
        mousemove.forget();
    }

    {
        let runner_for_listener = runner.clone();
        let runner = runner.clone();
        let mousedown = Closure::wrap(Box::new(move |_event: MouseEvent| {
            runner.borrow_mut().mouse_down();
        }) as Box<dyn FnMut(_)>);
        runner_for_listener
            .borrow()
            .canvas
            .add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())
            .map_err(|error| anyhow!("failed to register mousedown: {error:?}"))?;
        mousedown.forget();
    }

    {
        let runner_for_listener = runner.clone();
        let runner = runner.clone();
        let mouseup = Closure::wrap(Box::new(move |_event: MouseEvent| {
            runner.borrow_mut().mouse_up();
        }) as Box<dyn FnMut(_)>);
        runner_for_listener
            .borrow()
            .canvas
            .add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())
            .map_err(|error| anyhow!("failed to register mouseup: {error:?}"))?;
        mouseup.forget();
    }

    {
        let runner_for_listener = runner.clone();
        let runner = runner.clone();
        let wheel = Closure::wrap(Box::new(move |event: WheelEvent| {
            runner
                .borrow_mut()
                .scroll(event.delta_x() as f32, event.delta_y() as f32);
        }) as Box<dyn FnMut(_)>);
        runner_for_listener
            .borrow()
            .canvas
            .add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref())
            .map_err(|error| anyhow!("failed to register wheel: {error:?}"))?;
        wheel.forget();
    }

    Ok(())
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
fn logical_size(window: &winit::window::Window) -> (u32, u32) {
    let size = window.inner_size();
    let logical = size.to_logical::<f64>(window.scale_factor());
    (logical.width.round() as u32, logical.height.round() as u32)
}

impl RenderItem {
    fn layer(&self) -> i32 {
        match self {
            RenderItem::Rect(rect) => rect.layer,
            RenderItem::Sprite(sprite) => sprite.layer,
            RenderItem::Text(text) => text.layer,
        }
    }

    fn order(&self) -> i32 {
        match self {
            RenderItem::Rect(rect) => rect.order,
            RenderItem::Sprite(sprite) => sprite.order,
            RenderItem::Text(text) => text.order,
        }
    }

    fn texture_key(&self) -> Option<String> {
        match self {
            RenderItem::Rect(_) => None,
            RenderItem::Sprite(sprite) => sprite.texture_path.clone(),
            RenderItem::Text(text) => Some(text.texture_key()),
        }
    }
}

impl RenderText {
    fn texture_key(&self) -> String {
        format!(
            "text://{}:{}:{}",
            self.font_path, self.font_size, self.value
        )
    }
}

#[derive(Clone)]
struct GpuTexture {
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

struct DrawBatch {
    texture_path: Option<String>,
    vertices: Vec<QuadVertex>,
}

impl GpuTexture {
    fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rpu-texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width.max(1)),
                rows_per_image: Some(height.max(1)),
            },
            wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rpu-texture-bind-group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        Self {
            bind_group,
            width: width.max(1),
            height: height.max(1),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadVertex {
    position: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

fn build_vertices(
    size: (u32, u32),
    rects: &[RenderItem],
    texture_cache: &HashMap<String, GpuTexture>,
) -> Vec<QuadVertex> {
    let w = size.0.max(1) as f32;
    let h = size.1.max(1) as f32;
    let mut out = Vec::with_capacity(rects.len() * 6);
    for rect in rects {
        let (x, y, width, height, rotation, color, flip_x, flip_y) = match rect {
            RenderItem::Rect(rect) => (
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                0.0,
                rect.color,
                false,
                false,
            ),
            RenderItem::Sprite(rect) => (
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                rect.rotation,
                [
                    (rect.color[0] * 0.92).min(1.0),
                    (rect.color[1] * 0.98).min(1.0),
                    (rect.color[2] * 1.05).min(1.0),
                    rect.color[3],
                ],
                rect.flip_x,
                rect.flip_y,
            ),
            RenderItem::Text(text) => {
                let key = text.texture_key();
                let dims = texture_cache
                    .get(&key)
                    .map(|texture| (texture.width as f32, texture.height as f32))
                    .unwrap_or((1.0, 1.0));
                let (tx, ty) = anchored_text_position(size, text, dims.0, dims.1);
                (tx, ty, dims.0, dims.1, 0.0, text.color, false, false)
            }
        };
        let u0 = if flip_x { 1.0 } else { 0.0 };
        let u1 = if flip_x { 0.0 } else { 1.0 };
        let v0 = if flip_y { 1.0 } else { 0.0 };
        let v1 = if flip_y { 0.0 } else { 1.0 };
        let uv0 = [u0, v0];
        let uv1 = [u1, v0];
        let uv2 = [u1, v1];
        let uv3 = [u0, v1];
        let center_x = x + width * 0.5;
        let center_y = y + height * 0.5;
        let half_w = width * 0.5;
        let half_h = height * 0.5;
        let (sin_r, cos_r) = rotation.sin_cos();
        let rotate = |local_x: f32, local_y: f32| -> [f32; 2] {
            let world_x = center_x + local_x * cos_r - local_y * sin_r;
            let world_y = center_y + local_x * sin_r + local_y * cos_r;
            to_ndc(world_x, world_y, w, h)
        };
        let p0 = rotate(-half_w, -half_h);
        let p1 = rotate(half_w, -half_h);
        let p2 = rotate(half_w, half_h);
        let p3 = rotate(-half_w, half_h);
        out.extend_from_slice(&[
            QuadVertex {
                position: p0,
                color,
                uv: uv0,
            },
            QuadVertex {
                position: p1,
                color,
                uv: uv1,
            },
            QuadVertex {
                position: p2,
                color,
                uv: uv2,
            },
            QuadVertex {
                position: p0,
                color,
                uv: uv0,
            },
            QuadVertex {
                position: p2,
                color,
                uv: uv2,
            },
            QuadVertex {
                position: p3,
                color,
                uv: uv3,
            },
        ]);
    }
    out
}

fn anchored_text_position(
    size: (u32, u32),
    text: &RenderText,
    width: f32,
    height: f32,
) -> (f32, f32) {
    let viewport_w = size.0 as f32;
    let viewport_h = size.1 as f32;
    let base_x = match text.anchor {
        Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft | Anchor::World => text.x,
        Anchor::Top | Anchor::Center | Anchor::Bottom => viewport_w * 0.5 + text.x,
        Anchor::TopRight | Anchor::Right | Anchor::BottomRight => viewport_w + text.x,
    };
    let final_x = match text.align {
        TextAlign::Left => base_x,
        TextAlign::Center => base_x - width * 0.5,
        TextAlign::Right => base_x - width,
    };
    let final_y = match text.anchor {
        Anchor::TopLeft | Anchor::Top | Anchor::TopRight | Anchor::World => text.y,
        Anchor::Left | Anchor::Center | Anchor::Right => viewport_h * 0.5 - height * 0.5 + text.y,
        Anchor::BottomLeft | Anchor::Bottom | Anchor::BottomRight => viewport_h - height + text.y,
    };
    (final_x, final_y)
}

fn build_batches(
    size: (u32, u32),
    quads: &[RenderItem],
    texture_cache: &HashMap<String, GpuTexture>,
) -> Vec<DrawBatch> {
    let mut sorted: Vec<RenderItem> = quads.to_vec();
    sorted.sort_by_key(|item| (item.layer(), item.order()));

    let mut batches: Vec<DrawBatch> = Vec::new();
    for quad in &sorted {
        let key = quad.texture_key();
        let vertices = build_vertices(size, std::slice::from_ref(quad), texture_cache);
        match batches.last_mut() {
            Some(last) if last.texture_path == key => last.vertices.extend(vertices),
            _ => batches.push(DrawBatch {
                texture_path: key,
                vertices,
            }),
        }
    }
    batches
}

fn to_ndc(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
    [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0]
}

fn srgb_encode(channel: f32) -> f32 {
    channel.clamp(0.0, 1.0).powf(1.0 / 2.2)
}

fn clear_color_for_surface(color: [f32; 4], surface_is_srgb: bool) -> wgpu::Color {
    let [r, g, b, a] = color;
    let (r, g, b) = if surface_is_srgb {
        (r, g, b)
    } else {
        (srgb_encode(r), srgb_encode(g), srgb_encode(b))
    };
    wgpu::Color {
        r: r as f64,
        g: g as f64,
        b: b as f64,
        a: a as f64,
    }
}

fn create_quad_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("rpu-quad-texture-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let fragment = if format.is_srgb() {
        "    return texel * v.color;\n"
    } else {
        "    let out_color = texel * v.color;\n    return vec4<f32>(pow(out_color.rgb, vec3<f32>(1.0 / 2.2)), out_color.a);\n"
    };
    let shader_source = format!(
        r#"
struct VertexIn {{
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOut {{
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}};

@group(0) @binding(0) var quad_tex: texture_2d<f32>;
@group(0) @binding(1) var quad_sampler: sampler;

@vertex
fn vs_main(v: VertexIn) -> VertexOut {{
    var out: VertexOut;
    out.position = vec4<f32>(v.position, 0.0, 1.0);
    out.color = v.color;
    out.uv = v.uv;
    return out;
}}

@fragment
fn fs_main(v: VertexOut) -> @location(0) vec4<f32> {{
    let texel = textureSample(quad_tex, quad_sampler, v.uv);
{fragment}}}
"#
    );
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("rpu-quad-shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_source)),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("rpu-quad-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("rpu-quad-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<QuadVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x2],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    (pipeline, bind_group_layout)
}
