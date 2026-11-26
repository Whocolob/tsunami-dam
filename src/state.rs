use bevy::prelude::*;
use crate::blocks::BlockType;

pub const GRID_COLS: usize = 12;
pub const GRID_ROWS: usize = 5;
pub const GRID_MAX_Y: usize = 5;
pub const CELL_SIZE: f32 = 1.0;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Build,
    Wave,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildTool {
    Place(BlockType),
    Delete,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct SelectedTool {
    pub tool: BuildTool,
}

impl Default for SelectedTool {
    fn default() -> Self {
        Self {
            tool: BuildTool::Place(BlockType::Concrete),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct LevelConfig {
    pub level: u32,
    pub start_budget: i32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_duration: f32,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self::for_level(1)
    }
}

impl LevelConfig {
    pub fn for_level(level: u32) -> Self {
        let lvl = level.max(1);
        let f = (lvl - 1) as f32;
        Self {
            level: lvl,
            start_budget: 360 + (lvl as i32 - 1) * 90,
            wave_height: 3.4 + f * 0.75,
            wave_speed: 8.5 + f * 1.15,
            wave_duration: (6.8 - f * 0.15).clamp(4.8, 7.0),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GameBudget {
    pub remaining: i32,
}

impl Default for GameBudget {
    fn default() -> Self {
        Self {
            remaining: LevelConfig::for_level(1).start_budget,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct WaveRuntime {
    pub elapsed: f32,
    pub front_z: f32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ResultStats {
    pub total_houses: usize,
    pub survived_houses: usize,
    pub passed: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct DamShield {
    pub column_heights: [f32; GRID_COLS],
    pub breaker_bonus: [f32; GRID_COLS],
}

impl Default for DamShield {
    fn default() -> Self {
        Self {
            column_heights: [0.0; GRID_COLS],
            breaker_bonus: [0.0; GRID_COLS],
        }
    }
}

pub fn grid_to_world(gx: usize, gy: usize, gz: usize) -> Vec3 {
    let x = (gx as f32 - (GRID_COLS as f32 - 1.0) * 0.5) * CELL_SIZE;
    let y = (gy as f32 + 0.5) * CELL_SIZE;
    let z = (gz as f32 - (GRID_ROWS as f32 - 1.0) * 0.5) * CELL_SIZE;
    Vec3::new(x, y, z)
}

pub fn world_to_grid_xz(x: f32, z: f32) -> Option<(usize, usize)> {
    let fx = (x / CELL_SIZE) + (GRID_COLS as f32) * 0.5;
    let fz = (z / CELL_SIZE) + (GRID_ROWS as f32) * 0.5;
    if fx < 0.0 || fz < 0.0 {
        return None;
    }
    let gx = fx.floor() as usize;
    let gz = fz.floor() as usize;
    if gx < GRID_COLS && gz < GRID_ROWS {
        Some((gx, gz))
    } else {
        None
    }
}
