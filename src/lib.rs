pub mod blocks;
pub mod builder;
pub mod camera;
pub mod state;
pub mod tsunami;
pub mod ui;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::blocks::{setup_block_assets, BlockAssets};
use crate::builder::{
    activate_wave_physics, evaluate_result, handle_build_input, handle_level_events,
    setup_environment, DamBlueprint, GridOccupancy, LevelEvent,
};
use crate::camera::{setup_camera, update_orbit_camera, PointerGestureState};
use crate::state::{
    DamShield, GameBudget, GameState, LevelConfig, ResultStats, SelectedTool, WaveRuntime,
};
use crate::tsunami::{
    advance_wave_timer, apply_water_physics, reset_wave_runtime, setup_tsunami, update_dam_shield,
    update_spray_particles, update_water_mesh,
};
use crate::ui::{
    despawn_result_overlay, handle_ui_buttons, highlight_selected_tool, setup_hud,
    spawn_result_overlay, update_hud_texts,
};

#[bevy_main]
fn main() {
    run_app();
}

pub fn run_app() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Tsunami Dam Simulator".into(),
            resolution: (1280.0, 720.0).into(),
            resizable: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PhysicsPlugins::default())
    .insert_resource(ClearColor(Color::srgb(0.86, 0.88, 0.90)))
    .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
    .insert_resource(AmbientLight {
        color: Color::srgb(0.96, 0.95, 0.93),
        brightness: 420.0,
    })
    .init_state::<GameState>()
    .init_resource::<LevelConfig>()
    .init_resource::<GameBudget>()
    .init_resource::<SelectedTool>()
    .init_resource::<WaveRuntime>()
    .init_resource::<ResultStats>()
    .init_resource::<DamShield>()
    .init_resource::<GridOccupancy>()
    .init_resource::<DamBlueprint>()
    .init_resource::<PointerGestureState>()
    .init_resource::<BlockAssets>()
    .add_event::<LevelEvent>()
    .add_systems(
        Startup,
        (
            setup_block_assets,
            setup_camera,
            setup_hud,
            setup_tsunami,
            setup_environment,
        )
            .chain(),
    )
    .add_systems(OnEnter(GameState::Wave), (reset_wave_runtime, activate_wave_physics))
    .add_systems(OnEnter(GameState::Result), (evaluate_result, spawn_result_overlay).chain())
    .add_systems(OnExit(GameState::Result), despawn_result_overlay)
    .add_systems(
        Update,
        (
            update_orbit_camera,
            handle_ui_buttons,
            handle_level_events,
            highlight_selected_tool,
            update_hud_texts,
            update_water_mesh,
            update_spray_particles,
        ),
    )
    .add_systems(
        Update,
        handle_build_input
            .after(update_orbit_camera)
            .run_if(in_state(GameState::Build)),
    )
    .add_systems(Update, advance_wave_timer.run_if(in_state(GameState::Wave)))
    .add_systems(
        FixedUpdate,
        (update_dam_shield, apply_water_physics)
            .chain()
            .run_if(in_state(GameState::Wave)),
    );

    app.run();
}
