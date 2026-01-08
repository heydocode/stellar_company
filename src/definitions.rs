use bevy::prelude::*;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vec3f64 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Resource)]
pub struct InterpolatingObjects(pub bool);

impl Vec3f64 {
    pub const ZERO: Self = Vec3f64 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, rhs: Self) -> f64 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }
}

impl Mul<f64> for Vec3f64 {
    type Output = Vec3f64;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f64> for Vec3f64 {
    type Output = Vec3f64;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Add for Vec3f64 {
    type Output = Vec3f64;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3f64 {
    type Output = Vec3f64;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Neg for Vec3f64 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z
        }
    }
}

impl AddAssign for Vec3f64 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Into<Vec3> for Vec3f64 {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }
}

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

/// Mass in kg
#[derive(Component, Debug)]
pub struct Mass(pub f64);

/// Stores 2 Positions:
/// - actual Position (index: 0)
/// - last Position (index: 1) (used for interpolation)
#[derive(Component, Debug)]
pub struct Position(pub Vec3f64, pub Vec3f64);

/// Wrapper of Vec3 representing m/s*s velocity
/// Stores 2 Velocitues:
/// - actual Velocity (index: 0)
/// - last Velocity (index: 1) (used for interpolation)
#[derive(Component, Debug)]
pub struct Velocity(pub Vec3f64, pub Vec3f64);

#[derive(Component)]
pub struct Acceleration(pub Vec3f64);

impl Neg for Acceleration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[derive(Component)]
pub struct ObjectMarker;

pub struct ObjectDefinition {
    pos: Position,
    vel: Velocity,
    mass: Mass,
}

/// The number that is used as DT at each position & velocity calculation
#[derive(Resource)]
pub struct PhysicsDT(pub f64);

#[derive(Resource)]
pub struct TimePaused(pub bool);