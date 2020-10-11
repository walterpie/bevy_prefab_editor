use bevy::prelude::*;
use bevy_fly_camera::*;

use bevy_prefab_editor::*;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(FlyCameraPlugin)
        .register_component::<Asset<Mesh>>()
        .register_component::<IntoAsset<Color, StandardMaterial>>()
        .register_component::<DefaultComponent<GlobalTransform>>()
        .add_startup_system(setup.system())
        .add_system_to_stage(stage::LAST, load_asset_system::<Mesh>.system())
        .add_system_to_stage(
            stage::LAST,
            into_asset_system::<Color, StandardMaterial>.system(),
        )
        .add_system_to_stage(
            stage::LAST,
            default_component_system::<GlobalTransform>.system(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    commands
        .spawn(Camera3dComponents::default())
        .with(FlyCamera::default());
    let handle = asset_server.load::<Scene, _>("assets/prefab.scn").unwrap();

    scene_spawner.spawn(handle);
    asset_server.watch_for_changes().unwrap();
}
