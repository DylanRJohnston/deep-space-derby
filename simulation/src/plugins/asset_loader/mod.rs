use bevy::{gltf::Gltf, prelude::*};
use iyes_progress::prelude::*;

#[derive(Resource, Debug)]
pub struct AssetPack {
    pub mushnub: Handle<Gltf>,
    pub alien: Handle<Gltf>,
    pub cactoro: Handle<Gltf>,
}

#[cfg(target_arch = "wasm32")]
const ASSET_PREFIX: &str = "/assets";
#[cfg(not(target_arch = "wasm32"))]
const ASSET_PREFIX: &str = "./";

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    // let asset_pack = AssetPack {
    //     mushnub: asset_server.load(format!("{}/animated/Mushnub Evolved.glb", ASSET_PREFIX)),
    //     alien: asset_server.load(format!("{}/animated/Alien.glb", ASSET_PREFIX)),
    //     cactoro: asset_server.load(format!("{}/animated/Cactoro.glb", ASSET_PREFIX)),
    // };

    // loading.add(asset_pack.mushnub.clone());
    // loading.add(asset_pack.alien.clone());
    // loading.add(asset_pack.cactoro.clone());

    commands.insert_resource(AssetPack {
        alien: Handle::weak_from_u128(0),
        mushnub: Handle::weak_from_u128(0),
        cactoro: Handle::weak_from_u128(0),
    });
}

