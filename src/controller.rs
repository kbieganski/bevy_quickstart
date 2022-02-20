use bevy::{math::const_vec3, prelude::*};
use heron::{rapier_plugin::PhysicsWorld, CollisionLayers, CollisionShape, Gravity, Velocity};

#[derive(Component)]
pub struct BodyTag;
#[derive(Component)]
pub struct HeadTag;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(controller_grounded)
            .add_system(controller_move)
            .add_system(controller_jump)
            .add_system(controller_fly)
            .add_system(controller_yaw)
            .add_system(controller_pitch);
    }
}

#[derive(Component, Default)]
pub struct Jump(pub bool);

#[derive(Component, Default)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}

impl Look {
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation() * -Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation() * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation() * Vec3::Y
    }
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Component, Default)]
pub struct Grounded(bool);

#[derive(Component, Default)]
pub struct Flying;

#[derive(Component)]
pub struct CharacterProperties {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
}

impl CharacterProperties {
    fn interp_speed(&self, frac: f32) -> f32 {
        self.walk_speed + frac * (self.run_speed - self.walk_speed)
    }
}

impl Default for CharacterProperties {
    fn default() -> Self {
        Self {
            walk_speed: 5.0,
            run_speed: 8.0,
            jump_speed: 6.0,
        }
    }
}

#[derive(Component, Default)]
pub struct Movement {
    pub vector: Vec3,
    pub speed: f32,
}

pub fn controller_move(
    mut controller_query: Query<(&mut Velocity, &CharacterProperties, &Movement), Without<Flying>>,
) {
    const XZ: Vec3 = const_vec3!([1.0, 0.0, 1.0]);
    for (mut velocity, properties, movement) in controller_query.iter_mut() {
        let mut desired_velocity = (movement.vector * XZ).normalize();
        let speed = properties.interp_speed(movement.speed);
        desired_velocity = if desired_velocity.length_squared() > 1E-6 {
            desired_velocity.normalize() * speed
        } else {
            velocity.linear * 0.5 * XZ
        };
        velocity.linear.x = desired_velocity.x;
        velocity.linear.z = desired_velocity.z;
    }
}

pub fn controller_jump(
    mut controller_query: Query<
        (&mut Velocity, &CharacterProperties, &Jump, &Grounded),
        Without<Flying>,
    >,
) {
    for (mut velocity, properties, jump, grounded) in controller_query.iter_mut() {
        if jump.0 && grounded.0 {
            velocity.linear.y = properties.jump_speed;
        }
    }
}

pub fn controller_fly(
    mut controller_query: Query<(&mut Velocity, &CharacterProperties, &Movement), With<Flying>>,
) {
    for (mut velocity, properties, movement) in controller_query.iter_mut() {
        let direction = movement.vector;
        let speed =
            properties.walk_speed + movement.speed * (properties.run_speed - properties.walk_speed);
        velocity.linear = if direction.length_squared() > 1E-6 {
            direction.normalize() * speed
        } else {
            velocity.linear * 0.5
        };
    }
}

#[profiling::function]
pub fn controller_grounded(
    physics_world: PhysicsWorld,
    gravity: Res<Gravity>,
    mut query: Query<(Entity, &Transform, &CollisionShape, &mut Grounded)>,
) {
    for (entity, transform, shape, mut grounded) in query.iter_mut() {
        profiling::scope!("Grounded check");
        let dist = 0.01
            + match shape {
                CollisionShape::Sphere { radius } => *radius,
                CollisionShape::Capsule {
                    half_segment,
                    radius,
                } => half_segment + radius,
                _ => panic!("Grounded not implemented for this shape"),
            };
        grounded.0 = physics_world
            .ray_cast_with_filter(
                transform.translation,
                gravity.vector().normalize() * dist,
                false,
                CollisionLayers::default(),
                |hit_entity| entity != hit_entity,
            )
            .is_some();
    }
}

pub fn controller_yaw(mut query: Query<(&Look, &mut Transform)>) {
    for (look, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y(look.yaw);
    }
}

pub fn controller_pitch(
    looks: Query<&Look>,
    mut transforms: Query<(&Parent, &mut Transform), With<HeadTag>>,
) {
    for (parent, mut transform) in transforms.iter_mut() {
        if let Ok(look) = looks.get(parent.0) {
            transform.rotation = Quat::from_rotation_x(look.pitch);
        }
    }
}
