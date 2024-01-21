use bevy::prelude::{Component, Transform};
use bevy::math::{Vec2, Vec3};
use bevy_rapier2d::na::Point2;
use bevy_rapier2d::parry::math::Real;
use bevy_rapier2d::parry::transformation::convex_polygons_intersection_points;

#[derive(Component)]
pub struct Geometry {
    local_vertices: Vec<Vec2>,
}

impl Geometry {
    pub fn new(local_vertices: Vec<Vec2>) -> Geometry {
        Geometry { local_vertices }
    }

    pub fn subtract((t0, g0): (&Transform, &Geometry), (t1, g1): (&Transform, &Geometry)) -> Vec<Point2<Real>> {
        let mut result = vec![];
        let intersection: &mut Vec<Point2<Real>> = &mut vec![];
        convex_polygons_intersection_points(&g0.vertices(&t0), &g1.vertices(&t1), intersection);
        for intersection_vertex in intersection {
            result.push(*intersection_vertex)
        }
        result
    }

    fn vertices(&self, transform: &Transform) -> Vec<Point2<Real>> {
        self.local_vertices.iter().map(|local_point_vec2| {
            let local_point_vec3 = Vec3::new(local_point_vec2.x, local_point_vec2.y, 0.);
            let global_point_vec3 = transform.transform_point(local_point_vec3);
            Point2::new(global_point_vec3.x, global_point_vec3.y)
        }).collect()
    }
}
