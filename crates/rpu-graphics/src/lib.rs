//! Host implementation of RPU's portable graphics service.
//!
//! RPU source uses `graphics.*`; `rpu_graphics` is reserved for the shared WASM
//! ABI, and this crate owns the trusted renderer behind both paths.

#[cfg(feature = "wasm-host")]
mod service;
#[cfg(feature = "wasm-host")]
pub use service::GraphicsService;

use anyhow::{Context, Result, anyhow};
use bytemuck::{Pod, Zeroable};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use rpu_core::{Anchor, TextAlign};
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_os = "macos", target_os = "tvos", target_os = "ios")
))]
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
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

#[cfg(target_arch = "wasm32")]
pub fn web_asset_bytes(path: &str) -> Option<Vec<u8>> {
    WEB_ASSETS.with(|assets| assets.borrow().get(path.trim_start_matches('/')).cloned())
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

pub struct SceneFrame {
    size: (u32, u32),
    pub clear_color: [f32; 4],
    pub items: Vec<RenderItem>,
}

impl SceneFrame {
    pub fn new(size: (u32, u32)) -> Self {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderError {
    SurfaceLost,
    SurfaceOutdated,
    OutOfMemory,
    Timeout,
    Other,
}

impl From<wgpu::SurfaceError> for RenderError {
    fn from(error: wgpu::SurfaceError) -> Self {
        match error {
            wgpu::SurfaceError::Lost => Self::SurfaceLost,
            wgpu::SurfaceError::Outdated => Self::SurfaceOutdated,
            wgpu::SurfaceError::OutOfMemory => Self::OutOfMemory,
            wgpu::SurfaceError::Timeout => Self::Timeout,
            wgpu::SurfaceError::Other => Self::Other,
        }
    }
}

pub struct GraphicsRenderer {
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

impl GraphicsRenderer {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "tvos"),
        not(target_os = "ios")
    ))]
    pub async fn new_window(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
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
    pub async fn new_canvas(canvas: HtmlCanvasElement) -> Result<Self> {
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
    pub fn new_core_animation_layer(
        layer_ptr: *mut c_void,
        width: u32,
        height: u32,
    ) -> Result<Self> {
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
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, frame_ctx: &SceneFrame) -> std::result::Result<(), RenderError> {
        for item in &frame_ctx.items {
            self.ensure_item_texture(item);
        }
        let batches = build_batches(frame_ctx.size, &frame_ctx.items, &self.texture_cache);

        let frame = self
            .surface
            .get_current_texture()
            .map_err(RenderError::from)?;
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
