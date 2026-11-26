use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;

use crate::blocks::{HydroBody, PlacedBlock};
use crate::state::{
    world_to_grid_xz, DamShield, GameState, LevelConfig, WaveRuntime, GRID_COLS,
};

const WATER_NX: usize = 50;
const WATER_NZ: usize = 30;
const WATER_X_MIN: f32 = -6.0;
const WATER_X_MAX: f32 = 6.0;
const WATER_Z_MIN: f32 = -26.0;
const WATER_Z_MAX: f32 = 20.0;

const SPRAY_POOL_SIZE: usize = 140;

#[derive(Resource)]
pub struct WaterMeshHandle(pub Handle<Mesh>);

#[derive(Resource)]
pub struct WaterBuffers {
    pub heights: Vec<f32>,
}

#[derive(Component)]
pub struct SprayParticle {
    pub active: bool,
    pub velocity: Vec3,
    pub life: f32,
    pub max_life: f32,
}

#[derive(Resource)]
pub struct SpraySpawner {
    pub cursor: usize,
    pub rng: u32,
    pub pending: Vec<(Vec3, Vec3)>,
}

impl SpraySpawner {
    fn next_f32(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32) / (u32::MAX as f32)
    }

    pub fn queue_burst(&mut self, origin: Vec3, base_vel: Vec3, count: usize) {
        for _ in 0..count {
            if self.pending.len() >= SPRAY_POOL_SIZE {
                break;
            }
            let rx = (self.next_f32() - 0.5) * 3.5;
            let ry = self.next_f32() * 3.2;
            let rz = (self.next_f32() - 0.5) * 3.0;
            self.pending.push((origin, base_vel + Vec3::new(rx, ry, rz)));
        }
    }
}

pub fn setup_tsunami(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let vert_count = WATER_NX * WATER_NZ;
    let mut positions = vec![[0.0, 0.35, 0.0]; vert_count];
    let normals = vec![[0.0, 1.0, 0.0]; vert_count];
    let mut indices = Vec::with_capacity((WATER_NX - 1) * (WATER_NZ - 1) * 6);

    for iz in 0..WATER_NZ {
        let tz = iz as f32 / (WATER_NZ - 1) as f32;
        let z = WATER_Z_MIN + tz * (WATER_Z_MAX - WATER_Z_MIN);
        for ix in 0..WATER_NX {
            let tx = ix as f32 / (WATER_NX - 1) as f32;
            let x = WATER_X_MIN + tx * (WATER_X_MAX - WATER_X_MIN);
            positions[iz * WATER_NX + ix] = [x, 0.35, z];
        }
    }

    for iz in 0..(WATER_NZ - 1) {
        for ix in 0..(WATER_NX - 1) {
            let a = (iz * WATER_NX + ix) as u32;
            let b = a + 1;
            let c = a + WATER_NX as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    let water_handle = meshes.add(mesh);
    let water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.05, 0.38, 0.50, 0.78),
        perceptual_roughness: 0.14,
        metallic: 0.12,
        reflectance: 0.85,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(water_handle.clone()),
        MeshMaterial3d(water_mat),
        Transform::IDENTITY,
    ));

    commands.insert_resource(WaterMeshHandle(water_handle));
    commands.insert_resource(WaterBuffers {
        heights: vec![0.35; vert_count],
    });
    commands.insert_resource(SpraySpawner {
        cursor: 0,
        rng: 0x1337_BEEF,
        pending: Vec::with_capacity(SPRAY_POOL_SIZE),
    });

    let spray_mesh = meshes.add(Cuboid::new(0.16, 0.16, 0.16));
    let spray_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.96, 0.98, 1.0, 0.9),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    for _ in 0..SPRAY_POOL_SIZE {
        commands.spawn((
            Mesh3d(spray_mesh.clone()),
            MeshMaterial3d(spray_mat.clone()),
            Transform::from_xyz(0.0, -60.0, 0.0),
            SprayParticle {
                active: false,
                velocity: Vec3::ZERO,
                life: 0.0,
                max_life: 0.8,
            },
        ));
    }
}

pub fn reset_wave_runtime(mut wave: ResMut<WaveRuntime>, mut shield: ResMut<DamShield>) {
    wave.elapsed = 0.0;
    wave.front_z = -24.0;
    shield.column_heights = [0.0; GRID_COLS];
    shield.breaker_bonus = [0.0; GRID_COLS];
}

pub fn advance_wave_timer(
    time: Res<Time>,
    level: Res<LevelConfig>,
    mut wave: ResMut<WaveRuntime>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    wave.elapsed += time.delta_secs();
    wave.front_z = -24.0 + wave.elapsed * level.wave_speed;

    if wave.elapsed >= level.wave_duration {
        next_state.set(GameState::Result);
    }
}

