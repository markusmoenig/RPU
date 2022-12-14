use std::collections::HashMap;

use crate::prelude::*;
use rayon::{slice::ParallelSliceMut, iter::{IndexedParallelIterator, ParallelIterator}};

pub struct Context {
    pub nodes                   : Vec<Node>,

    pub textures                : Vec<usize>,
    pub objects                 : Vec<usize>,
    pub sprites                 : Vec<Box<dyn Element2D>>,

    pub layouts                 : Vec<Object>,

    pub symbols_node_index      : HashMap<char, usize>,

    pub renderer                : Box<dyn Renderer>,
    pub camera                  : Box<dyn Camera3D>,

    pub size                    : [usize; 2],

    pub out_texture             : Option<usize>,
}

impl Context {

    pub fn new() -> Self {
        Self {
            nodes               : vec![],

            textures            : vec![],
            objects             : vec![],
            sprites             : vec![],

            layouts             : vec![],
            symbols_node_index  : HashMap::new(),

            renderer            : Box::new(Textured::new()),
            camera              : Box::new(Pinhole::new()),

            size                : [0, 0],

            out_texture         : None,
        }
    }

    pub fn init(&mut self) {
        // Render textures

        /*
        for node_index in &self.textures {
            match &mut self.nodes[*node_index].object {
                Object::Element2D(el) => {
                    el.render(*node_index, &self);
                },
                _ => {}
            }
        }*/
    }

    pub fn update(&mut self) {
        self.camera.update();
        for o in &mut self.nodes {
            match &mut o.object {
                Object::AnalyticalObject(object) => {
                    object.update();
                },
                _ => {},
            }
        }
    }

    pub fn render_distributed(&mut self, color: &mut ColorBuffer<F>) {
        self.size = color.size.clone();
        let [width, height] = color.size;

        self.update();

        const LINES: usize = 20;
        let ratio = width as F / height as F;

        // let num_objects = self.bvh_nodes.len();
        // println!("num {}", num_objects);

        color.pixels
            .par_rchunks_exact_mut(width * LINES * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = (LINES - j - 1) * width * LINES + i;
                    let x = (i % width) as F;
                    let y = (i / width) as F;

                    let xx = x as F / width as F;
                    let yy = y as F / height as F;

                    let coord = Vector2::new((xx - 0.5) * ratio, (1.0 - yy) - 0.5);

                    let ray = self.camera.gen_ray(coord);
                    let mut hit = false;

                    if let Some(texture) = self.out_texture {
                        let index = self.textures[texture];
                        if let Some(c) = self.get_color(&ray,&[x as usize, y as usize], &color.size, &self.nodes[index].object, index) {
                            pixel.copy_from_slice(&c);
                        }
                        hit = true;
                    } else
                    if let Some(layout) = self.layouts.last() {
                        if let Some(c) = self.get_color(&ray,&[x as usize, y as usize], &color.size, &layout, 0) {
                            pixel.copy_from_slice(&c);
                            hit = true;
                        }
                    } /*else {
                        for i in 0..self.textures.len() {
                            if let Some(c) = self.get_color(&ray,&[x as usize, y as usize], &color.size, &self.nodes[i].object) {
                                pixel.copy_from_slice(&c);
                                hit = true;
                                break;
                            }
                        }
                    }*/

                    if hit == false {
                        let c = [0.0, 0.0, 0.0, 1.0];
                        pixel.copy_from_slice(&c);
                    }
                }
            });
    }

    /*
    pub fn render(&mut self, camera: &Box<dyn Camera3D>, color: &mut ByteBuffer, _depth: &mut Buffer<f32>) {

        self.update();

        let [width, height] = color.size;
        let ratio = width as F / height as F;

        for y in 0..height {
            for x in 0..width {
                let i = y * 4 * width + x * 4;
                let xx = x as F / width as F;
                let yy = y as F / height as F;
                let coord = Vector2::new((xx - 0.5) * ratio, yy - 0.5);

                let ray = camera.gen_ray(coord);

                let c = self.get_color(&ray,&[x, y], &color.size, &self.root);
                color.pixels[i..i + 4].copy_from_slice(&c);
            }
        }
    }*/

    #[inline(always)]
    fn get_color(&self, ray: &[Vector3<F>; 2], p: &[usize; 2], size: &[usize;2], object: &Object, index: usize) -> Option<Color> {
        let mut c = [0.0, 0.0, 0.0, 1.0];

        match object {
            /*
            Object::AnalyticalObject(object) => {
                if let Some(hit) = object.get_distance(&ray) {

                    let tex_index= 0_usize;
                    match &self.textures[tex_index] {
                        Object::Element2D(el) => {
                            let uv = hit.uv;
                            c = el.get_color_at(&[uv.x, -uv.y]);
                        },
                        _ => {},
                    }
                } else {
                    return None;
                }
            },*/
            Object::SDF3D(object) => {

                let [ro, rd] = ray;
                let mut t = 0.01;
                let translate = Vector3::new(0.0, 0.0, 0.0);
                for _i in 0..12 {
                    let p = ro + rd * t;
                    let d = object.get_distance(&p, &translate);
                    if d < 0.001 {
                        c[0] = 1.0;
                        return Some(c);
                    }
                    t += d;

                    if t > 5.0 {
                        return None;
                    }
                }
            },
            /*
            Object::Layout3D(_layout) => {
                self.renderer.render(ray, object, &self);
                /*
                if let Some(hit) = layout.traverse3d(&ray, &self) {

                    c[0] = 255;
                    /*
                    let tex_index= 0_usize;
                    match &self.textures[tex_index] {
                        Object::Element2D(el) => {
                            let uv = hit.uv;
                            c = el.get_color_at(&[uv.x, -uv.y]);
                        },
                        _ => {},
                    }*/
                } else {
                    return None;
                }*/
            },*/
            Object::Element2D(element) => {
                let [width, height]= size;
                let [x, y]= p;

                let xx = (*x as F / *width as F) - 0.5;
                let yy = 0.5 - ((*y as F / *height as F));

                let mut uv = UV::new(GF2::new(xx, yy), GF4::new(0.0, 0.0, *width as F, *height as F), GF2::new(xx, yy));
                let v = element.get_color_at(&mut uv, index, &self);
                c[0] = v[0];
                c[1] = v[1];
                c[2] = v[2];
                c[3] = v[3];
            },
            _ => {
                let v = self.renderer.render(ray, object, &self);
                c[0] = v[0];
                c[1] = v[1];
                c[2] = v[2];
                c[3] = v[3];
            }
        }
        Some(c)
    }

}