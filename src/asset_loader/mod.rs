use bevy::{gltf::Gltf, prelude::*, utils::HashMap};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, States)]
pub enum AssetLoaderState {
    #[default]
    Loading,
    Done,
}

pub struct AssetLoaderPlugin;
impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AssetLoaderState>()
            .add_systems(OnEnter(AssetLoaderState::Loading), load_assets)
            .add_systems(
                Update,
                check_for_load_complete.run_if(in_state(AssetLoaderState::Loading)),
            );
    }
}

#[derive(Resource, Debug)]
pub struct AssetPack(pub HashMap<&'static str, Handle<Gltf>>);

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AssetPack(HashMap::from([
        ("mushnub", asset_server.load("animated/Mushnub Evolved.glb")),
        ("alien", asset_server.load("animated/Alien.glb")),
        ("cactoro", asset_server.load("animated/Cactoro.glb")),
    ])))
}

fn check_for_load_complete(
    asset_pack: Res<AssetPack>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AssetLoaderState>>,
) {
    let all_loaded = asset_pack
        .0
        .iter()
        .all(|(_name, handle)| asset_server.is_loaded_with_dependencies(handle));

    if !all_loaded {
        return;
    }

    println!("All assets loaded");

    next_state.set(AssetLoaderState::Done);
}
