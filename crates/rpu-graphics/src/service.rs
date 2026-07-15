use crate::SceneFrame;
use rpu_core::wasm_abi;
use wasmer::{Function, FunctionEnv, FunctionEnvMut, Imports, RuntimeError, Store};

const MAX_FRAME_DIMENSION: i32 = 16_384;

pub struct GraphicsService {
    env: FunctionEnv<GraphicsState>,
}

struct GraphicsState {
    active_frame: Option<SceneFrame>,
    completed_frame: Option<SceneFrame>,
}

impl GraphicsService {
    pub fn new(store: &mut Store) -> Self {
        Self {
            env: FunctionEnv::new(
                store,
                GraphicsState {
                    active_frame: None,
                    completed_frame: None,
                },
            ),
        }
    }

    pub fn namespace() -> &'static str {
        wasm_abi::GRAPHICS_IMPORT_MODULE
    }

    pub fn supports_import(name: &str) -> bool {
        wasm_abi::graphics_imports()
            .iter()
            .any(|spec| spec.name == name)
    }

    pub fn register(&self, store: &mut Store, imports: &mut Imports) {
        let namespace = Self::namespace();
        imports.define(
            namespace,
            wasm_abi::IMPORT_GRAPHICS_BEGIN_FRAME,
            Function::new_typed_with_env(store, &self.env, host_begin_frame),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_GRAPHICS_CLEAR,
            Function::new_typed_with_env(store, &self.env, host_clear),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_GRAPHICS_DRAW_RECT,
            Function::new_typed_with_env(store, &self.env, host_draw_rect),
        );
        imports.define(
            namespace,
            wasm_abi::IMPORT_GRAPHICS_END_FRAME,
            Function::new_typed_with_env(store, &self.env, host_end_frame),
        );
    }

    pub fn take_completed_frame(&self, store: &mut Store) -> Option<SceneFrame> {
        self.env.as_mut(store).completed_frame.take()
    }
}

fn host_begin_frame(
    mut env: FunctionEnvMut<GraphicsState>,
    width: i32,
    height: i32,
) -> Result<(), RuntimeError> {
    if !(1..=MAX_FRAME_DIMENSION).contains(&width) || !(1..=MAX_FRAME_DIMENSION).contains(&height) {
        return Err(RuntimeError::new(format!(
            "graphics frame size must be between 1 and {MAX_FRAME_DIMENSION}"
        )));
    }
    if env.data().active_frame.is_some() {
        return Err(RuntimeError::new(
            "graphics begin_frame called before end_frame",
        ));
    }
    env.data_mut().active_frame = Some(SceneFrame::new((width as u32, height as u32)));
    Ok(())
}

fn host_clear(
    mut env: FunctionEnvMut<GraphicsState>,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) -> Result<(), RuntimeError> {
    let color = checked_color(red, green, blue, alpha)?;
    env.data_mut()
        .active_frame
        .as_mut()
        .ok_or_else(|| RuntimeError::new("graphics command called outside a frame"))?
        .clear_color(color);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn host_draw_rect(
    mut env: FunctionEnvMut<GraphicsState>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) -> Result<(), RuntimeError> {
    let x = checked_finite("rectangle x", x)?;
    let y = checked_finite("rectangle y", y)?;
    let width = checked_finite("rectangle width", width)?;
    let height = checked_finite("rectangle height", height)?;
    if width < 0.0 || height < 0.0 {
        return Err(RuntimeError::new(
            "graphics rectangle dimensions cannot be negative",
        ));
    }
    let color = checked_color(red, green, blue, alpha)?;
    env.data_mut()
        .active_frame
        .as_mut()
        .ok_or_else(|| RuntimeError::new("graphics command called outside a frame"))?
        .draw_rect(x, y, width, height, color);
    Ok(())
}

fn host_end_frame(mut env: FunctionEnvMut<GraphicsState>) -> Result<(), RuntimeError> {
    let frame = env
        .data_mut()
        .active_frame
        .take()
        .ok_or_else(|| RuntimeError::new("graphics end_frame called without begin_frame"))?;
    env.data_mut().completed_frame = Some(frame);
    Ok(())
}

fn checked_color(red: f32, green: f32, blue: f32, alpha: f32) -> Result<[f32; 4], RuntimeError> {
    Ok([
        checked_finite("red", red)?.clamp(0.0, 1.0),
        checked_finite("green", green)?.clamp(0.0, 1.0),
        checked_finite("blue", blue)?.clamp(0.0, 1.0),
        checked_finite("alpha", alpha)?.clamp(0.0, 1.0),
    ])
}

fn checked_finite(name: &str, value: f32) -> Result<f32, RuntimeError> {
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| RuntimeError::new(format!("graphics {name} must be finite")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertises_only_graphics_imports() {
        assert!(GraphicsService::supports_import(
            wasm_abi::IMPORT_GRAPHICS_BEGIN_FRAME
        ));
        assert!(GraphicsService::supports_import(
            wasm_abi::IMPORT_GRAPHICS_DRAW_RECT
        ));
        assert!(!GraphicsService::supports_import("create_buffer"));
    }
}
