use std::time::Duration;

use bevy::{
    color::palettes::css::{GREEN, ORANGE_RED},
    math::VectorSpace,
    platform::collections::HashMap,
    prelude::*,
    tasks::block_on,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use definitions::{
    bevy::{Radius, RequestedBodies, SearchBodyResponse},
    prelude::{
        InterpolatingObjects, JPLHorizonsBodySearch, MainCameraTracker, Mass, ObjectMarker, PhysicsDT, Position, SearchBody, SelectedFocusEntity, TimePaused, UiCameraTracker, UniversalG, Vec3f64, Velocity
    },
};

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PanOrbitCameraPlugin, EguiPlugin::default()));
        app.insert_resource(AmbientLight::default());
        app.insert_resource(InterpolatingObjects(true));
        app.insert_resource(SelectedFocusEntity(None));
        app.insert_resource(SearchBody(String::new()));
        app.add_systems(Startup, setup);
        app.add_systems(Update, (interpolate_objects, draw_velocity_gizmos, update_camera));
        app.add_systems(
            EguiPrimaryContextPass,
            (simulation_ui, camera_focus_ui),
        );
        app.add_systems(Update, drag_bodies);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PanOrbitCamera {
            // Panning the camera changes the focus, and so you most likely want to disable
            // panning when setting the focus manually
            pan_sensitivity: 0.0,
            // If you want to fully control the camera's focus, set smoothness to 0 so it
            // immediately snaps to that location. If you want the 'follow' to be smoothed,
            // leave this at default or set it to something between 0 and 1.
            pan_smoothness: 0.01,
            ..default()
        },
        Camera {
            order: 2,
            is_active: true,
            ..default()
        },
        Transform::from_xyz(0., 500., 0.).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("3D Camera (PanOrbit supervisor)"),
        MainCameraTracker,
    ));

    /*
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0,
            is_active: true,
            ..default()
        },
        Name::new("UI Camera"),
        UiCameraTracker,
    ));
    */
}

fn interpolate_objects(
    interpolating_objects: Res<InterpolatingObjects>,
    mut motion_q: Query<(&mut Transform, &Position), With<ObjectMarker>>,
    fixed_time: Res<Time<Fixed>>,
    time_paused: Res<TimePaused>,
) {
    let alpha = fixed_time.overstep_fraction();

    for (mut transform, pos) in &mut motion_q {
        if !interpolating_objects.0 || time_paused.0 {
            transform.translation = pos.0.into();
            continue;
        }
        let previous: Vec3 = pos.1.into();
        let current: Vec3 = pos.0.into();

        transform.translation = previous.lerp(current, alpha);
    }
}

fn draw_velocity_gizmos(
    interpolating_objects: Res<InterpolatingObjects>,
    mut gizmos: Gizmos,
    motion_q: Query<(&Transform, &Velocity, &Radius), With<ObjectMarker>>, // ← added Radius
    // mut dt: ResMut<PhysicsDT>,
    fixed_time: Res<Time<Fixed>>,
    time_paused: Res<TimePaused>,
) {
    let alpha = fixed_time.overstep_fraction();

    for (transform, velocity, radius) in motion_q.iter() {
        let center = transform.translation;
        let radius = radius.0;

        let vel_vec3 = if !interpolating_objects.0 || time_paused.0 {
            velocity.0.into()
        } else {
            let old_vel: Vec3 = velocity.1.into();
            let new_vel: Vec3 = velocity.0.into();
            old_vel.lerp(new_vel, alpha)
        };

        if vel_vec3.length_squared() < 0.0001 {
            continue;
        }

        let direction = vel_vec3.normalize();
        let start = center + direction * radius as f32;

        let end = start + vel_vec3;

        gizmos.arrow(start, end, ORANGE_RED);
    }
}

fn camera_focus_ui(
    mut contexts: EguiContexts,
    bodies_q: Query<(&Transform, &Name, Entity), With<ObjectMarker>>,
    mut selected_res: ResMut<SelectedFocusEntity>
) -> Result {
    egui::Window::new("Camera focus").scroll(true).show(
        contexts.ctx_mut()?,
        |ui: &mut egui::Ui| {
            let selected = &mut selected_res.0;
            egui::ComboBox::from_label("Select a body by name")
                .selected_text(format!("{:?}", selected))
                .show_ui(ui, |ui| {
                    for (_, name, entity) in bodies_q.iter() {
                        ui.selectable_value(&mut *selected, Some(entity), format!("{}", name));
                    }
                });
        },
    );
    Ok(())
}

