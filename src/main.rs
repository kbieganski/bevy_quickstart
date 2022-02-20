use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

use bevy_atmosphere::AtmosphereMat;
use controller::{Grounded, Jump, Look, Movement, Player};
use editor::{
    cursor_grab, draw_profiler_ui, inspect_cursor, selection_freeze_rigid_body,
    selection_inspector, selection_kill_inspector, selection_unfreeze_rigid_body, toggle_modes,
    Inspection, Modes,
};
use heron::prelude::*;

use input::{InputMap, InputPlugin};

pub mod controller;
pub mod editor;
pub mod input;
pub mod util;
use crate::controller::{BodyTag, CharacterControllerPlugin, CharacterProperties, HeadTag};
use bevy_inspector_egui::InspectorPlugin;

#[macro_use]
extern crate bevy_discovery;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::Static,
            CollisionShape::Cuboid {
                half_extends: Vec3::new(50.0, 0.0, 50.0),
                border_radius: None,
            },
        ));

    for _ in 1..10 {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 2.0, 0.0),
                ..Default::default()
            })
            .insert_bundle((
                RigidBody::Dynamic,
                CollisionShape::Cuboid {
                    half_extends: Vec3::new(0.5, 0.5, 0.5),
                    border_radius: None,
                },
            ))
            .insert_bundle(bevy_mod_picking::PickableBundle::default())
            .insert(bevy_transform_gizmo::GizmoTransformable);
    }

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 15000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_axis_angle(
            Vec3::new(1.0, 1.0, 1.0).normalize(),
            -3.14 / 3.0,
        )),
        directional_light: DirectionalLight {
            illuminance: 25000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
    let body = commands
        .spawn_bundle((
            GlobalTransform::identity(),
            //Transform::identity(),
            Transform::from_xyz(5.0, 20.0, 5.0),
            CharacterProperties::default(),
            Jump::default(),
            Movement::default(),
            Grounded::default(),
            RigidBody::Dynamic,
            CollisionShape::Capsule {
                half_segment: 0.5,
                radius: 0.5,
            },
            Velocity::default(),
            RotationConstraints::lock(),
            Player,
            BodyTag,
        ))
        .id();
    let head = commands
        .spawn_bundle((
            GlobalTransform::identity(),
            Transform::from_matrix(Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::default(),
                (0.5 * 1.9 + 0.3) * Vec3::Y,
            )),
            HeadTag,
        ))
        .id();
    let camera = commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(Vec3::ZERO, -Vec3::Z, Vec3::Y)),
            ..Default::default()
        })
        .insert_bundle(bevy_mod_picking::PickingCameraBundle::default())
        .insert(bevy_transform_gizmo::GizmoPickSource::default())
        .id();
    commands
        .entity(body)
        .insert(Look::default())
        .push_children(&[head]);
    commands.entity(head).push_children(&[camera]);
}

fn save_scene_system(world: &mut World) {
    let modes = world.get_resource::<Modes>().unwrap();
    if !modes.gameplay() {
        let key = world.get_resource::<Input<KeyCode>>().unwrap();
        if key.just_pressed(KeyCode::S) && key.pressed(KeyCode::LControl) {
            let type_registry = world.get_resource::<TypeRegistry>().unwrap();
            let scene = DynamicScene::from_world(&world, type_registry);
            info!("{}", scene.serialize_ron(type_registry).unwrap());
        }
    }
}

//#[derive(DiscoveryPlugin)] -- update discovery lib to bevy 0.6
//struct DiscoveryPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Modes::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(InputMap::default())
        .insert_resource(AtmosphereMat::default())
        .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: true })
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(CharacterControllerPlugin)
        .add_plugins(bevy_mod_picking::DefaultPickingPlugins)
        .add_plugin(bevy_transform_gizmo::TransformGizmoPlugin::new(
            Quat::IDENTITY,
        ))
        .add_plugin(InspectorPlugin::<Inspection>::new())
        //.add_plugin(BlenderPlugin)
        .add_startup_system(setup)
        .add_system(save_scene_system.exclusive_system())
        .add_system(selection_freeze_rigid_body)
        .add_system(selection_unfreeze_rigid_body)
        .add_system(selection_inspector)
        .add_system(selection_kill_inspector)
        .add_system(toggle_modes)
        .add_system(cursor_grab)
        .add_system(inspect_cursor)
        .add_system(draw_profiler_ui)
        .run();
}
