//! BEVY-DEPENDENT DEFINITIONS

use bevy::{ecs::component::{Mutable, StorageType}, prelude::*};
use crate::standalone::{Acceleration, Mass, Position, Vec3f64, Velocity};

#[derive(Component)]
pub struct UiCameraTracker;

#[derive(Component)]
pub struct MainCameraTracker;

#[derive(Resource, Debug)]
pub struct UniversalG(pub f64);

impl Default for UniversalG {
    fn default() -> Self {
        Self(6.6743e-11)
    }
}

#[derive(Component)]
pub struct ObjectMarker;

/// The number that is used as DT at each position & velocity calculation
#[derive(Resource)]
pub struct PhysicsDT(pub f64);

#[derive(Resource)]
pub struct TimePaused(pub bool);

#[derive(Resource)]
pub struct InterpolatingObjects(pub bool);

impl Into<Vec3> for Vec3f64 {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

impl Component for Mass {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Mutable;
}

impl Component for Position {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Mutable;
}

impl Component for Velocity {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Mutable;
}

impl Component for Acceleration {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Mutable;
}