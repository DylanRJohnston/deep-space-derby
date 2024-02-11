use bevy::prelude::*;

use crate::plugins::async_task::AsyncTask;

pub struct FetchDataPlugin;

impl Plugin for FetchDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, fetch_data)
            .add_systems(Update, display_data);
    }
}

#[derive(Debug, Component)]
pub struct DataRequest(AsyncTask<Result<String, ()>>);

fn fetch_data(mut commands: Commands) {
    // let task = AsyncTask::spawn(async {
    //     reqwest::get("https://dummyjson.com/products/1")
    //         .await?
    //         .text()
    //         .await
    // });

    // commands.spawn(DataRequest(task));
}

fn display_data(tasks: Query<(Entity, &DataRequest)>, mut commands: Commands) {
    for (entity, DataRequest(task)) in &tasks {
        task.on_completion(|data| {
            match data {
                Ok(data) => {
                    commands.spawn(TextBundle {
                        text: Text::from_section(
                            data.clone(),
                            TextStyle {
                                font_size: 100.0,
                                ..default()
                            },
                        ),
                        ..default()
                    });
                }
                Err(error) => println!("Oh fuck, an error {:?}", error),
            };

            commands.entity(entity).remove::<DataRequest>();
        })
    }
}

