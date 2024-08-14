use bevy::{prelude::*, text::TextLayoutInfo, utils::tracing};
use shared::models::projections;

use crate::plugins::{
    event_stream::GameEvents,
    monster::{DespawnAllMonsters, MonsterBehaviour, MonsterID, MonsterInfo, SpawnMonster},
};

use super::{SceneMetadata, SceneState};

pub struct PreGamePlugin;

impl Plugin for PreGamePlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<SceneState>()
            .register_type::<PreGameSpawnPoint>()
            .register_type::<PreGameCamera>()
            .add_systems(OnEnter(SceneState::PreGame), (spawn_monsters).chain())
            // .add_systems(
            //     Update,
            //     update_ui_position.run_if(in_state(SceneState::PreGame)),
            // )
            .add_systems(
                OnEnter(SceneState::PreGame),
                init_pregame.after(spawn_pregame_spawn_point_on_scene_load),
            )
            .add_systems(Update, spawn_pregame_spawn_point_on_scene_load)
            .add_systems(
                OnEnter(SceneState::PreGame),
                |mut query: Query<&mut Visibility, With<SpotLight>>| {
                    for mut visibility in query.iter_mut() {
                        *visibility = Visibility::Inherited;
                    }
                },
            );
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct PreGameSpawnPoint {
    pub id: usize,
}

#[derive(Debug, Default, Reflect, Component)]
#[reflect(Component)]
pub struct PreGameCamera;

pub fn spawn_pregame_spawn_point_on_scene_load(
    query: Query<(Entity, &SceneMetadata), Added<SceneMetadata>>,
    mut commands: Commands,
) {
    for (entity, metadata) in &query {
        if let Some(value) = metadata.0.get("PreGameSpawnPoint") {
            match value {
                serde_json::Value::Number(n) if n.is_u64() => {
                    commands.entity(entity).insert(PreGameSpawnPoint {
                        id: n.as_u64().unwrap() as usize,
                    });
                }
                other => panic!("PreGameSpawnPoint must be a number, got {:?}", other),
            }
        }

        if metadata.0.get("PreGameCamera").is_some() {
            commands.entity(entity).insert(PreGameCamera);
        }
    }
}

fn spawn_monsters(
    spawn_points: Query<(&PreGameSpawnPoint, &Transform)>,
    game_events: Res<GameEvents>,
    mut commands: Commands,
) {
    commands.trigger(DespawnAllMonsters);

    let seed = projections::race_seed(&game_events);
    let monsters = projections::monsters(&game_events, seed);

    spawn_points
        .into_iter()
        .for_each(|(spawn_point, transform)| {
            let monster = monsters
                .get(spawn_point.id - 1)
                .ok_or_else(|| "failed to find race point for monster".to_string())
                .copied()
                .unwrap();

            commands.trigger(SpawnMonster {
                transform: *transform,
                monster,
                behaviour: MonsterBehaviour::Idle,
                id: spawn_point.id,
            })
        });
}

fn init_pregame(
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera>>,
    position: Query<&Transform, (With<PreGameCamera>, Without<Camera>)>,
) {
    let position = position.get_single().unwrap();
    let (mut camera, mut projection) = camera.get_single_mut().unwrap();

    camera.translation = position.translation;
    // Don't know why the rotation coming from blender is fucked up
    camera.rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * position.rotation;

    let Projection::Perspective(projection) = projection.as_mut() else {
        return;
    };

    projection.fov = 0.4;
}

#[derive(Debug, Component)]
struct StatID(Entity);

const FONT_COLOR: Color = Color::linear_rgb(0.0, 0.0, 0.0);

