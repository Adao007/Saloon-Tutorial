use avian2d::{math::*, prelude::*}; 
use bevy::{ecs::entity::EntityHashSet, prelude::*};
use serde::{Deserialize, Serialize};
use crate::gameplay::inventory::inventory::Inventory;
use super::{
    stamina::Stamina,
    player::{Player, PlayerStatus, Status},
};

const DIRECTION: f32 = 1.0; 
const RUN_SPEED: f32 = 170.0;
const STAMINA_DRAIN: f32 = 0.25; 
const EMPTY: f32 = 0.0;

// --- COMPONENTS --- 
#[derive(Component)]
pub struct Velocity {
    pub linvel: Vec3,
}

#[derive(Component)]
pub struct Speed {
    pub base: f32, 
    pub current: f32, 
}

// Entities touched during the last "move and slide" call. Stored for debug printing. 
#[derive(Component, Default, Deref, DerefMut, Reflect, Deserialize, Serialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct TouchedEntities(EntityHashSet);

#[derive(Component)]
pub struct DebugText; 

// --- SYSTEMS ---
pub fn movement(
    mut query: Query<(&mut Velocity, &Speed), With<Player>>, 
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (mut velocity, speed) in &mut query {
        let mut direction = Vec3::ZERO; 

        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += DIRECTION; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= DIRECTION; }
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= DIRECTION; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += DIRECTION; }

        if direction.length() > 0.0 {
            direction = direction.normalize(); 
            velocity.linvel = direction * speed.current; 
        } 
        else {
            velocity.linvel = Vec3::ZERO; 
        }
    }
}

/// System to handle player movement and friction.
///
/// This only updates velocity. The actual movement is handled by the `run_move_and_slide` system.
pub fn player_movement(
    mut query: Query<&mut LinearVelocity, With<Player>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for mut lin_vel in &mut query {
        // Determine movement velocity from input
        let mut movement_velocity = Vec2::ZERO;
        if input.pressed(KeyCode::KeyW) {
            movement_velocity += Vec2::Y
        }
        if input.pressed(KeyCode::KeyS) {
            movement_velocity += Vec2::NEG_Y
        }
        if input.pressed(KeyCode::KeyA) {
            movement_velocity += Vec2::NEG_X
        }
        if input.pressed(KeyCode::KeyD) {
            movement_velocity += Vec2::X
        }
        movement_velocity = movement_velocity.normalize_or_zero();
        movement_velocity *= 100.0;
        if input.pressed(KeyCode::ShiftLeft) {
            movement_velocity *= 2.0;
        }

        // Add to current velocity
        lin_vel.0 += movement_velocity.adjust_precision();

        let current_speed = lin_vel.length();
        if current_speed > 0.0 {
            // Apply friction
            lin_vel.0 = lin_vel.0 / current_speed
                * (current_speed - current_speed * 20.0 * time.delta_secs().adjust_precision())
                    .max(0.0)
        }
    }
}

/// System to run the move and slide algorithm, updating the player's transform and velocity.
///
/// This replaces Avian's default "position integration" that moves kinematic bodies based on their
/// velocity without any collision handling.
fn run_move_and_slide(
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut LinearVelocity,
            &mut TouchedEntities,
            &Collider,
        ),
        With<Player>,
    >,
    move_and_slide: MoveAndSlide,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (entity, mut transform, mut lin_vel, mut touched, collider) in &mut query {
        touched.clear();

        // Perform move and slide
        let MoveAndSlideOutput {
            position,
            projected_velocity,
        } = move_and_slide.move_and_slide(
            collider,
            transform.translation.xy().adjust_precision(),
            transform
                .rotation
                .to_euler(EulerRot::XYZ)
                .2
                .adjust_precision(),
            lin_vel.0,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &SpatialQueryFilter::from_excluded_entities([entity]),
            |hit| {
                // For each collision, draw debug gizmos
                if hit.intersects() {
                    gizmos.circle_2d(transform.translation.xy(), 33.0, tailwind::RED_600);
                } else {
                    gizmos.arrow_2d(
                        hit.point.f32(),
                        (hit.point
                            + hit.normal.adjust_precision() * hit.collision_distance
                                / time.delta_secs().adjust_precision())
                        .f32(),
                        tailwind::EMERALD_400,
                    );
                }
                touched.insert(hit.entity);
                true
            },
        );

        // Update transform and velocity
        transform.translation = position.extend(0.0).f32();
        lin_vel.0 = projected_velocity;
    }
}

pub fn update_debug_text(
    mut text: Single<&mut Text, With<DebugText>>,
    player: Single<(&LinearVelocity, &TouchedEntities, &CollidingEntities), With<Player>>,
    names: Query<NameOrEntity>,
) {
    let (lin_vel, touched, colliding_entities) = player.into_inner();
    ***text = format!(
        "velocity: [{:.3}, {:.3}]\n{} intersections (goal is 0): {:#?}\n{} touched: {:#?}",
        lin_vel.x,
        lin_vel.y,
        colliding_entities.len(),
        names
            .iter_many(colliding_entities.iter())
            .map(|name| name
                .name
                .map(|n| format!("{} ({})", name.entity, n))
                .unwrap_or_else(|| format!("{}", name.entity)))
            .collect::<Vec<_>>(),
        touched.len(),
        names
            .iter_many(touched.iter())
            .map(|name| name
                .name
                .map(|n| format!("{} ({})", name.entity, n))
                .unwrap_or_else(|| format!("{}", name.entity)))
            .collect::<Vec<_>>()
    );
}

pub fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity), With<Player>>, 
    time: Res<Time>, 
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.linvel * time.delta_secs(); 
    }
}

pub fn run(
    player_query: Single<(&mut Stamina, &mut Speed, &Velocity, &mut PlayerStatus), With<Player>>,
    inventory: Single<&Inventory>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut stamina, mut speed, velocity, mut player) = player_query.into_inner(); 
    if player.condition != Status::Normal || inventory.searching {
        return; 
    }

    if keyboard_input.pressed(KeyCode::ShiftLeft) && velocity.linvel != Vec3::ZERO {
        speed.current = RUN_SPEED; 
        if stamina.current > EMPTY { stamina.current -= STAMINA_DRAIN; }
        else { 
            stamina.current = EMPTY;
            player.condition = Status::Exhausted;
            speed.current = speed.base; 
        }
    }
    else if keyboard_input.just_released(KeyCode::ShiftLeft) {
        speed.current = speed.base;
    }
}

pub fn prevent_movement (
    inventory: Single<&Inventory>,  
    mut velocity: Single<&mut Velocity, With<Player>>, 
) {
    if !inventory.searching {
        return; 
    }

    velocity.linvel = Vec3::ZERO; 
}