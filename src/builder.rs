use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::blocks::{wavebreaker_collider, BlockAssets, BlockType, House, HydroBody, PlacedBlock};
use crate::camera::{OrbitCamera, PointerGestureState};
use crate::state::{
    grid_to_world, world_to_grid_xz, BuildTool, GameBudget, GameState, LevelConfig, ResultStats,
    SelectedTool, CELL_SIZE, GRID_COLS, GRID_MAX_Y, GRID_ROWS,
};

#[derive(Resource, Default)]
pub struct GridOccupancy {
    pub cells: HashMap<(usize, usize, usize), Entity>,
}

#[derive(Resource, Default, Clone)]
pub struct DamBlueprint {
    pub entries: Vec<(BlockType, usize, usize, usize)>,
}

#[derive(Event, Debug, Clone, Copy)]
pub enum LevelEvent {
    ClearDam,
    RetryWave,
    NextLevel,
}

pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<BlockAssets>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.94),
            ..default()
        },
        Transform::from_xyz(14.0, 24.0, -10.0).looking_at(Vec3::new(0.0, 0.0, 4.0), Vec3::Y),
    ));

    let floor_mesh = meshes.add(Cuboid::new(16.0, 2.0, 64.0));
    commands.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(assets.seabed_mat.clone()),
        Transform::from_xyz(0.0, -1.0, 2.0),
        RigidBody::Static,
        Collider::cuboid(16.0, 2.0, 64.0),
        Friction::new(0.95),
    ));

    let cliff_mesh = meshes.add(Cuboid::new(12.0, 12.0, 64.0));
    for side in [-1.0_f32, 1.0_f32] {
        commands.spawn((
            Mesh3d(cliff_mesh.clone()),
            MeshMaterial3d(assets.stone_mat.clone()),
            Transform::from_xyz(side * 12.0, 5.0, 2.0),
            RigidBody::Static,
            Collider::cuboid(12.0, 12.0, 64.0),
            Friction::new(0.8),
        ));
    }

    let back_wall = meshes.add(Cuboid::new(16.0, 1.2, 8.0));
    commands.spawn((
        Mesh3d(back_wall),
        MeshMaterial3d(assets.stone_mat.clone()),
        Transform::from_xyz(0.0, 0.6, 22.0),
        RigidBody::Static,
        Collider::cuboid(16.0, 1.2, 8.0),
    ));

    let line_x = meshes.add(Cuboid::new(GRID_COLS as f32 * CELL_SIZE, 0.02, 0.04));
    let line_z = meshes.add(Cuboid::new(0.04, 0.02, GRID_ROWS as f32 * CELL_SIZE));

    for r in 0..=GRID_ROWS {
        let z = (r as f32 - GRID_ROWS as f32 * 0.5) * CELL_SIZE;
        commands.spawn((
            Mesh3d(line_x.clone()),
            MeshMaterial3d(assets.grid_line_mat.clone()),
            Transform::from_xyz(0.0, 0.015, z),
        ));
    }
    for c in 0..=GRID_COLS {
        let x = (c as f32 - GRID_COLS as f32 * 0.5) * CELL_SIZE;
        commands.spawn((
            Mesh3d(line_z.clone()),
            MeshMaterial3d(assets.grid_line_mat.clone()),
            Transform::from_xyz(x, 0.015, 0.0),
        ));
    }

    spawn_town(&mut commands, &assets);
}

