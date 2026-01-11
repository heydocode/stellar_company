use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3f64 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

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

/// I really don't think these four information must be regrouped in a struct,
/// but we'll see later. TODO
#[derive(Default, Debug, PartialEq)]
pub struct JPLHorizonsBodySearch {
    pub id: i64,
    pub name: String,
    pub designation: String,
    pub other: String
}

pub struct ObjectDefinition {
    pub pos: Position,
    pub vel: Velocity,
    pub mass: Mass,
}

/// Mass in kg
#[derive(Debug)]
pub struct Mass(pub f64);

/// Stores 2 Positions:
/// - actual Position (index: 0)
/// - last Position (index: 1) (used for interpolation)
#[derive(Debug, PartialEq)]
pub struct Position(pub Vec3f64, pub Vec3f64);

/// Wrapper of Vec3 representing m/s*s velocity
/// Stores 2 Velocities:
/// - actual Velocity (index: 0)
/// - last Velocity (index: 1) (used for interpolation)
#[derive(Debug, PartialEq)]
pub struct Velocity(pub Vec3f64, pub Vec3f64);

pub struct Acceleration(pub Vec3f64);

impl Neg for Acceleration {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}