pub fn update_dam_shield(
    mut shield: ResMut<DamShield>,
    blocks: Query<(&Transform, &PlacedBlock)>,
) {
    shield.column_heights = [0.0; GRID_COLS];
    shield.breaker_bonus = [0.0; GRID_COLS];

    for (tf, block) in &blocks {
        let p = tf.translation;
        if p.z >= -3.5 && p.z <= 3.8 && p.y >= 0.0 {
            if let Some((gx, _)) = world_to_grid_xz(p.x, 0.0) {
                let top = p.y + 0.5;
                if top > shield.column_heights[gx] {
                    shield.column_heights[gx] = top;
                }
                if block.kind == crate::blocks::BlockType::Wavebreaker {
                    shield.breaker_bonus[gx] = (shield.breaker_bonus[gx] + 0.55).min(1.3);
                }
            }
        }
    }
}

pub fn sample_wave(
    x: f32,
    z: f32,
    t: f32,
    active_wave: bool,
    level: &LevelConfig,
    shield: &DamShield,
) -> (f32, f32) {
    let ripple = 0.09 * (x * 0.75 + t * 2.4).sin() * (z * 0.45 - t * 3.1).cos();
    let calm_base = 0.25 + ripple * 0.5;

    if !active_wave {
        let sea_mask = ((-z - 3.5) / 3.0).clamp(0.0, 1.0);
        return (0.04 + (calm_base + 0.35) * sea_mask, 0.0);
    }

    let front_z = -24.0 + t * level.wave_speed;
    let d = front_z - z;

    let (raw_surge, crest) = if d < 0.0 {
        let s = (-d / 1.55).powi(2);
        let front = (-s).exp();
        (level.wave_height * front, front)
    } else {
        let peak = (-d / 4.2).exp();
        let tail = 0.46 * (-d / 18.0).exp();
        (level.wave_height * (peak * 0.72 + tail), peak)
    };

    let mut h = 0.12 + raw_surge + ripple * (0.4 + crest);

    if z > -0.5 {
        let col_f = ((x - WATER_X_MIN) / (WATER_X_MAX - WATER_X_MIN) * GRID_COLS as f32)
            .clamp(0.0, (GRID_COLS - 1) as f32);
        let col = col_f as usize;
        let shield_h = shield.column_heights[col] * 0.92 + shield.breaker_bonus[col];

        if shield_h > 0.1 {
            let behind = ((z + 0.5) / 2.2).clamp(0.0, 1.0);
            let overtop = (h - shield_h).max(0.0);
            let blocked_h = 0.12 + overtop * 1.05;
            h = h * (1.0 - behind) + blocked_h * behind;
        }
    }

    (h.max(0.03), crest)
}

pub fn update_water_mesh(
    time: Res<Time>,
    state: Res<State<GameState>>,
    level: Res<LevelConfig>,
    wave: Res<WaveRuntime>,
    shield: Res<DamShield>,
    mesh_handle: Res<WaterMeshHandle>,
    mut buffers: ResMut<WaterBuffers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut spawner: ResMut<SpraySpawner>,
) {
    let active = *state.get() == GameState::Wave;
    let t = if active { wave.elapsed } else { time.elapsed_secs() };

    let dx = (WATER_X_MAX - WATER_X_MIN) / (WATER_NX - 1) as f32;
    let dz = (WATER_Z_MAX - WATER_Z_MIN) / (WATER_NZ - 1) as f32;

    for iz in 0..WATER_NZ {
        let z = WATER_Z_MIN + iz as f32 * dz;
        for ix in 0..WATER_NX {
            let x = WATER_X_MIN + ix as f32 * dx;
            let (h, _) = sample_wave(x, z, t, active, &level, &shield);
            buffers.heights[iz * WATER_NX + ix] = h;
        }
    }

    if active && wave.front_z > WATER_Z_MIN && wave.front_z < WATER_Z_MAX {
        let rx = WATER_X_MIN + spawner.next_f32() * (WATER_X_MAX - WATER_X_MIN);
        let (h, _) = sample_wave(rx, wave.front_z, t, true, &level, &shield);
        spawner.queue_burst(
            Vec3::new(rx, h, wave.front_z),
            Vec3::new(0.0, 2.2, level.wave_speed * 0.45),
            2,
        );
    }

    let Some(mesh) = meshes.get_mut(&mesh_handle.0) else {
        return;
    };

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for (i, pos) in positions.iter_mut().enumerate() {
            pos[1] = buffers.heights[i];
        }
    }

    if let Some(VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for iz in 0..WATER_NZ {
            let iz_m = iz.saturating_sub(1);
            let iz_p = (iz + 1).min(WATER_NZ - 1);
            for ix in 0..WATER_NX {
                let ix_m = ix.saturating_sub(1);
                let ix_p = (ix + 1).min(WATER_NX - 1);

                let hl = buffers.heights[iz * WATER_NX + ix_m];
                let hr = buffers.heights[iz * WATER_NX + ix_p];
                let hd = buffers.heights[iz_m * WATER_NX + ix];
                let hu = buffers.heights[iz_p * WATER_NX + ix];

                let dh_dx = (hr - hl) / ((ix_p - ix_m).max(1) as f32 * dx);
                let dh_dz = (hu - hd) / ((iz_p - iz_m).max(1) as f32 * dz);

                let n = Vec3::new(-dh_dx, 1.0, -dh_dz).normalize();
                normals[iz * WATER_NX + ix] = n.to_array();
            }
        }
    }
}

