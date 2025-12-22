use avian2d::{math::*, prelude::*}; 
use bevy::{ecs::query::Has, prelude::*};
use crate::gameplay::player::player::Player;  
use crate::gameplay::player::setup::Speed;
use crate::gameplay::inventory::inventory::Searching;

const MAX_SPEED: f32 = 10.0; 

pub struct PlayerControllerPlugin; 

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<MovementAction>().add_systems(
            Update, (
                    keyboard_input, 
                    movement, 
                    apply_movement_damping
                )
            .chain()); 
    }
}

/* --- MESSAGES --- */
#[derive(Message)]
pub enum MovementAction {
    Gait(Vec2), 
    Jump,
    Run(Vec2), 
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
}

impl MovementBundle {
    pub const fn new(
        accleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
    ) -> Self {
        Self {
            accleration: MovementAcceleration(accleration),
            damping: MovementDampingFactor(damping), 
            jump_impulse: JumpImpulse(jump_impulse), 
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0)
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
    ) -> Self {
        self.movement = MovementBundle::new(accleration, damping, jump_impulse);
        self 
    }
}

// Sends ['MovementAction'] events based on keyboard input. 
// Remove Jump implementation and include up and down movement. 
fn keyboard_input(
    mut movement_writer: MessageWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>, 
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]); 
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]); 
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]); 
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let vertical = (up as i8 - down as i8) as Scalar;  
    let horizontal = (right as i8 - left as i8) as Scalar; 
    let direction = Vec2::new(horizontal, vertical).normalize_or_zero(); // Ensure or_zero to prevent NaN clash with Avian

    if direction != Vec2::ZERO {
        println!("direction: {:?}", direction); 
        movement_writer.write(MovementAction::Gait(direction)); 
    }

    if keyboard_input.just_pressed(KeyCode::Space){
        movement_writer.write(MovementAction::Jump); 
    }
}

// Contains gamepad_inputs 
fn movement(
    time: Res<Time>,
    mut movement_reader: MessageReader<MovementAction>,
    mut controllers: Query<(&MovementAcceleration, &JumpImpulse, &mut LinearVelocity, Has<Grounded>), (With<Player>, Without<Searching>)>, 
    speed: Single<&Speed, With<Player>>,
) {
    // Precision is adjusted so that the example works with 
    // both 'f32' and 'f64' features. Otherwise remove this. 
    let delta_time = time.delta_secs_f64().adjust_precision(); 

    for event in movement_reader.read() {
        for (movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in controllers.iter_mut() {
            match event {
                MovementAction::Gait(direction) => {
                    linear_velocity.x += direction.x * speed.current * movement_acceleration.0 * delta_time;
                    linear_velocity.y += direction.y * speed.current * movement_acceleration.0 * delta_time; 
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
    speed: Single<&Speed, With<Player>>,
) {
    // Precision is adjusted
    let delta_time = time.delta_secs_f64().adjust_precision(); 
    let speed_ratio = (speed.current / MAX_SPEED).min(1.0);
    let damping_multiplier = 1.0 - speed_ratio;

    for (damping_factor, mut linear_velocity) in query.iter_mut() {
        linear_velocity.x *= 1.0 / (1.0 + damping_factor.0 * damping_multiplier * delta_time); 
        linear_velocity.y *= 1.0 / (1.0 + damping_factor.0 * damping_multiplier * delta_time);
    }
}