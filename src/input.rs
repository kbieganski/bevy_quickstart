use bevy::input::keyboard::KeyCode;
use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::controller::{Flying, Jump, Look, Movement, Player};
use crate::Modes;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseSettings>()
            .add_system_to_stage(CoreStage::PreUpdate, input_to_jump)
            .add_system_to_stage(CoreStage::PreUpdate, input_to_movement)
            .add_system_to_stage(CoreStage::PreUpdate, input_toggle_flying)
            .add_system(input_to_look);
    }
}

#[derive(Debug)]
pub struct InputMap {
    pub key_forward: KeyCode,
    pub key_backward: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_jump: KeyCode,
    pub key_run: KeyCode,
    pub key_crouch: KeyCode,
    pub invert_y: bool,
    pub key_fly: KeyCode,
    pub key_fly_up: KeyCode,
    pub key_fly_down: KeyCode,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            key_forward: KeyCode::W,
            key_backward: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_jump: KeyCode::Space,
            key_run: KeyCode::LShift,
            key_crouch: KeyCode::LControl,
            invert_y: false,
            key_fly: KeyCode::F,
            key_fly_up: KeyCode::E,
            key_fly_down: KeyCode::Q,
        }
    }
}

pub struct MouseSettings {
    pub sensitivity: f32,
}

impl Default for MouseSettings {
    fn default() -> Self {
        Self { sensitivity: 0.005 }
    }
}

const PITCH_BOUND: f32 = std::f32::consts::FRAC_PI_2 - 1E-3;

pub fn input_to_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    settings: Res<MouseSettings>,
    mut looks: Query<&mut Look, With<Player>>,
    modes: Res<Modes>,
) {
    let mut delta = Vec2::ZERO;
    for motion in mouse_motion_events.iter() {
        delta -= motion.delta;
    }
    if modes.gameplay() && delta.length_squared() > 1E-6 {
        delta *= settings.sensitivity;
        for mut look in looks.iter_mut() {
            look.yaw += delta.x;
            look.pitch += delta.y;
            look.pitch = crate::util::clamp(look.pitch, -PITCH_BOUND, PITCH_BOUND);
        }
    }
}

pub fn input_to_jump(
    keyboard_input: Res<Input<KeyCode>>,
    input_map: Res<InputMap>,
    mut jumps: Query<&mut Jump, With<Player>>,
    modes: Res<Modes>,
) {
    if modes.gameplay() {
        for mut jump in jumps.iter_mut() {
            jump.0 = keyboard_input.just_pressed(input_map.key_jump);
        }
    }
}

pub fn input_to_movement(
    keyboard_input: Res<Input<KeyCode>>,
    input_map: Res<InputMap>,
    mut jumps: Query<(&mut Movement, &Look), With<Player>>,
    modes: Res<Modes>,
) {
    if modes.gameplay() {
        for (mut movement, look) in jumps.iter_mut() {
            let (forward, right, up) = (look.forward(), look.right(), look.up());
            let mut direction = Vec3::ZERO;
            if keyboard_input.pressed(input_map.key_forward) {
                direction += forward;
            }
            if keyboard_input.pressed(input_map.key_backward) {
                direction -= forward;
            }
            if keyboard_input.pressed(input_map.key_right) {
                direction += right;
            }
            if keyboard_input.pressed(input_map.key_left) {
                direction -= right;
            }
            if keyboard_input.pressed(input_map.key_fly_up) {
                direction += up;
            }
            if keyboard_input.pressed(input_map.key_fly_down) {
                direction -= up;
            }
            movement.vector = direction.normalize();
            movement.speed = if keyboard_input.pressed(input_map.key_run) {
                1.0
            } else {
                0.0
            };
        }
    }
}

pub fn input_toggle_flying(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    input_map: Res<InputMap>,
    flying_player: Query<Entity, (With<Player>, With<Flying>)>,
    grounded_player: Query<Entity, (With<Player>, Without<Flying>)>,
) {
    if keyboard_input.just_pressed(input_map.key_fly) {
        for entity in flying_player.iter() {
            commands.entity(entity).remove::<Flying>();
        }
        for entity in grounded_player.iter() {
            commands.entity(entity).insert(Flying);
        }
    }
}
