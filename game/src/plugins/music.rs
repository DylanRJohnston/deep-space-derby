use std::{cell::RefCell, time::Duration};

use bevy::{prelude::*, utils::tracing};
use bevy_kira_audio::prelude::*;

use super::scenes::{MusicAssets, RaceState, SceneState};

#[derive(Debug, Resource)]
pub struct MusicChannel;

#[derive(Debug, Resource)]
pub struct SoundEffectsChannel;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MusicHandles::default())
            .add_plugins(bevy_kira_audio::AudioPlugin)
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<SoundEffectsChannel>()
            .add_systems(
                Update,
                update_audio.run_if(|state: Res<State<SceneState>>| {
                    !matches!(state.get(), SceneState::Loading | SceneState::Spawning)
                }),
            )
            .observe(play_countdown);
    }
}

#[derive(Debug, Default, Resource)]
struct MusicHandles {
    lobby: Option<Handle<AudioInstance>>,
    pregame: Option<Handle<AudioInstance>>,
    race: Option<Handle<AudioInstance>>,
    results: Option<Handle<AudioInstance>>,
    crowd: Option<Handle<AudioInstance>>,
}

fn update_audio(
    scene_state: Res<State<SceneState>>,
    race_state: Option<Res<State<RaceState>>>,
    game_assets: Res<MusicAssets>,
    mut handles: ResMut<MusicHandles>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    music_channel: ResMut<AudioChannel<MusicChannel>>,
) {
    if !scene_state.is_changed()
        && !race_state
            .as_ref()
            .map(|state| state.is_changed())
            .unwrap_or_default()
    {
        return;
    }

    tracing::info!(?scene_state, "Updating audio");

    let audio_instances = RefCell::new(audio_instances.as_mut());

    // let play = |handle: &Option<Handle<AudioInstance>>| {
    //     if let Some(audio) = handle {
    //         audio_instances
    //             .borrow_mut()
    //             .get_mut(audio)
    //             .unwrap()
    //             .resume(AudioTween::new(
    //                 Duration::from_secs_f32(0.5),
    //                 AudioEasing::InOutPowi(2),
    //             ));
    //     }
    // };

    let pause = |handle: &Option<Handle<AudioInstance>>| {
        if let Some(audio) = handle {
            audio_instances
                .borrow_mut()
                .get_mut(audio)
                .unwrap()
                .pause(AudioTween::new(
                    Duration::from_secs_f32(0.1),
                    AudioEasing::OutPowi(2),
                ));
        }
    };

    let stop = |handle: &Option<Handle<AudioInstance>>| {
        if let Some(audio) = handle {
            audio_instances
                .borrow_mut()
                .get_mut(audio)
                .unwrap()
                .stop(AudioTween::new(
                    Duration::from_secs_f32(0.1),
                    AudioEasing::OutPowi(2),
                ));
        }
    };

    match scene_state.get() {
        SceneState::Loading => {}
        SceneState::Spawning => {}
        SceneState::Lobby => {
            tracing::info!("starting lobby music");
            match handles.lobby.as_ref() {
                Some(audio) => {
                    audio_instances
                        .borrow_mut()
                        .get_mut(audio)
                        .unwrap()
                        .resume(AudioTween::new(
                            Duration::from_secs_f32(0.1),
                            AudioEasing::InPowi(2),
                        ));
                }
                None => {
                    handles.lobby = Some(
                        music_channel
                            .play(game_assets.music_lobby.clone())
                            .fade_in(AudioTween::new(
                                Duration::from_secs_f32(0.1),
                                AudioEasing::InPowi(2),
                            ))
                            .looped()
                            .handle(),
                    );
                }
            };

            pause(&handles.pregame);
            pause(&handles.race);
            pause(&handles.results);
            stop(&handles.crowd);
        }
        SceneState::PreGame => {
            match handles.pregame.as_ref() {
                Some(audio) => {
                    audio_instances
                        .borrow_mut()
                        .get_mut(audio)
                        .unwrap()
                        .resume(AudioTween::new(
                            Duration::from_secs_f32(0.1),
                            AudioEasing::InPowi(2),
                        ));
                }
                None => {
                    handles.pregame = Some(
                        music_channel
                            .play(game_assets.music_pregame.clone())
                            .fade_in(AudioTween::new(
                                Duration::from_secs_f32(0.1),
                                AudioEasing::InPowi(2),
                            ))
                            .looped()
                            .handle(),
                    );
                }
            }

            pause(&handles.lobby);
            pause(&handles.race);
            pause(&handles.results);
            stop(&handles.crowd);
        }
        SceneState::Race => {
            match race_state.unwrap().get() {
                RaceState::PreRace => {
                    handles.crowd = Some(
                        music_channel
                            .play(game_assets.crowd.clone())
                            .fade_in(AudioTween::new(
                                Duration::from_secs_f32(0.1),
                                AudioEasing::InPowi(2),
                            ))
                            .looped()
                            .handle(),
                    );
                }
                RaceState::Race => match handles.race.as_ref() {
                    Some(audio) => {
                        audio_instances.borrow_mut().get_mut(audio).unwrap().resume(
                            AudioTween::new(Duration::from_secs_f32(0.1), AudioEasing::InPowi(2)),
                        );
                    }
                    None => {
                        handles.race = Some(
                            music_channel
                                .play(game_assets.music_race.clone())
                                .fade_in(AudioTween::new(
                                    Duration::from_secs_f32(0.1),
                                    AudioEasing::InPowi(2),
                                ))
                                .looped()
                                .handle(),
                        );
                    }
                },
            };

            pause(&handles.lobby);
            pause(&handles.pregame);
            pause(&handles.results);
        }
        SceneState::Results => {
            match handles.results.as_ref() {
                Some(audio) => {
                    audio_instances
                        .borrow_mut()
                        .get_mut(audio)
                        .unwrap()
                        .resume(AudioTween::new(
                            Duration::from_secs_f32(0.1),
                            AudioEasing::InPowi(2),
                        ));
                }
                None => {
                    handles.results = Some(
                        music_channel
                            .play(game_assets.music_results.clone())
                            // .fade_in(AudioTween::new(
                            //     Duration::from_secs(2),
                            //     AudioEasing::InPowi(2),
                            // ))
                            .looped()
                            .handle(),
                    );
                }
            }

            pause(&handles.lobby);
            pause(&handles.pregame);
            pause(&handles.race);
        }
    }
}

#[derive(Debug, Event)]
pub struct PlayCountdown;

fn play_countdown(
    _trigger: Trigger<PlayCountdown>,
    game_assets: Res<MusicAssets>,
    effects_channel: Res<AudioChannel<SoundEffectsChannel>>,
) {
    effects_channel.play(game_assets.countdown.clone());
}
