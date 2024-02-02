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
            global_vertices[index] = (transform.rotation * global_vertices[index].extend(0.)).truncate();
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
            local_vertices[index] = (transform.rotation.inverse() * local_vertices[index].extend(0.)).truncate();
        }
        return Polygon::from(local_vertices);
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;
    use bevy::math::{Quat, Vec2, Vec3};
    use bevy::prelude::Transform;
    use crate::polygon::Polygon;

    #[test]
    fn test_translation_scale() {
        let local = Polygon::from(vec![
            Vec2::new(0., 0.),
            Vec2::new(0., 1.),
            Vec2::new(1., 0.),
        ]);

        let global = Polygon::from(vec![
            Vec2::new(0. * 8. + 2., 0. * 16. + 4.),
            Vec2::new(0. * 8. + 2., 1. * 16. + 4.),
            Vec2::new(1. * 8. + 2., 0. * 16. + 4.),
        ]);

        let transform = Transform::from_xyz(2., 4., 0.)
            .with_scale(Vec3::new(8., 16., 0.));

        let actual_global = local.to_global_space(&transform);
        let actual_local = global.to_local_space(transform);

        assert_eq!(actual_global, global);
        assert_eq!(actual_local, local);
    }

    #[test]
    fn test_scale_rotation() {
        let local = Polygon::from(vec![
            Vec2::new(0., 0.),
            Vec2::new(0., 1.),
            Vec2::new(1., 0.),
        ]);

        let global = Polygon::from(vec![
            (Quat::from_rotation_z(PI) * Vec2::new(0., 0.).extend(0.)).truncate() * Vec2::new(2., 4.),
            (Quat::from_rotation_z(PI) * Vec2::new(0., 1.).extend(0.)).truncate() * Vec2::new(2., 4.),
            (Quat::from_rotation_z(PI) * Vec2::new(1., 0.).extend(0.)).truncate() * Vec2::new(2., 4.),
        ]);

        let transform = Transform::from_scale(Vec3::new(2., 4., 0.))
            .with_rotation(Quat::from_rotation_z(PI));

        let actual_global = local.to_global_space(&transform);
        let actual_local = global.to_local_space(transform);

        assert_eq!(actual_global, global);
        assert_eq!(actual_local, local);
    }

    #[test]
    fn test_translation_rotation() {
        let local = Polygon::from(vec![
            Vec2::new(0., 0.),
            Vec2::new(8.742278e-8, 1.),
            Vec2::new(1., -8.742278e-8),
        ]);

        let global = Polygon::from(vec![
            (Quat::from_rotation_z(PI) * Vec2::new(0., 0.).extend(0.)).truncate() + Vec2::new(2., 4.),
            (Quat::from_rotation_z(PI) * Vec2::new(0., 1.).extend(0.)).truncate() + Vec2::new(2., 4.),
            (Quat::from_rotation_z(PI) * Vec2::new(1., 0.).extend(0.)).truncate() + Vec2::new(2., 4.),
        ]);

        let transform = Transform::from_xyz(2., 4., 0.)
            .with_rotation(Quat::from_rotation_z(PI));

        let actual_global = local.to_global_space(&transform);
        let actual_local = global.to_local_space(transform);

        assert_eq!(actual_global, global);
        assert_eq!(actual_local, local);
    }
}