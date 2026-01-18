use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    tasks::Task,
    tasks::futures::check_ready,
};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use fetch_space_bodies::{get_body_motion, get_body_properties, search_bodies};
use shared::{
    bevy::{
        ClientRes, ObjectMarker, Radius, RequestedBodies, RetrieveBody, SearchBody,
        SearchBodyResponse,
    },
    prelude::{Mass, Position, Velocity},
};
use tokio::runtime::Runtime;

pub struct AsyncPlugin;

impl Plugin for AsyncPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RequestedBodies(vec![]));
        app.insert_resource(SearchBodyResponse(vec![]));
        app.add_systems(
            Update,
            (fetch_requested_and_searched_bodies, handle_tasks).chain(),
        );
    }
}

fn handle_tasks(
    mut commands: Commands,
    mut transform_tasks: Query<(Entity, &mut RetrieveBody)>,
    obj_radius: Query<&Radius, With<ObjectMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ((entity, mut task), radius) in &mut transform_tasks.iter_mut().zip(obj_radius.iter()) {
        // Use `check_ready` to efficiently poll the task without blocking the main thread.
        if let Some(mut commands_queue) = check_ready(&mut task.0) {
            // Append the returned command queue to execute it later.
            commands.append(&mut commands_queue);
            // Task is complete, so remove the task component from the entity.
            commands.entity(entity).remove::<RetrieveBody>();
        }
        commands
            .entity(entity)
            .insert(Mesh3d(meshes.add(Sphere::new(radius.0 as f32))))
            .insert(MeshMaterial3d(
                materials.add(Color::from(bevy::color::palettes::css::GREEN)),
            ));
    }
}

fn fetch_requested_and_searched_bodies(
    mut commands: Commands,
    client: Res<ClientRes>,
    requested_bodies: Res<RequestedBodies>,
    search_string: Res<SearchBody>,
    mut previous_search: Local<String>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let client = client.clone();
    let entity = commands.spawn_empty().id();
    let search_string_v = search_string.0.clone();
    let previous_search_v = previous_search.clone();
    let requested_bodies_copy: Vec<i64> = requested_bodies.0.iter().cloned().collect();
    let task = thread_pool.spawn(async move {
        let runtime = Runtime::new().unwrap();
        let join = runtime.spawn(async move {
        let mut searched_bodies = Vec::new();
        if search_string_v != previous_search_v {
                if let Some(bodies_list) = search_bodies(client.0.clone(), search_string_v.as_str()).await {
                    searched_bodies = bodies_list;
                }
        }

        // Radius, Position, Velocity, Mass, id
        let mut bundle: Option<(Radius, Position, Velocity, Mass, i64)> = None;
        // Spawn bodies
        for id in requested_bodies_copy.iter() {
                if let Some((position, velocity)) = get_body_motion(client.0.clone(), *id).await {
                    if let Some((mass, radius)) = get_body_properties(client.0.clone(), *id).await {
                        bundle = Some((
                            Radius(radius),
                            position,
                            velocity,
                            mass,
                            *id
                        ));
                    } else {
                        error!(
                            "Unable to find body {} physic properties (mass & radius)\nPlease search another Body, if possible, a major one (e.g. Earth, Mars, Sun)",
                            id
                        );
                    }
                } else {
                    error!(
                        "Unable to find body {} motion parameters (position & velocity)",
                        id
                    );
                }


        }

        let mut command_queue = CommandQueue::default();

        // we use a raw command queue to pass a FnOnce(&mut World) back to be
        // applied in a deferred manner.
        command_queue.push(move |world: &mut World| {
            if let Some(bundle) = bundle {
                        world.entity_mut(entity)
                            .insert(bundle.0)
                            .insert(bundle.1)
                            .insert(bundle.2)
                            .insert(bundle.3);


                if !searched_bodies.is_empty() {
                    if let Some(mut res) = world.get_resource_mut::<SearchBodyResponse>() {
                        res.0 = searched_bodies;
                    }
                }
            }
            else {
                return;
            }
        });
        command_queue

    });
        join.await.ok().unwrap()
    });

    commands.entity(entity).insert(RetrieveBody(task));
    *previous_search = search_string.0.clone();
}