fn spawn_ui(
    mut commands: Commands,
    camera: Query<(&Camera, &GlobalTransform)>,
    monsters: Query<(Entity, &MonsterInfo, &GlobalTransform), With<MonsterID>>,
) {
    let (camera, camera_transform) = camera.get_single().unwrap();

    commands
        .spawn((
            StateScoped(SceneState::PreGame),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(75.),
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::End,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|container| {
            for (id, details, monster) in &monsters {
                let Some(position) =
                    camera.world_to_viewport(camera_transform, monster.translation())
                else {
                    return;
                };

                container
                    .spawn(NodeBundle {
                        style: Style {
                            // position_type: PositionType::Absolute,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.),
                            align_items: AlignItems::Stretch,
                            padding: UiRect::all(Val::Px(16.0)),
                            width: Val::Px(500.),
                            ..default()
                        },
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        background_color: Color::srgba_u8(0xff, 0xff, 0xff, 0x77).into(),
                        ..default()
                    })
                    .with_children(|container| {
                        let mut row = |f: &mut dyn FnMut(&mut ChildBuilder)| {
                            container
                                .spawn(NodeBundle {
                                    style: Style {
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|container| {
                                    f(container);
                                });
                        };

                        row(&mut |container| {
                            container.spawn(TextBundle {
                                text: Text::from_section(
                                    details.name,
                                    TextStyle {
                                        font_size: 40.0,
                                        color: FONT_COLOR,
                                        ..default()
                                    },
                                ),
                                ..default()
                            });
                        });

                        let odds_text_style = TextStyle {
                            color: FONT_COLOR,
                            font_size: 32.,
                            ..default()
                        };

                        row(&mut |container| {
                            // odds
                            container
                                .spawn(NodeBundle {
                                    style: Style {
                                        margin: UiRect::right(Val::Auto),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|container| {
                                    container.spawn(TextBundle {
                                        text: Text::from_section("Odds: ", odds_text_style.clone()),
                                        ..default()
                                    });
                                    container.spawn(TextBundle {
                                        text: Text::from_section("33%", odds_text_style.clone()),
                                        ..default()
                                    });
                                });

                            // Payout
                            container
                                .spawn(NodeBundle {
                                    style: Style { ..default() },
                                    ..default()
                                })
                                .with_children(|container| {
                                    container.spawn(TextBundle {
                                        text: Text::from_section(
                                            "Payout: ",
                                            odds_text_style.clone(),
                                        ),
                                        ..default()
                                    });
                                    container.spawn(TextBundle {
                                        text: Text::from_section("3.0x", odds_text_style.clone()),
                                        ..default()
                                    });
                                });
                        });

                        // Odds

                        let stats_text_style = TextStyle {
                            font_size: 32.,
                            color: FONT_COLOR,
                            ..default()
                        };

                        // Speed
                        let mut stats_row = |name: &str, amount: f32, color: Color| {
                            row(&mut |container| {
                                container
                                    .spawn(NodeBundle {
                                        style: Style {
                                            width: Val::Percent(33.),
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|container| {
                                        container.spawn(TextBundle {
                                            text: Text::from_section(
                                                format!("{name}:"),
                                                stats_text_style.clone(),
                                            ),
                                            ..default()
                                        });
                                    });

                                container
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_grow: 1.0,
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|container| {
                                        container.spawn(NodeBundle {
                                            style: Style {
                                                width: Val::Percent(100.),
                                                height: Val::Px(28.),
                                                position_type: PositionType::Absolute,
                                                ..default()
                                            },
                                            border_radius: BorderRadius::all(Val::Px(8.)),
                                            background_color: Color::srgb_u8(0x6b, 0x71, 0x79)
                                                .into(),
                                            ..default()
                                        });

                                        container.spawn(NodeBundle {
                                            style: Style {
                                                width: Val::Percent(100. * amount),
                                                height: Val::Px(28.),
                                                border: UiRect::all(Val::Px(4.)),
                                                ..default()
                                            },
                                            border_radius: BorderRadius::all(Val::Px(8.)),
                                            border_color: Color::srgb_u8(0x6b, 0x71, 0x79).into(),
                                            background_color: BackgroundColor::from(color),
                                            ..default()
                                        });

                                        for i in 1..9 {
                                            container.spawn(NodeBundle {
                                                style: Style {
                                                    height: Val::Px(28.),
                                                    position_type: PositionType::Absolute,
                                                    left: Val::Percent((i as f32) * 10.),
                                                    border: UiRect::all(Val::Px(2.)),
                                                    ..default()
                                                },
                                                border_color: Color::srgb_u8(0x6b, 0x71, 0x79)
                                                    .into(),
                                                ..default()
                                            });
                                        }
                                    });
                            });
                        };

                        stats_row("Speed", 0.5, Color::srgb_u8(0x97, 0xd9, 0x48));
                        // stats_row("Strength", 0.5, Color::srgb_u8(0xec, 0x6a, 0x45));
                        stats_row("Strength", 0.5, Color::srgb_u8(0xdc, 0x47, 0x3c));
                    });
                // });
            }
        });
}

fn update_ui_position(
    camera: Query<(&Camera, &GlobalTransform)>,
    monsters: Query<&GlobalTransform>,
    mut ui_nodes: Query<(&mut Style, &StatID)>,
) {
    // let (camera, camera_transform) = camera.get_single().unwrap();

    // for (mut style, StatID(entity)) in &mut ui_nodes {
    //     let monster = monsters.get(*entity).unwrap();

    //     let Some(position) = camera.world_to_viewport(camera_transform, monster.translation())
    //     else {
    //         continue;
    //     };

    //     style.top = Val::Px(position.y);
    //     style.left = Val::Px(position.x);
    // }
}
