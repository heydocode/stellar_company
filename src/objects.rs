use bevy::{
    color::palettes::css::{BLUE, RED, WHITE},
    prelude::*,
};

use crate::definitions::{Mass, ObjectMarker, Position, Vec3f64, Velocity};

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bodies);
    }
}

fn spawn_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
commands.spawn((
    Mesh3d(meshes.add(Sphere::new(60.))),
    MeshMaterial3d(materials.add(Color::from(RED))),
    Transform::from_xyz(-500., 0., 0.),
    Position(Vec3f64::new(-500., 0., 0.), Vec3f64::ZERO),
    Mass(1e23),
    Velocity(Vec3f64::new(0., 0., 15.), Vec3f64::ZERO),
    Name::new("Red sphere"),
    ObjectMarker
));

commands.spawn((
    Mesh3d(meshes.add(Sphere::new(60.))),
    MeshMaterial3d(materials.add(Color::from(BLUE))),
    Transform::from_xyz(500., 0., 0.),
    Position(Vec3f64::new(500., 0., 0.), Vec3f64::ZERO),
    Mass(1e23),
    Velocity(Vec3f64::new(0., 0., -15.), Vec3f64::ZERO),
    Name::new("Blue sphere"),
    ObjectMarker
));

commands.spawn((
    Mesh3d(meshes.add(Sphere::new(40.))),
    MeshMaterial3d(materials.add(Color::from(WHITE))),
    Transform::from_xyz(0., 250., 0.),
    Position(Vec3f64::new(0., 250., 0.), Vec3f64::ZERO),
    Mass(1e21),
    Velocity(Vec3f64::new(15., 0., 0.), Vec3f64::ZERO),
    Name::new("White sphere"),
    ObjectMarker
));
}
