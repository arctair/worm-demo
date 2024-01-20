use bevy::prelude::shape::RegularPolygon;
use bevy::math::Vec2;
use std::f32::consts::PI;

pub trait Vertices {
    fn vertices(self) -> Vec<Vec2>;
}

impl Vertices for RegularPolygon {
    fn vertices(self) -> Vec<Vec2> {
        let mut result = vec![];
        for index in 0..self.sides {
            let angle = PI / self.sides as f32 + PI * index as f32 * 2. / self.sides as f32;
            result.push(self.radius * Vec2::new(f32::sin(angle), f32::cos(angle)));
        }
        result
    }
}
