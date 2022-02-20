use bevy::prelude::*;

use bevy_inspector_egui::bevy_egui::EguiContext;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_mod_picking::{PickingPluginsState, Selection};

use heron::prelude::*;

use crate::controller::Player;
use bevy_inspector_egui::widgets::InspectorQuery;
use bevy_inspector_egui::Inspectable;

#[derive(Component, Reflect)]
pub struct FrozenRigidBody(RigidBody);

#[derive(Component)]
pub struct Inspected;

#[derive(Inspectable, Default)]
pub struct Inspection {
    player: InspectorQuery<Entity, With<Player>>,
    selected: InspectorQuery<Entity, With<Inspected>>,
}

pub fn selection_inspector(
    mut commands: Commands,
    query: Query<(Entity, &Selection), (Without<Inspected>, Changed<Selection>)>,
) {
    for (entity, selection) in query.iter() {
        if selection.selected() {
            commands.entity(entity).insert(Inspected);
        }
    }
}

pub fn selection_kill_inspector(
    mut commands: Commands,
    query: Query<(Entity, &Selection), (With<Inspected>, Changed<Selection>)>,
) {
    for (entity, selection) in query.iter() {
        if !selection.selected() {
            commands.entity(entity).remove::<Inspected>();
        }
    }
}

pub fn selection_freeze_rigid_body(
    mut commands: Commands,
    query: Query<(Entity, &RigidBody, &Selection), Changed<Selection>>,
) {
    for (entity, body, selection) in query.iter() {
        if selection.selected() && body.can_have_velocity() {
            commands
                .entity(entity)
                .remove::<RigidBody>()
                .insert(RigidBody::Static)
                .insert(FrozenRigidBody(*body));
        }
    }
}

pub fn selection_unfreeze_rigid_body(
    mut commands: Commands,
    query: Query<(Entity, &FrozenRigidBody, &Selection), Changed<Selection>>,
) {
    for (entity, body, selection) in query.iter() {
        if !selection.selected() {
            commands
                .entity(entity)
                .remove::<FrozenRigidBody>()
                .remove::<RigidBody>()
                .insert(body.0);
        }
    }
}

#[derive(Default)]
pub struct Modes {
    inspect: bool,
    profiler: bool,
}

impl Modes {
    pub fn gameplay(&self) -> bool {
        return !(self.inspect || self.profiler);
    }
}

pub fn toggle_modes(mut modes: ResMut<Modes>, key: Res<Input<KeyCode>>) {
    if key.just_pressed(KeyCode::F1) {
        modes.inspect = !modes.inspect;
    }
    if key.just_pressed(KeyCode::F2) {
        modes.profiler = !modes.profiler;
    }
}

#[system]
pub fn cursor_grab(mut windows: ResMut<Windows>, modes: Res<Modes>) {
    if modes.is_changed() {
        let grab = modes.gameplay();
        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(grab);
        window.set_cursor_visibility(!grab);
    }
}

#[system]
pub fn inspect_cursor(
    egui: Res<EguiContext>,
    inspectors: Res<InspectorWindows>,
    mut picking: ResMut<PickingPluginsState>,
    modes: ResMut<Modes>,
) {
    let cursor_over_area = egui
        .ctx_for_window(inspectors.window_data::<Inspection>().window_id)
        .is_pointer_over_area();
    picking.enable_picking = modes.inspect; // && !pointer_over_area;
    picking.enable_interacting = modes.inspect && !cursor_over_area;
    picking.enable_highlighting = modes.inspect; // && !pointer_over_area;
}

#[system]
pub fn draw_profiler_ui(#[cfg(feature = "profiler")] egui: Res<EguiContext>, modes: Res<Modes>) {
    profiling::finish_frame!();
    if modes.profiler {
        if modes.is_changed() {
            #[cfg(feature = "profiler")]
            profiling::puffin::set_scopes_on(true);
        }
        #[cfg(feature = "profiler")]
        puffin_egui::profiler_window(egui.ctx());
    }
}
