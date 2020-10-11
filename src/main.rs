use std::fs;

use bevy::prelude::*;
use bevy::render::render_graph::base::MainPass;
use bevy::type_registry::*;

use bevy_prefab_editor::*;

#[derive(Default)]
pub struct Editor {
    world: World,
    registry: TypeRegistry,
}

fn main() {
    App::build()
        .add_default_plugins()
        .init_resource::<Editor>()
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut res: ResMut<Editor>) {
    res.registry.register_component::<Asset<Mesh>>();
    res.registry
        .register_component::<IntoAsset<Color, StandardMaterial>>();
    res.registry.register_component::<MainPass>();
    res.registry.register_component::<Draw>();
    res.registry.register_component::<RenderPipelines>();
    res.registry.register_component::<Transform>();
    res.registry
        .register_component::<DefaultComponent<GlobalTransform>>();

    // we really only have to create this because of RenderPipelines
    let pbr = PbrComponents::default();
    res.world.spawn((
        Asset::<Mesh>::new("assets/bed.gltf"),
        IntoAsset::<_, StandardMaterial>::new(Color::rgb(1.0, 1.0, 1.0)),
        pbr.main_pass,
        pbr.draw,
        pbr.render_pipelines,
        pbr.transform,
        DefaultComponent::<GlobalTransform>::new(),
    ));

    let component = res.registry.component.read();
    let scene = Scene::from_world(&res.world, &component);

    let property = res.registry.property.read();
    fs::write("assets/prefab.scn", scene.serialize_ron(&property).unwrap()).unwrap();
}
