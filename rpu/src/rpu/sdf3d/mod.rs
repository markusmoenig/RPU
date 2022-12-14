pub mod sphere;
pub mod cube;

use crate::prelude::*;

pub trait SDF3D : Sync + Send + Script {
    fn new() -> Self where Self: Sized;

    fn get_distance(&self, x: &Vector3<F>, instance: &Vector3<F>) -> F;
    fn get_normal(&self, x: &Vector3<F>, instance: &Vector3<F>) -> Vector3<F> {

        let e = Vector2::new(1.0,-1.0)*0.5773*0.0005;

        // IQs normal function

        let mut n = e.xyy() * self.get_distance(&(x + e.xyy()), instance);
        n += e.yyx() * self.get_distance(&(x + e.yyx()), instance);
        n += e.yxy() * self.get_distance(&(x + e.yxy()), instance);
        n += e.xxx() * self.get_distance(&(x + e.xxx()), instance);
        n.normalize()
    }
}