pub fn apply_water_physics(
    level: Res<LevelConfig>,
    wave: Res<WaveRuntime>,
    shield: Res<DamShield>,
    mut spawner: ResMut<SpraySpawner>,
    mut bodies: Query<(
        &Transform,
        &RigidBody,
        &HydroBody,
        &Mass,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &LinearVelocity,
    )>,
) {
    let g = 9.81_f32;

    for (tf, rb, hydro, mass, mut ext_force, mut ext_impulse, vel) in &mut bodies {
        if *rb != RigidBody::Dynamic {
            continue;
        }

        let pos = tf.translation;
        let (water_h, crest) = sample_wave(pos.x, pos.z, wave.elapsed, true, &level, &shield);
        let bottom_y = pos.y - hydro.half_height;

        if bottom_y >= water_h {
            continue;
        }

        let submerged = ((water_h - bottom_y) / (2.0 * hydro.half_height)).clamp(0.0, 1.0);
        let m = mass.0.max(0.5);

        let buoyancy_force = Vec3::Y * (submerged * m * g * hydro.buoyancy);

        let impact_base = submerged * level.wave_speed * level.wave_height * (0.55 + crest * 1.45);
        let horiz_force = impact_base * 6.8 * hydro.horizontal_damping;
        let up_lift = if hydro.is_wavebreaker {
            -impact_base * 0.8
        } else {
            impact_base * 1.6
        };

        let flow_vel = Vec3::new(0.0, 0.0, level.wave_speed * (0.35 + 0.55 * crest));
        let rel_vel = flow_vel - vel.0;
        let drag_force = rel_vel * (submerged * 5.2);

        ext_force.apply_force(buoyancy_force + Vec3::new(0.0, up_lift, horiz_force) + drag_force);

        if crest > 0.65 && submerged > 0.25 {
            let impulse_z = crest * submerged * level.wave_speed * 0.35 * hydro.horizontal_damping;
            ext_impulse.apply_impulse(Vec3::new(0.0, impulse_z * 0.2, impulse_z));

            if hydro.is_wavebreaker {
                spawner.queue_burst(
                    pos + Vec3::new(0.0, 0.4, -0.4),
                    Vec3::new(0.0, 6.2, 1.5),
                    3,
                );
            } else if spawner.next_f32() < 0.35 {
                spawner.queue_burst(
                    pos + Vec3::new(0.0, 0.3, -0.4),
                    Vec3::new(0.0, 3.8, level.wave_speed * 0.4),
                    1,
                );
            }
        }
    }
}

pub fn update_spray_particles(
    time: Res<Time>,
    mut spawner: ResMut<SpraySpawner>,
    mut particles: Query<(&mut Transform, &mut SprayParticle)>,
) {
    let dt = time.delta_secs();

    for (mut tf, mut p) in &mut particles {
        if !p.active {
            continue;
        }
        p.life += dt;
        if p.life >= p.max_life || tf.translation.y < 0.0 {
            p.active = false;
            tf.translation.y = -60.0;
            continue;
        }
        p.velocity.y -= 16.5 * dt;
        tf.translation += p.velocity * dt;
        let s = (1.0 - p.life / p.max_life).clamp(0.2, 1.0);
        tf.scale = Vec3::splat(s);
    }

    if spawner.pending.is_empty() {
        return;
    }

    let mut pool_vec: Vec<_> = particles.iter_mut().collect();
    let len = pool_vec.len();
    if len == 0 {
        spawner.pending.clear();
        return;
    }

    let pending: Vec<(Vec3, Vec3)> = spawner.pending.drain(..).collect();
    for (origin, vel) in pending {
        let idx = spawner.cursor % len;
        spawner.cursor = spawner.cursor.wrapping_add(1);
        let (tf, p) = &mut pool_vec[idx];
        tf.translation = origin;
        tf.scale = Vec3::ONE;
        p.active = true;
        p.velocity = vel;
        p.life = 0.0;
        p.max_life = 0.65;
    }
}