pub fn spawn_town(commands: &mut Commands, assets: &BlockAssets) {
    let house_positions = [
        Vec3::new(-4.2, 0.63, 7.2),
        Vec3::new(-1.4, 0.63, 7.6),
        Vec3::new(1.5, 0.63, 7.4),
        Vec3::new(4.2, 0.63, 7.2),
        Vec3::new(-3.8, 0.63, 11.4),
        Vec3::new(-1.1, 0.63, 11.8),
        Vec3::new(1.8, 0.63, 11.5),
        Vec3::new(4.0, 0.63, 11.2),
    ];

    for pos in house_positions {
        commands
            .spawn((
                Mesh3d(assets.house_body_mesh.clone()),
                MeshMaterial3d(assets.house_mat.clone()),
                Transform::from_translation(pos),
                RigidBody::Static,
                Collider::cuboid(1.35, 1.25, 1.35),
                Mass(5.5),
                Friction::new(0.75),
                Restitution::new(0.08),
                ((LinearVelocity::ZERO,
                AngularVelocity::ZERO,
                LinearDamping(0.25),
                AngularDamping(0.45),
                ExternalForce::ZERO.with_persistence(false),
                ExternalImpulse::ZERO.with_persistence(false))),
                House { initial_pos: pos },
                HydroBody {
                    half_height: 0.625,
                    buoyancy: 0.85,
                    horizontal_damping: 1.0,
                    is_wavebreaker: false,
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(assets.house_roof_mesh.clone()),
                    MeshMaterial3d(assets.roof_mat.clone()),
                    Transform::from_xyz(0.0, 0.72, 0.0),
                ));
            });
    }
}

pub fn spawn_block_entity(
    commands: &mut Commands,
    assets: &BlockAssets,
    kind: BlockType,
    gx: usize,
    gy: usize,
    gz: usize,
) -> Entity {
    let spec = kind.spec();
    let pos = grid_to_world(gx, gy, gz);
    let s = CELL_SIZE * 0.96;

    let (mesh, mat, collider) = match kind {
        BlockType::Concrete => (
            assets.cube_mesh.clone(),
            assets.concrete_mat.clone(),
            Collider::cuboid(s, s, s),
        ),
        BlockType::Wavebreaker => (
            assets.breaker_mesh.clone(),
            assets.breaker_mat.clone(),
            wavebreaker_collider(s),
        ),
        BlockType::Wood => (
            assets.cube_mesh.clone(),
            assets.wood_mat.clone(),
            Collider::cuboid(s, s, s),
        ),
        BlockType::Anchor => (
            assets.anchor_mesh.clone(),
            assets.anchor_mat.clone(),
            Collider::cuboid(s * 0.92, s, s * 0.92),
        ),
    };

    commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(pos),
            RigidBody::Static,
            collider,
            Mass(spec.mass),
            Friction::new(spec.friction),
            Restitution::new(0.05),
            ((LinearVelocity::ZERO,
            AngularVelocity::ZERO,
            LinearDamping(0.2),
            AngularDamping(0.45),
            ExternalForce::ZERO.with_persistence(false),
            ExternalImpulse::ZERO.with_persistence(false))),
            PlacedBlock { kind, gx, gy, gz },
            HydroBody {
                half_height: CELL_SIZE * 0.5,
                buoyancy: spec.buoyancy,
                horizontal_damping: spec.horizontal_damping,
                is_wavebreaker: kind == BlockType::Wavebreaker,
            },
        ))
        .id()
}

