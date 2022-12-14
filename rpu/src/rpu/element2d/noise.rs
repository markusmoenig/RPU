use crate::prelude::*;

pub struct Noise<'a> {
    engine                  : ScriptEngine<'a>,
    color                   : GF4,
    scale                   : GF2,
}

impl Element2D for Noise<'_> {
    fn new() -> Self {

        let engine = ScriptEngine::new();

        Self {
            engine,
            color           : Vector4::new(1.0, 1.0, 1.0, 1.0),
            scale           : GF2::new(1.0, 1.0),
        }
    }

    fn name(&self) -> String {
        "Noise".to_string()
    }

    fn compute_color_at(&self, uv : &UV, color: &mut GF4, _node: usize, ctx: &Context) {
        use noise::{NoiseFn, Perlin};

        let mut uv_local = uv.world + GF2::new(10000.0, 10000.0);
        let rr = ctx.size[0] as F / ctx.size[1] as F;
        uv_local.x *= rr;
        uv_local.y *= rr;

        let value = Perlin::new();
        let v = value.get([uv_local.x * 20.0 / self.scale.x, uv_local.y * 20.0 / self.scale.y]);

        //println!("{}", v);
        *color = glm::mix(&color, &self.color, (self.color.w * (v / 2.0 + 0.5)).clamp(0.0, 1.0));

        //*color = GF4::new(v, v, v, v);

        self.engine.execute_shader(uv, color);
    }
}

impl Script for Noise<'_> {

    fn get_scope<'a>(&mut self) -> &'a Scope {
        self.engine.get_scope()
    }

    fn get_engine<'a>(&self) -> &'a ScriptEngine {
        &self.engine
    }

    fn apply_properties(&mut self, props: Vec<Property>) -> Result<(), RPUError> {
        let rc = self.engine.apply_properties(props);
        if let Some(color) = self.engine.get_vector4("color") {
            self.color = color;
        }
        if let Some(scale) = self.engine.get_vector2("scale") {
            self.scale = scale;
        }
        rc
    }

    fn execute(&mut self, code: String) {
        self.engine.execute(code);
    }

    fn set_code_block(&mut self, name: String, code: String) {
        _ = self.engine.set_code_block(name, code);
    }
}
