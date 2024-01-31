use bevy::math::Vec2;
use bevy::prelude::{Bundle, Color, Component, Gizmos, Query};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{Collider, CollisionGroups};
use bevy_rapier2d::prelude::Group;

#[derive(Bundle)]
pub(crate) struct PolylineBundle {
    collider: Collider,
    collision_groups: CollisionGroups,
    polyline: Polyline,
    rigid_body: RigidBody,
}

impl From<Vec<Vec2>> for PolylineBundle {
    fn from(points: Vec<Vec2>) -> Self {
        let polyline = Polyline::from(points);
        PolylineBundle {
            collider: polyline.collider(),
            collision_groups: CollisionGroups::new(Group::GROUP_1, Group::ALL),
            polyline,
            rigid_body: RigidBody::Fixed,
        }
    }
}

#[derive(Component)]
pub(crate) struct Polyline {
    pub(crate) points: Vec<Vec2>,
    pub(crate) version: usize,
}

impl Polyline {
    pub(crate) fn collider(&self) -> Collider {
        Collider::polyline(self.points.clone(), None)
    }
}

impl From<Vec<Vec2>> for Polyline {
    fn from(points: Vec<Vec2>) -> Self {
        Polyline {
            points,
            version: 0,
        }
    }
}

pub(crate) fn polyline_gizmo(
    query: Query<&Polyline>,
    mut gizmos: Gizmos,
) {
    for polyline in query.iter() {
        for point in &polyline.points {
            gizmos.circle_2d(*point, 1. / 4., Color::MAROON);
        }
    }
}