pub fn handle_build_input(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    gesture: Res<PointerGestureState>,
    ui_interactions: Query<&Interaction, With<Button>>,
    selected: Res<SelectedTool>,
    mut budget: ResMut<GameBudget>,
    mut occupancy: ResMut<GridOccupancy>,
    assets: Res<BlockAssets>,
    block_q: Query<&PlacedBlock>,
) {
    if ui_interactions
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered))
    {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let mut tap_pos: Option<Vec2> = None;

    for touch in touches.iter_just_released() {
        let moved = touch.position().distance(touch.start_position());
        if moved < 16.0 && !gesture.was_multitouch {
            tap_pos = Some(touch.position());
            break;
        }
    }

    if tap_pos.is_none() && mouse.just_released(MouseButton::Left) && !gesture.was_drag {
        tap_pos = window.cursor_position();
    }

    let Some(screen_pos) = tap_pos else {
        return;
    };

    if screen_pos.y < 76.0 || screen_pos.y > window.height() - 116.0 {
        return;
    }

    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_transform, screen_pos) else {
        return;
    };

    if ray.direction.y.abs() < 1e-4 {
        return;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t <= 0.0 {
        return;
    }
    let hit = ray.origin + *ray.direction * t;

    let Some((gx, gz)) = world_to_grid_xz(hit.x, hit.z) else {
        return;
    };

    match selected.tool {
        BuildTool::Place(kind) => {
            let cost = kind.spec().cost;
            if budget.remaining < cost {
                return;
            }
            if let Some(gy) = (0..GRID_MAX_Y).find(|&y| !occupancy.cells.contains_key(&(gx, y, gz)))
            {
                let entity = spawn_block_entity(&mut commands, &assets, kind, gx, gy, gz);
                occupancy.cells.insert((gx, gy, gz), entity);
                budget.remaining -= cost;
            }
        }
        BuildTool::Delete => {
            if let Some(gy) = (0..GRID_MAX_Y)
                .rev()
                .find(|&y| occupancy.cells.contains_key(&(gx, y, gz)))
            {
                if let Some(entity) = occupancy.cells.remove(&(gx, gy, gz)) {
                    if let Ok(block) = block_q.get(entity) {
                        budget.remaining += block.kind.spec().cost;
                    }
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

pub fn activate_wave_physics(
    mut commands: Commands,
    mut blueprint: ResMut<DamBlueprint>,
    mut blocks: Query<(Entity, &PlacedBlock, &mut RigidBody), Without<House>>,
    mut houses: Query<(Entity, &mut RigidBody), With<House>>,
) {
    blueprint.entries.clear();

    for (entity, block, mut rb) in &mut blocks {
        blueprint.entries.push((block.kind, block.gx, block.gy, block.gz));
        if !block.kind.spec().static_in_wave {
            *rb = RigidBody::Dynamic;
            commands.entity(entity).remove::<Sleeping>();
        }
    }

    for (entity, mut rb) in &mut houses {
        *rb = RigidBody::Dynamic;
        commands.entity(entity).remove::<Sleeping>();
    }
}

pub fn evaluate_result(
    houses: Query<(&Transform, &House)>,
    mut stats: ResMut<ResultStats>,
) {
    let mut total = 0;
    let mut survived = 0;

    for (tf, house) in &houses {
        total += 1;
        let drift = tf.translation.distance(house.initial_pos);
        let upright = tf.up().y > 0.70;
        let on_ground = tf.translation.y > -0.25;
        if drift < 1.65 && upright && on_ground {
            survived += 1;
        }
    }

    stats.total_houses = total;
    stats.survived_houses = survived;
    stats.passed = total > 0 && survived * 2 >= total;
}

pub fn handle_level_events(
    mut events: EventReader<LevelEvent>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut level: ResMut<LevelConfig>,
    mut budget: ResMut<GameBudget>,
    mut occupancy: ResMut<GridOccupancy>,
    blueprint: Res<DamBlueprint>,
    assets: Res<BlockAssets>,
    blocks_q: Query<Entity, With<PlacedBlock>>,
    houses_q: Query<Entity, With<House>>,
) {
    for ev in events.read() {
        for e in &blocks_q {
            commands.entity(e).despawn_recursive();
        }
        occupancy.cells.clear();

        match ev {
            LevelEvent::ClearDam => {
                budget.remaining = level.start_budget;
                for e in &houses_q {
                    commands.entity(e).despawn_recursive();
                }
                spawn_town(&mut commands, &assets);
                next_state.set(GameState::Build);
            }
            LevelEvent::RetryWave => {
                for e in &houses_q {
                    commands.entity(e).despawn_recursive();
                }
                spawn_town(&mut commands, &assets);

                let mut spent = 0;
                for &(kind, gx, gy, gz) in &blueprint.entries {
                    let e = spawn_block_entity(&mut commands, &assets, kind, gx, gy, gz);
                    occupancy.cells.insert((gx, gy, gz), e);
                    spent += kind.spec().cost;
                }
                budget.remaining = (level.start_budget - spent).max(0);
                next_state.set(GameState::Build);
            }
            LevelEvent::NextLevel => {
                *level = LevelConfig::for_level(level.level + 1);
                budget.remaining = level.start_budget;
                for e in &houses_q {
                    commands.entity(e).despawn_recursive();
                }
                spawn_town(&mut commands, &assets);
                next_state.set(GameState::Build);
            }
        }
    }
}
