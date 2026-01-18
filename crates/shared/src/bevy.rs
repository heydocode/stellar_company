//! BEVY-DEPENDENT DEFINITIONS

use crate::{prelude::JPLHorizonsBodySearch, standalone::{Acceleration, Mass, Position, Vec3f64, Velocity}};
use bevy::{
    ecs::component::{Mutable, StorageType},
    prelude::*,
    render::extract_resource::ExtractResource,
};
use bevy::{ecs::world::CommandQueue, tasks::Task};
use reqwest::Client;

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

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct ClientRes(pub Client);

impl ClientRes {
    pub fn insert_res(app: &mut App) {
        let client = Client::builder().user_agent("curl/7.79.1").build().unwrap();
        app.insert_resource(Self(client));
    }
}

#[derive(Resource)]
pub struct SelectedFocusEntity(pub Option<Entity>);

#[derive(Component, Clone, Copy)]
pub struct Radius(pub f64);

/// Status of the async retrieval of body datas from NASA JPL Horizons
#[derive(Component)]
pub struct RetrieveBody(pub Task<CommandQueue>);

/// Resource containing bodies that are not spawned yet (that are not processed yet
/// in the meaning of not requested data yet from JPL Horizons API).
#[derive(Resource)]
pub struct RequestedBodies(pub Vec<i64>);

/// The string the async utils need to search in JPL Horizons
#[derive(Resource)]
pub struct SearchBody(pub String);

#[derive(Resource)]
pub struct SearchBodyResponse(pub Vec<JPLHorizonsBodySearch>);
