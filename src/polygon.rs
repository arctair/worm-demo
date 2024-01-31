use bevy::math::Vec2;
use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Polygon {
    pub(crate) vertices: Vec<Vec2>,
}

impl From<Vec<Vec2>> for Polygon {
    fn from(vertices: Vec<Vec2>) -> Self {
        Polygon { vertices }
    }
}
