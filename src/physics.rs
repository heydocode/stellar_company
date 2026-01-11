use bevy::{platform::collections::HashMap, prelude::*};

use shared::prelude::{
    Acceleration, Mass, ObjectMarker, PhysicsDT, Position, TimePaused, UniversalG, Vec3f64,
    Velocity,
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(0.03));
        app.insert_resource(UniversalG::default());
        app.insert_resource(PhysicsDT(0.15));
        app.insert_resource(TimePaused(false));
        app.add_systems(FixedUpdate, simplectic_euler);
    }
}

fn simplectic_euler(
    mut bodies_q: Query<(&mut Position, &mut Velocity, &Mass, Entity), With<ObjectMarker>>,
    universal_g: Res<UniversalG>,
    dt: Res<PhysicsDT>,
    time_paused: Res<TimePaused>,
) {
    if time_paused.0 {
        return;
    }
    let bodies_copy: Vec<(&Position, &Velocity, &Mass, Entity)> = bodies_q.iter().collect();
    let mut accelerations: HashMap<Entity, Acceleration> = HashMap::new();
    for (position_outer, _, _, entity_outer) in bodies_copy.iter() {
        let mut acceleration: Acceleration = Acceleration(Vec3f64::ZERO);
        for (position_inner, _, mass_inner, entity_inner) in bodies_copy.iter() {
            // Skip self
            if entity_outer == entity_inner {
                continue;
            }
            acceleration.0 += calculate_acceleration(
                position_outer.0,
                position_inner.0,
                mass_inner,
                universal_g.0,
            )
            .0;
        }
        accelerations.insert(*entity_outer, acceleration);
    }
    for (index, (mut position, mut velocity, _, entity)) in bodies_q.iter_mut().enumerate() {
        let previous_position = position.0;
        let previous_velocity = velocity.0;

        position.1 = previous_position;
        velocity.1 = previous_velocity;

        velocity.0 = previous_velocity
            + accelerations
                .get(&entity)
                .expect("Entity should be present")
                .0
                * dt.0;
        position.0 = previous_position + velocity.0 * dt.0;
    }
}

fn calculate_acceleration(pos_a: Vec3f64, pos_b: Vec3f64, mass_b: &Mass, g: f64) -> Acceleration {
    let d = pos_b - pos_a;
    let distance = d.length();

    // avoid division by zero
    if distance == 0.0 {
        return Acceleration(Vec3f64::ZERO);
    }

    let accel = d * g * mass_b.0 / distance.powi(3);

    Acceleration(accel)
}
