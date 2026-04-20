use anyhow::{Context as AnyhowContext, Result, anyhow};
use bytemuck::{Pod, Zeroable};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::util::DeviceExt;

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

#[derive(Debug, Clone)]
pub struct RuntimeContext {
    window_size: (u32, u32),
    scale_factor: f32,
}

impl RuntimeContext {
    pub fn new(window_size: (u32, u32), scale_factor: f32) -> Self {
        Self {
            window_size,
            scale_factor,
        }
    }

    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    fn set_window_size(&mut self, window_size: (u32, u32)) {
        self.window_size = window_size;
    }

    fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
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
        mut color: [f32; 4],
        texture: Option<&str>,
    ) {
        color[3] = color[3].min(0.96);
        self.push_sprite(
            0,
            self.items.len() as i32,
            x,
            y,
            width,
            height,
            color,
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
        mut color: [f32; 4],
        texture: Option<&str>,
    ) {
        color[3] = color[3].min(0.96);
        self.items.push(RenderItem::Sprite(RenderSprite {
            layer,
            order,
            x,
            y,
            width,
            height,
            color,
            texture_path: texture.map(ToOwned::to_owned),
        }));
    }
}

#[derive(Clone)]
pub enum RenderItem {
    Rect(RenderRect),
    Sprite(RenderSprite),
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
    pub color: [f32; 4],
    pub texture_path: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
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

            let mut attrs =
                WindowAttributes::default().with_title(self.app.window_title().unwrap_or_else(|| {
                    "RPU".to_string()
                }));
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
                    event_loop.set_control_flow(ControlFlow::WaitUntil(
                        self.last_frame + frame_interval,
                    ));
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
struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    quad_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    white_texture: GpuTexture,
    texture_cache: HashMap<String, GpuTexture>,
}

#[cfg(not(target_arch = "wasm32"))]
impl GpuState {
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
            .first()
            .copied()
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
        let white_texture =
            GpuTexture::from_rgba(&device, &queue, &sampler, &bind_group_layout, 1, 1, &[255; 4]);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            quad_pipeline,
            sampler,
            bind_group_layout,
            white_texture,
            texture_cache: HashMap::new(),
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render(&mut self, frame_ctx: &SceneFrame) -> std::result::Result<(), wgpu::SurfaceError> {
        let batches = build_batches(frame_ctx.size, &frame_ctx.items);
        for batch in &batches {
            if let Some(path) = batch.texture_path.as_deref() {
                self.ensure_texture(path);
            }
        }

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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: frame_ctx.clear_color[0] as f64,
                            g: frame_ctx.clear_color[1] as f64,
                            b: frame_ctx.clear_color[2] as f64,
                            a: frame_ctx.clear_color[3] as f64,
                        }),
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
                        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            match std::fs::read(path)
                .ok()
                .and_then(|bytes| image::load_from_memory(&bytes).ok())
            {
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

    fn texture_for_key(&self, path: Option<&str>) -> &GpuTexture {
        let Some(path) = path else {
            return &self.white_texture;
        };
        self.texture_cache.get(path).unwrap_or(&self.white_texture)
    }
}

#[cfg(target_arch = "wasm32")]
pub fn run_app<A: RpuSceneApp + 'static>(_app: A) -> Result<()> {
    anyhow::bail!("wasm runner is not implemented yet")
}

#[cfg(not(target_arch = "wasm32"))]
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
        }
    }

    fn order(&self) -> i32 {
        match self {
            RenderItem::Rect(rect) => rect.order,
            RenderItem::Sprite(sprite) => sprite.order,
        }
    }

    fn texture_path(&self) -> Option<&str> {
        match self {
            RenderItem::Rect(_) => None,
            RenderItem::Sprite(sprite) => sprite.texture_path.as_deref(),
        }
    }
}

#[derive(Clone)]
struct GpuTexture {
    bind_group: wgpu::BindGroup,
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
        Self { bind_group }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadVertex {
    position: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

fn build_vertices(size: (u32, u32), rects: &[RenderItem]) -> Vec<QuadVertex> {
    let w = size.0.max(1) as f32;
    let h = size.1.max(1) as f32;
    let mut out = Vec::with_capacity(rects.len() * 6);
    for rect in rects {
        let (x, y, width, height, color) = match rect {
            RenderItem::Rect(rect) => (rect.x, rect.y, rect.width, rect.height, rect.color),
            RenderItem::Sprite(rect) => (
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                [
                    (rect.color[0] * 0.92).min(1.0),
                    (rect.color[1] * 0.98).min(1.0),
                    (rect.color[2] * 1.05).min(1.0),
                    rect.color[3],
                ],
            ),
        };
        let x0 = x;
        let y0 = y;
        let x1 = x + width;
        let y1 = y + height;
        let p0 = to_ndc(x0, y0, w, h);
        let p1 = to_ndc(x1, y0, w, h);
        let p2 = to_ndc(x1, y1, w, h);
        let p3 = to_ndc(x0, y1, w, h);
        out.extend_from_slice(&[
            QuadVertex { position: p0, color, uv: [0.0, 0.0] },
            QuadVertex { position: p1, color, uv: [1.0, 0.0] },
            QuadVertex { position: p2, color, uv: [1.0, 1.0] },
            QuadVertex { position: p0, color, uv: [0.0, 0.0] },
            QuadVertex { position: p2, color, uv: [1.0, 1.0] },
            QuadVertex { position: p3, color, uv: [0.0, 1.0] },
        ]);
    }
    out
}

fn build_batches(size: (u32, u32), quads: &[RenderItem]) -> Vec<DrawBatch> {
    let mut sorted: Vec<RenderItem> = quads.to_vec();
    sorted.sort_by_key(|item| (item.layer(), item.order()));

    let mut batches: Vec<DrawBatch> = Vec::new();
    for quad in &sorted {
        let key = quad.texture_path().map(ToOwned::to_owned);
        let vertices = build_vertices(size, std::slice::from_ref(quad));
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
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("rpu-quad-shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
            r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0) var quad_tex: texture_2d<f32>;
@group(0) @binding(1) var quad_sampler: sampler;

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.position = vec4<f32>(v.position, 0.0, 1.0);
    out.color = v.color;
    out.uv = v.uv;
    return out;
}

@fragment
fn fs_main(v: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(quad_tex, quad_sampler, v.uv);
    return texel * v.color;
}
"#,
        )),
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
