use bevy::prelude::{Component, Transform};
use bevy::math::{Vec2, Vec3};
use bevy_rapier2d::na::Point2;
use bevy_rapier2d::parry::math::Real;

#[derive(Component)]
pub struct Geometry {
    local_vertices: Vec<Vec2>,
}

impl Geometry {
    pub fn new(local_vertices: Vec<Vec2>) -> Geometry {
        Geometry { local_vertices }
    }

    pub fn vertices(&self, transform: &Transform) -> Vec<Point2<Real>> {
        self.local_vertices.iter().map(|local_point_vec2| {
            let local_point_vec3 = Vec3::new(local_point_vec2.x, local_point_vec2.y, 0.);
            let global_point_vec3 = transform.transform_point(local_point_vec3);
            Point2::new(global_point_vec3.x, global_point_vec3.y)
        }).collect()
    }
}
