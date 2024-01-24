use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::{Camera2dBundle, Commands, OrthographicProjection};
use bevy::utils::default;
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierDebugRenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.,
            ..default()
        },
        ..default()
    });
}