fn update_camera(mut selected: ResMut<SelectedFocusEntity>, bodies_q: Query<(&Transform, &Name, Entity)>,
mut pan_orbit_q: Query<&mut PanOrbitCamera>,) {
    if let Some(entity) = selected.0 {
        if let Ok(mut pan_orbit) = pan_orbit_q.single_mut() {
            for (transform, _, body_entity) in bodies_q.iter() {
                if entity == body_entity {
                    pan_orbit.target_focus = transform.translation;
                    // Whenever changing properties manually like this, it's necessary to force
                    // PanOrbitCamera to update this frame (by default it only updates when there are
                    // input events).
                    pan_orbit.force_update = true;
                }
            }
        }
    }
}

fn simulation_ui(
    mut contexts: EguiContexts,
    mut bodies_q: Query<(
        &Transform,
        &mut Position,
        &mut Velocity,
        &mut Mass,
        &mut Name,
        Entity,
    )>,
    mut universal_g: ResMut<UniversalG>,
    mut physics_timer: ResMut<Time<Fixed>>,
    mut wanted_timer_time: Local<f64>,
    mut name_buf: Local<HashMap<Entity, String>>,
    mut mass_buf: Local<HashMap<Entity, f64>>,
    mut dt: ResMut<PhysicsDT>,
    mut interpolating_objects: ResMut<InterpolatingObjects>,
    mut time_paused: ResMut<TimePaused>,
    mut selected: Local<String>,
) -> Result {
    let bodies_copy: Vec<(&Transform, &Position, &Velocity, &Mass, &Name, Entity)> =
        bodies_q.iter().collect();
    egui::Window::new("PARAMETERS")
        .scroll(true)
        .show(contexts.ctx_mut()?, |ui: &mut egui::Ui| {
            ui.label("Please choose a section to work with");
            egui::CollapsingHeader::new("TWEAKING PARAMETERS").show(ui, |ui: &mut egui::Ui| {
                ui.label("Note: Position and Velocity can be easily changed by dragging the bodies with the mouse cursor or with fingers <NOT IMPLEMENTED YET>");
                for (mut transform, mut position, mut velocity, mut mass, mut name, mut entity) in
                    bodies_q.iter_mut()
                {
                    ui.separator();
                    ui.label(format!("Entity ID: {:#?}", entity));

                    if name_buf.get(&entity).is_none() {
                        name_buf.insert(entity, String::from(&*name));
                    }
                    if let Some(name_buf) = name_buf.get_mut(&entity) {
                        let response = ui.text_edit_singleline(name_buf);
                        if response.changed() {
                            name.set(name_buf.clone());
                        }
                    }

                    if mass_buf.get(&entity).is_none() {
                        mass_buf.insert(entity, mass.0);
                    }
                    if let Some(mass_buf) = mass_buf.get_mut(&entity) {
                        let response = ui.add(egui::DragValue::new(mass_buf));
                        if response.changed() {
                            mass.0 = *mass_buf;
                        }
                    }
                }

                ui.add(egui::DragValue::new(&mut dt.0));
                ui.checkbox(&mut interpolating_objects.0, "Interpolate objects");

                ui.separator();
                ui.add(egui::Label::new("TIME & CONSTANTS"));

                let response = ui.add(
                    egui::DragValue::new(&mut *wanted_timer_time)
                    .speed(0.01)
                    // TODO Re-change back to 0.01 later, and add proper time step tweaking
                    .range(0.03..=f64::INFINITY)
                    .prefix("Physics execution frequency (in seconds): "),
                );
                let wanted_time_duration = Duration::from_millis((*wanted_timer_time * 1e3) as u64);
                /*
                if wanted_time_duration.as_millis() < 1 {
                    ui.add(egui::Label::new("The physics execution timer can't be less than 0.001s (that's computationnally nonsense)"));
                    physics_timer.set_timestep(Duration::from_mins(5));
                    ui.add(egui::Label::new(format!("Current timer value: {}ms", physics_timer.timestep().as_millis())));
                    }
                    */
                    if response.changed() {
                        physics_timer.set_timestep(wanted_time_duration);
                    }
                    ui.add(
                        egui::DragValue::new(&mut universal_g.0)
                        .speed(1e-15)
                        .prefix("G: "),
                    );
                    let response = ui.add(egui::Button::new("Reset the Universal G value to default"));

                    if response.clicked() {
                        *universal_g = UniversalG::default();
                    }

                    ui.checkbox(&mut time_paused.0, "Time paused");
                });
            });
    /*

    ui.label(format!(
        "Position: \nx: {}\ny: {}\nz: {}",
        position.0.x, position.0.y, position.0.z
    ));
    ui.label(format!(
        "Velocity: \nx: {}\ny: {}\nz: {}",
        velocity.0.x, velocity.0.y, velocity.0.z
    ));
    ui.label(format!(
        "Actual Interpolated Position: {:#?}",
        transform.translation
        ));

        */
    Ok(())
}

fn drag_bodies(time_paused: Res<TimePaused>) {
    if !time_paused.0 {
        return;
    }
}
