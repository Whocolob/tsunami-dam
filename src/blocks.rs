use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::state::CELL_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Concrete,
    Wavebreaker,
    Wood,
    Anchor,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockSpec {
    pub name: &'static str,
    pub cost: i32,
    pub mass: f32,
    pub friction: f32,
    pub buoyancy: f32,
    pub horizontal_damping: f32,
    pub static_in_wave: bool,
}

impl BlockType {
    pub fn spec(self) -> BlockSpec {
        match self {
            BlockType::Concrete => BlockSpec {
                name: "CONCRETE",
                cost: 30,
                mass: 16.0,
                friction: 0.92,
                buoyancy: 0.45,
                horizontal_damping: 0.85,
                static_in_wave: false,
            },
            BlockType::Wavebreaker => BlockSpec {
                name: "BREAKER",
                cost: 40,
                mass: 12.0,
                friction: 0.85,
                buoyancy: 0.55,
                horizontal_damping: 0.42,
                static_in_wave: false,
            },
            BlockType::Wood => BlockSpec {
                name: "WOOD",
                cost: 12,
                mass: 2.8,
                friction: 0.60,
                buoyancy: 2.35,
                horizontal_damping: 1.15,
                static_in_wave: false,
            },
            BlockType::Anchor => BlockSpec {
                name: "ANCHOR",
                cost: 65,
                mass: 50.0,
                friction: 1.0,
                buoyancy: 0.0,
                horizontal_damping: 0.0,
                static_in_wave: true,
            },
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PlacedBlock {
    pub kind: BlockType,
    pub gx: usize,
    pub gy: usize,
    pub gz: usize,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct House {
    pub initial_pos: Vec3,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct HydroBody {
    pub half_height: f32,
    pub buoyancy: f32,
    pub horizontal_damping: f32,
    pub is_wavebreaker: bool,
}

#[derive(Resource, Default, Clone)]
pub struct BlockAssets {
    pub cube_mesh: Handle<Mesh>,
    pub breaker_mesh: Handle<Mesh>,
    pub anchor_mesh: Handle<Mesh>,
    pub house_body_mesh: Handle<Mesh>,
    pub house_roof_mesh: Handle<Mesh>,

    pub concrete_mat: Handle<StandardMaterial>,
    pub breaker_mat: Handle<StandardMaterial>,
    pub wood_mat: Handle<StandardMaterial>,
    pub anchor_mat: Handle<StandardMaterial>,
    pub house_mat: Handle<StandardMaterial>,
    pub roof_mat: Handle<StandardMaterial>,
    pub stone_mat: Handle<StandardMaterial>,
    pub seabed_mat: Handle<StandardMaterial>,
    pub grid_line_mat: Handle<StandardMaterial>,
}

pub fn setup_block_assets(
    mut assets: ResMut<BlockAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let s = CELL_SIZE * 0.96;
    assets.cube_mesh = meshes.add(Cuboid::new(s, s, s));
    assets.breaker_mesh = meshes.add(create_wavebreaker_mesh(s));
    assets.anchor_mesh = meshes.add(Cuboid::new(s * 0.92, s, s * 0.92));
    assets.house_body_mesh = meshes.add(Cuboid::new(1.35, 1.25, 1.35));
    assets.house_roof_mesh = meshes.add(Cuboid::new(1.45, 0.22, 1.45));

    assets.concrete_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 0.69, 0.68),
        perceptual_roughness: 0.85,
        ..default()
    });
    assets.breaker_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.34, 0.38),
        perceptual_roughness: 0.55,
        metallic: 0.15,
        ..default()
    });
    assets.wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.76, 0.62, 0.44),
        perceptual_roughness: 0.78,
        ..default()
    });
    assets.anchor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.067, 0.071, 0.078),
        perceptual_roughness: 0.40,
        metallic: 0.55,
        ..default()
    });
    assets.house_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.961, 0.953, 0.933),
        perceptual_roughness: 0.82,
        ..default()
    });
    assets.roof_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.12, 0.14),
        perceptual_roughness: 0.65,
        ..default()
    });
    assets.stone_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.91, 0.89, 0.85),
        perceptual_roughness: 0.92,
        ..default()
    });
    assets.seabed_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.84, 0.82, 0.78),
        perceptual_roughness: 0.95,
        ..default()
    });
    assets.grid_line_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.067, 0.071, 0.078, 0.25),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
}

pub fn create_wavebreaker_mesh(size: f32) -> Mesh {
    let h = size * 0.5;

    let p0 = [-h, -h, -h];
    let p1 = [h, -h, -h];
    let p2 = [h, -h, h];
    let p3 = [-h, -h, h];
    let p4 = [-h, h, h];
    let p5 = [h, h, h];

    let slope_n = Vec3::new(0.0, 1.0, -1.0).normalize().to_array();

    let positions = vec![
        p0, p3, p2, p1,
        p3, p4, p5, p2,
        p0, p1, p5, p4,
        p0, p4, p3,
        p1, p2, p5,
    ];

    let normals = vec![
        [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],  [0.0, 0.0, 1.0],  [0.0, 0.0, 1.0],  [0.0, 0.0, 1.0],
        slope_n,          slope_n,          slope_n,          slope_n,
        [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],  [1.0, 0.0, 0.0],  [1.0, 0.0, 0.0],
    ];

    let indices = vec![
        0, 1, 2, 0, 2, 3,
        4, 5, 6, 4, 6, 7,
        8, 9, 10, 8, 10, 11,
        12, 13, 14,
        15, 16, 17,
    ];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn wavebreaker_collider(size: f32) -> Collider {
    let h = size * 0.5;
    let pts = vec![
        Vec3::new(-h, -h, -h),
        Vec3::new(h, -h, -h),
        Vec3::new(h, -h, h),
        Vec3::new(-h, -h, h),
        Vec3::new(-h, h, h),
        Vec3::new(h, h, h),
    ];
    Collider::convex_hull(pts).unwrap_or_else(|| Collider::cuboid(size, size, size))
}
