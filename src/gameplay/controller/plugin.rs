use avian2d::{math::*, prelude::*}; 
use bevy::{ecs::query::Has, prelude::*}; 

pub struct PlayerControllerPlugin; 

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<MovementAction>().add_systems(
            Update, (
                    keyboard_input, 
                    update_grounded, 
                    movement, 
                    apply_movement_damping
                )
            .chain()); 
    }
}

/* --- MESSAGES --- */
#[derive(Message)]
pub enum MovementAction {
    Walk(Scalar), 
    Jump,
    Run(Scalar), 
}

/*  --- COMPONENTS --- */  
#[derive(Component)]
pub struct PlayerController; 

// Marker for component indicating that entity is grounded. 
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded; 

// The accleration used for player movement. 
#[derive(Component)]
pub struct MovementAcceleration(Scalar); 

// The damping factor used for slowing down movement. 
#[derive(Component)]
pub struct MovementDampingFactor(Scalar); 

// The strength of a jump. 
#[derive(Component)]
pub struct JumpImpulse(Scalar); 

// The max angle a slope can have for a character controller
// to be ablet to climb and jump. If the slope is steeper than this angle, 
// the Character will slide down. 
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar); 

// Bundle that contains the components needed for a basic 
// dynamic character controller. 
#[derive(Bundle)]
pub struct PlayerControllerBundle {
    player_controller: PlayerController, 
    body: RigidBody,
    collider: Collider, 
    ground_caster: ShapeCaster, 
    locked_axes: LockedAxes, 
    movement: MovementBundle, 
}

// A bundle that contains components for character movement. 
#[derive(Bundle)]
pub struct MovementBundle {
    accleration: MovementAcceleration, 
    damping: MovementDampingFactor, 
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        accleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            accleration: MovementAcceleration(accleration),
            damping: MovementDampingFactor(damping), 
            jump_impulse: JumpImpulse(jump_impulse), 
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45)
    }
}

impl PlayerControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone(); 
        caster_shape.set_scale(Vector::ONE * 0.99, 10); 

        Self {
            player_controller: PlayerController,
            body: RigidBody::Dynamic, 
            collider, 
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Dir2::NEG_Y)
                .with_max_distance(10.0), 
            locked_axes: LockedAxes::ROTATION_LOCKED, 
            movement: MovementBundle::default(), 
        }
    }

    pub fn with_movement(
        mut self,
        accleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(accleration, damping, jump_impulse, max_slope_angle);
        self 
    }
}

// Sends ['MovementAction'] events based on keyboard input. 
// Remove Jump implementation and include up and down movement. 
fn keyboard_input(
    mut movement_writer: MessageWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>, 
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]); 
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]); 
    let horizontal = right as i8 - left as i8; 
    let direction = horizontal as Scalar; 

    if direction != 0.0 {
        movement_writer.write(MovementAction::Walk(direction)); 
    }

    if keyboard_input.just_pressed(KeyCode::Space){
        movement_writer.write(MovementAction::Jump); 
    }
}

// Contains gamepad_inputs 

// Updates the ['Grounded'] status for character controllers 
fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>), With<PlayerController>>,
) {
    for (entity, hits, rotation, max_slope_angle) in query.iter_mut() {
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_to(Vector::Y).abs() <= angle.0 
            } 
            else {
                true
            }
        }); 

        if is_grounded {
            commands.entity(entity).insert(Grounded); 
        } 
        else {
            commands.entity(entity).remove::<Grounded>(); 
        }
    }
}

fn movement(
    time: Res<Time>,
    mut movement_reader: MessageReader<MovementAction>,
    mut controllers: Query<(&MovementAcceleration, &JumpImpulse, &mut LinearVelocity, Has<Grounded>,)>, 
) {
    // Precision is adjusted so that the example works with 
    // both 'f32' and 'f64' features. Otherwise remove this. 
    let delta_time = time.delta_secs_f64().adjust_precision(); 

    for event in movement_reader.read() {
        for (movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in controllers.iter_mut() {
            match event {
                MovementAction::Walk(direction) => {
                    linear_velocity.x += *direction * movement_acceleration.0 * delta_time;
                }
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0; 
                    }
                }
                _ => {}
            }
        }
    }
}

// Slows down movement in the X direction. 
fn apply_movement_damping(
    time: Res<Time>, 
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>, 
) {
    // Precision is adjusted
    let delta_time = time.delta_secs_f64().adjust_precision(); 

    for (damping_factor, mut linear_velocity) in query.iter_mut() {
        linear_velocity.x *= 1.0 / (1.0 + damping_factor.0 * delta_time); 
    }
}