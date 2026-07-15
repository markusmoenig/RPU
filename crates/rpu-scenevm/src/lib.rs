use anyhow::{Result, anyhow};
#[cfg(target_arch = "wasm32")]
use base64::Engine;
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use rpu_graphics::GraphicsRenderer;
#[cfg(not(target_arch = "wasm32"))]
use rpu_graphics::RenderError;
pub use rpu_graphics::{SceneFrame, register_generated_rgba_texture, register_web_asset};
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
use std::ffi::CString;
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "tvos", target_os = "ios")
))]
use std::ffi::c_void;
#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "tvos"),
    not(target_os = "ios")
))]
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "tvos", target_os = "ios")
))]
use std::{fs::File, io::Write, path::PathBuf};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlAudioElement, HtmlCanvasElement, KeyboardEvent, MouseEvent, WheelEvent};

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
fn write_pcm16_wav(
    path: &PathBuf,
    channels: u16,
    sample_rate: u32,
    samples: &[i16],
) -> std::io::Result<()> {
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
        let bytes = rpu_graphics::web_asset_bytes(&key)?;
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

#[cfg(target_arch = "wasm32")]
struct WebRunner<A: RpuSceneApp> {
    app: A,
    gpu: GraphicsRenderer,
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
        gpu: Option<GraphicsRenderer>,
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

            let gpu = block_on(GraphicsRenderer::new_window(window.clone()))
                .expect("failed to initialize GPU");

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
                            RenderError::SurfaceLost | RenderError::SurfaceOutdated => {
                                gpu.resize(window.inner_size().width, window.inner_size().height);
                            }
                            RenderError::OutOfMemory => {
                                eprintln!("rpu-scenevm: surface out of memory");
                                event_loop.exit();
                            }
                            RenderError::Timeout | RenderError::Other => {}
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

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "tvos", target_os = "ios")
))]
pub struct MetalLayerRunner<A: RpuSceneApp> {
    app: A,
    gpu: GraphicsRenderer,
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
        let gpu = GraphicsRenderer::new_core_animation_layer(layer_ptr, width, height)?;
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
            Err(RenderError::SurfaceLost | RenderError::SurfaceOutdated) => {
                let (w, h) = self.ctx.window_size();
                self.gpu.resize(w.max(1), h.max(1));
                Ok(())
            }
            Err(RenderError::OutOfMemory) => Err(anyhow!("GPU surface out of memory")),
            Err(RenderError::Timeout | RenderError::Other) => Ok(()),
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
    let gpu = GraphicsRenderer::new_canvas(canvas.clone()).await?;
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
