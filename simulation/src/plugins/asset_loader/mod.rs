use bevy::{gltf::Gltf, prelude::*};
use iyes_progress::prelude::*;

#[derive(Resource, Debug)]
pub struct AssetPack {
    pub mushnub: Handle<Gltf>,
    pub alien: Handle<Gltf>,
    pub cactoro: Handle<Gltf>,
}

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let asset_pack = AssetPack {
        mushnub: asset_server.load("/assets/animated/Mushnub Evolved.glb"),
        alien: asset_server.load("/assets/animated/Alien.glb"),
        cactoro: asset_server.load("/assets/animated/Cactoro.glb"),
    };

    loading.add(asset_pack.mushnub.clone());
    loading.add(asset_pack.alien.clone());
    loading.add(asset_pack.cactoro.clone());

    commands.insert_resource(asset_pack);
}
