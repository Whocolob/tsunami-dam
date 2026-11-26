use bevy::prelude::*;

use crate::blocks::BlockType;
use crate::builder::LevelEvent;
use crate::state::{BuildTool, GameBudget, GameState, LevelConfig, ResultStats, SelectedTool};

const PANEL_DARK: Color = Color::srgba(0.067, 0.071, 0.078, 0.90);
const BTN_IDLE: Color = Color::srgb(0.13, 0.14, 0.16);
const BTN_SELECTED: Color = Color::srgb(0.16, 0.56, 0.66);
const BTN_START: Color = Color::srgb(0.14, 0.62, 0.44);
const BTN_DANGER: Color = Color::srgb(0.76, 0.25, 0.24);
const TEXT_MILK: Color = Color::srgb(0.961, 0.953, 0.933);

#[derive(Component, Clone, Copy)]
pub enum HudAction {
    SelectTool(BuildTool),
    StartWave,
    ClearDam,
    RetryWave,
    NextLevel,
}

#[derive(Component)]
pub struct TopInfoText;

#[derive(Component)]
pub struct ResultOverlayRoot;

pub fn setup_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(64.0),
                    padding: UiRect::horizontal(Val::Px(20.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                BackgroundColor(PANEL_DARK),
            ))
            .with_children(|top| {
                top.spawn((
                    Text::new("LVL 1  |  BUDGET: 360  |  WAVE: 3.4m"),
                    TextFont {
                        font_size: 21.0,
                        ..default()
                    },
                    TextColor(TEXT_MILK),
                    TopInfoText,
                ));
            });

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(96.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(PANEL_DARK),
            ))
            .with_children(|bar| {
                spawn_btn(
                    bar,
                    "CONCRETE
$30",
                    HudAction::SelectTool(BuildTool::Place(BlockType::Concrete)),
                    BTN_IDLE,
                );
                spawn_btn(
                    bar,
                    "BREAKER
$40",
                    HudAction::SelectTool(BuildTool::Place(BlockType::Wavebreaker)),
                    BTN_IDLE,
                );
                spawn_btn(
                    bar,
                    "WOOD
$12",
                    HudAction::SelectTool(BuildTool::Place(BlockType::Wood)),
                    BTN_IDLE,
                );
                spawn_btn(
                    bar,
                    "ANCHOR
$65",
                    HudAction::SelectTool(BuildTool::Place(BlockType::Anchor)),
                    BTN_IDLE,
                );
                spawn_btn(
                    bar,
                    "DELETE",
                    HudAction::SelectTool(BuildTool::Delete),
                    BTN_IDLE,
                );
                spawn_btn(bar, "START", HudAction::StartWave, BTN_START);
                spawn_btn(bar, "RESET", HudAction::ClearDam, BTN_DANGER);
            });
        });
}

fn spawn_btn(parent: &mut ChildBuilder, label: &str, action: HudAction, color: Color) {
    parent
        .spawn((
            Button,
            Node {
                min_width: Val::Px(96.0),
                height: Val::Px(72.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(color),
            BorderRadius::all(Val::Px(10.0)),
            action,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_MILK),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });
}

pub fn update_hud_texts(
    level: Res<LevelConfig>,
    budget: Res<GameBudget>,
    state: Res<State<GameState>>,
    mut q: Query<&mut Text, With<TopInfoText>>,
) {
    if !level.is_changed() && !budget.is_changed() && !state.is_changed() {
        return;
    }
    let phase = match state.get() {
        GameState::Build => "BUILD",
        GameState::Wave => "TSUNAMI!",
        GameState::Result => "RESULT",
    };
    for mut text in &mut q {
        **text = format!(
            "LVL {}   |   BUDGET: ${}   |   WAVE: {:.1}m   |   {}",
            level.level, budget.remaining, level.wave_height, phase
        );
    }
}

pub fn highlight_selected_tool(
    selected: Res<SelectedTool>,
    mut q: Query<(&HudAction, &mut BackgroundColor), With<Button>>,
) {
    if !selected.is_changed() {
        return;
    }
    for (action, mut bg) in &mut q {
        if let HudAction::SelectTool(tool) = action {
            *bg = if *tool == selected.tool {
                BackgroundColor(BTN_SELECTED)
            } else {
                BackgroundColor(BTN_IDLE)
            };
        }
    }
}

pub fn handle_ui_buttons(
    mut q: Query<(&Interaction, &HudAction), (Changed<Interaction>, With<Button>)>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected: ResMut<SelectedTool>,
    mut level_events: EventWriter<LevelEvent>,
) {
    for (interaction, action) in &mut q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            HudAction::SelectTool(tool) => {
                selected.tool = tool;
            }
            HudAction::StartWave => {
                if *state.get() == GameState::Build {
                    next_state.set(GameState::Wave);
                }
            }
            HudAction::ClearDam => {
                level_events.send(LevelEvent::ClearDam);
            }
            HudAction::RetryWave => {
                level_events.send(LevelEvent::RetryWave);
            }
            HudAction::NextLevel => {
                level_events.send(LevelEvent::NextLevel);
            }
        }
    }
}

pub fn spawn_result_overlay(mut commands: Commands, stats: Res<ResultStats>) {
    let title = if stats.passed {
        "TOWN SAVED"
    } else {
        "DAM BREACHED"
    };
    let subtitle = format!(
        "HOUSES SURVIVED: {} / {}",
        stats.survived_houses, stats.total_houses
    );

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.06, 0.65)),
            ResultOverlayRoot,
        ))
        .with_children(|bg| {
            bg.spawn((
                Node {
                    width: Val::Px(380.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                BackgroundColor(PANEL_DARK),
                BorderRadius::all(Val::Px(16.0)),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(title),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(TEXT_MILK),
                ));
                card.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(TEXT_MILK),
                ));

                card.spawn(Node {
                    column_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|row| {
                    spawn_btn(row, "RETRY", HudAction::RetryWave, BTN_IDLE);
                    if stats.passed {
                        spawn_btn(row, "NEXT LEVEL", HudAction::NextLevel, BTN_START);
                    }
                });
            });
        });
}

pub fn despawn_result_overlay(
    mut commands: Commands,
    q: Query<Entity, With<ResultOverlayRoot>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
