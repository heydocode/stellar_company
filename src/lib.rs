use bevy::prelude::*;

use crate::{objects::ObjectsPlugin, physics::PhysicsPlugin, view::ViewPlugin};

mod objects;
mod physics;
mod view;

pub struct SolarCompanyGameLib;

impl Plugin for SolarCompanyGameLib {
    fn build(&self, app: &mut App) {
        app.add_plugins((ViewPlugin, ObjectsPlugin, PhysicsPlugin));
    }
}
