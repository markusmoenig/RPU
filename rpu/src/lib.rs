pub mod rpu;

pub use crate::rpu::RPU as RPU;
pub use crate::rpu::buffer::Buffer as Buffer;

extern crate nalgebra_glm as glm;

pub type I = i32;
pub type F = f64;
pub type GF2 = glm::DVec2;
pub type GF3 = glm::DVec3;
pub type GF4 = glm::DVec4;

pub type Color = [F; 4];

pub mod prelude {
    pub use nalgebra::*;
    pub use crate::rpu::RPU;

    pub use crate::I;
    pub use crate::F;
    pub use crate::GF2;
    pub use crate::GF3;
    pub use crate::GF4;
    pub use crate::Color;

    pub use crate::rpu::buffer::Buffer;
    pub use crate::rpu::buffer::ColorBuffer;
    pub use crate::rpu::buffer::IndexBuffer3D;

    pub use crate::rpu::script::*;
    pub use crate::rpu::script::engine::ScriptEngine;
    pub use crate::rpu::element2d::Element2D;

    pub use crate::rpu::camera::*;
    pub use crate::rpu::camera::pinhole::Pinhole;

    pub use crate::rpu::sdf3d::SDF3D;
    pub use crate::rpu::sdf3d::sphere::SDF3DSphere;
    pub use crate::rpu::sdf3d::cube::SDF3DCube;

    pub use crate::rpu::analytical::Analytical;
    pub use crate::rpu::analytical::voxel::AnalyticalVoxel;
    // pub use crate::rpu::analytical::sphere::AnalyticalSphere;
    // pub use crate::rpu::analytical::cube::AnalyticalCube;

    pub use crate::rpu::layout3d::Layout3D;
    pub use crate::rpu::layout3d::grid3d::Grid3D;

    pub use crate::rpu::compiler::Compiler;
    pub use crate::rpu::compiler::RPUError;
    pub use crate::rpu::compiler::Property;
    pub use crate::rpu::compiler::scanner::Scanner;
    pub use crate::rpu::compiler::scanner::Token;
    pub use crate::rpu::compiler::context::Context;
    pub use crate::rpu::compiler::node::*;
    pub use crate::rpu::compiler::object::Object;
    pub use crate::rpu::compiler::context::*;

    pub use crate::rpu::renderer::Renderer;
    pub use crate::rpu::renderer::textured::Textured;

    pub use crate::rpu::element2d::UV;
    pub use crate::rpu::element2d::texture::Texture;
    pub use crate::rpu::element2d::vertical::Vertical;
    pub use crate::rpu::element2d::color::ColorElement;
    pub use crate::rpu::element2d::noise::Noise;
    pub use crate::rpu::element2d::bricks::Bricks;
    pub use crate::rpu::element2d::sprite::Sprite;

    pub use crate::rpu::hit::*;

    pub use std::path::PathBuf;
    pub use std::collections::HashMap;

    pub use rhai::{Engine, Scope, AST};

    pub use log::{debug, error, log_enabled, info, Level};
}

#[macro_use]
extern crate alloc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
