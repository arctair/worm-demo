use bevy::math::Vec2;
use bevy::prelude::{Component, Transform};

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) struct Polygon {
    pub(crate) vertices: Vec<Vec2>,
}

impl From<Vec<Vec2>> for Polygon {
    fn from(vertices: Vec<Vec2>) -> Self {
        Polygon { vertices }
    }
}

impl Polygon {
    pub(crate) fn to_global_space(&self, transform: &Transform) -> Polygon {
        let mut global_vertices = self.vertices.clone();
        for index in 0..global_vertices.len() {
            global_vertices[index] *= transform.scale.truncate();
            global_vertices[index] += transform.translation.truncate();
        }
        return Polygon::from(global_vertices);
    }

    pub(crate) fn to_local_space(&self, transform: Transform) -> Polygon {
        let mut local_vertices = self.vertices.clone();
        for index in 0..local_vertices.len() {
            local_vertices[index] -= transform.translation.truncate();
            local_vertices[index] /= transform.scale.truncate();
        }
        return Polygon::from(local_vertices);
    }
}

#[cfg(test)]
mod tests {}