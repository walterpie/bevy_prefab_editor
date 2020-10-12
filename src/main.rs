use std::fs::{self, File};
use std::mem;
use std::path::Path;

use bevy::asset::AssetServerError;
use bevy::ecs::IntoThreadLocalSystem;
use bevy::input::mouse::*;
use bevy::prelude::*;
use bevy::property::{erased_serde, DeserializeProperty, DynamicProperties};
use bevy::scene;
use bevy::type_registry::*;
use bevy_fly_camera::*;
use bevy_mod_picking::*;
use hashbrown::HashMap;
use ron::Error as WriteError;

use bevy_prefab_editor::display::*;
use bevy_prefab_editor::editor::*;
use bevy_prefab_editor::entity::*;
use bevy_prefab_editor::*;

pub const BUTTON_NONE_MATERIAL: Handle<ColorMaterial> = Handle::from_bytes([
    250, 189, 108, 221, 189, 142, 172, 126, 18, 121, 71, 114, 210, 186, 138, 64,
]);

pub const BUTTON_HOVERED_MATERIAL: Handle<ColorMaterial> = Handle::from_bytes([
    161, 86, 132, 199, 157, 251, 102, 104, 177, 148, 191, 143, 237, 240, 172, 253,
]);

pub const BUTTON_CLICKED_MATERIAL: Handle<ColorMaterial> = Handle::from_bytes([
    7, 231, 89, 151, 216, 39, 3, 222, 173, 114, 218, 131, 106, 223, 122, 201,
]);

pub const BUTTON_TOGGLED_MATERIAL: Handle<ColorMaterial> = Handle::from_bytes([
    17, 203, 92, 43, 60, 26, 232, 74, 24, 150, 29, 168, 6, 235, 184, 74,
]);

#[derive(Default, Debug, Clone, Copy)]
pub struct ButtonToggled(bool);

#[derive(Debug, Clone, Copy)]
pub enum ButtonFunction {
    Save,
    AddComponent,
}

pub type EditorCommand = Box<dyn FnOnce(&mut World, &Resources) + Send + Sync + 'static>;

pub struct Editor {
    entity_map: HashMap<u32, Entity>,
    next_entity: u32,
    current_entity: Option<usize>,
    scene: Handle<Scene>,
}

impl Editor {
    pub fn write<P: AsRef<Path>>(
        &self,
        path: P,
        registry: &TypeRegistry,
        assets: &Assets<Scene>,
    ) -> Result<(), WriteError> {
        let scene = assets.get(&self.scene).unwrap();
        let property = registry.property.read();
        fs::write(path, scene.serialize_ron(&property)?)?;
        Ok(())
    }

    pub fn read<P: AsRef<Path>>(
        &mut self,
        path: P,
        world: &mut World,
        resources: &Resources,
    ) -> Result<(), AssetServerError> {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut assets = resources.get_mut::<Assets<Scene>>().unwrap();
        let registry = resources.get::<TypeRegistry>().unwrap();
        let handle = asset_server.load_sync(&mut assets, path)?;
        let scene = assets.get(&handle).unwrap();

        let component_registry = registry.component.read();

        self.next_entity = 0;
        for scene_entity in &scene.entities {
            let entity = world.spawn(WidgetComponents::new(self.next_entity));
            for component in &scene_entity.components {
                let registration = component_registry
                    .get_with_name(&component.type_name)
                    .unwrap();
                registration.add_component_to_entity(world, resources, entity, component);
            }
            let next_entity = self.next_entity;
            self.entity_map.insert(next_entity, entity);
        }

        self.scene = handle;
        Ok(())
    }
}

#[derive(Default)]
pub struct EditorCommands {
    next_entity: u32,
    current_entity: Option<u32>,
    queue: Vec<EditorCommand>,
}

impl EditorCommands {
    pub fn apply(&mut self, world: &mut World, resources: &Resources) {
        self.current_entity = None;
        for command in self.queue.drain(..) {
            command(world, resources);
        }
    }

    pub fn spawn(&mut self, components: EditorBundle) -> &mut Self {
        self.queue.push(Box::new(move |world, resources| {
            let mut editor = resources.get_mut::<Editor>().unwrap();
            let mut assets = resources.get_mut::<Assets<Scene>>().unwrap();
            let registry = resources.get::<TypeRegistry>().unwrap();
            let component_registry = registry.component.read();

            let scene = assets.get_mut(&editor.scene).unwrap();
            editor.current_entity = Some(scene.entities.len());

            let components = components.into_inner();

            let entity = world.spawn(WidgetComponents::new(editor.next_entity));
            for component in &components {
                let registration = component_registry
                    .get_with_name(&component.type_name)
                    .unwrap();
                registration.add_component_to_entity(world, resources, entity, component);
            }

            let next_entity = editor.next_entity;
            editor.entity_map.insert(next_entity, entity);
            scene.entities.push(scene::Entity {
                entity: editor.next_entity,
                components,
            });
            editor.next_entity += 1;
        }));
        self.current_entity = Some(self.next_entity);
        self.next_entity += 1;
        self
    }

    pub fn with(&mut self, component: DynamicProperties) -> &mut Self {
        self.queue.push(Box::new(move |_world, resources| {
            let editor = resources.get::<Editor>().unwrap();
            let mut assets = resources.get_mut::<Assets<Scene>>().unwrap();
            let registry = resources.get::<TypeRegistry>().unwrap();
            let component_registry = registry.component.read();

            let scene = assets.get_mut(&editor.scene).unwrap();
            let current_entity = editor.current_entity.expect("no current entity found");
            scene.entities[current_entity]
                .components
                .add(component, &component_registry);
        }));
        self
    }

    pub fn with_bundle(&mut self, bundle: EditorBundle) -> &mut Self {
        self.queue.push(Box::new(move |_world, resources| {
            let editor = resources.get::<Editor>().unwrap();
            let mut assets = resources.get_mut::<Assets<Scene>>().unwrap();
            let registry = resources.get::<TypeRegistry>().unwrap();
            let component_registry = registry.component.read();

            let scene = assets.get_mut(&editor.scene).unwrap();
            let current_entity = editor.current_entity.expect("no current entity found");
            scene.entities[current_entity]
                .components
                .add_bundle(bundle.into_inner(), &component_registry);
        }));
        self
    }

    pub fn insert_one(&mut self, entity: u32, component: DynamicProperties) -> &mut Self {
        self.queue.push(Box::new(move |_world, resources| {
            let editor = resources.get::<Editor>().unwrap();
            let mut assets = resources.get_mut::<Assets<Scene>>().unwrap();
            let registry = resources.get::<TypeRegistry>().unwrap();
            let component_registry = registry.component.read();

            let scene = assets.get_mut(&editor.scene).unwrap();
            scene.entities[entity as usize]
                .components
                .add(component, &component_registry);
        }));
        self
    }

    pub fn sync_to_world(&mut self, entity: u32) -> &mut Self {
        self.queue.push(Box::new(move |world, resources| {
            let editor = resources.get::<Editor>().unwrap();
            let assets = resources.get_mut::<Assets<Scene>>().unwrap();
            let registry = resources.get::<TypeRegistry>().unwrap();
            let component_registry = registry.component.read();

            let scene = assets.get(&editor.scene).unwrap();
            let components = &scene.entities[entity as usize].components;

            let world_entity = editor.entity_map[&entity];
            for component in components {
                let registration = component_registry
                    .get_with_name(&component.type_name)
                    .unwrap();
                registration.add_component_to_entity(world, resources, world_entity, component);
            }
        }));
        self
    }

    pub fn sync_one_to_world(&mut self, entity: u32, name: String) -> &mut Self {
        self.queue.push(Box::new(move |world, resources| {
            let editor = resources.get::<Editor>().unwrap();
            let assets = resources.get_mut::<Assets<Scene>>().unwrap();
            let registry = resources.get::<TypeRegistry>().unwrap();
            let component_registry = registry.component.read();

            let scene = assets.get(&editor.scene).unwrap();
            let components = &scene.entities[entity as usize].components;

            let world_entity = editor.entity_map[&entity];
            for component in components {
                if component.type_name == name {
                    let registration = component_registry
                        .get_with_name(&component.type_name)
                        .unwrap();
                    registration.add_component_to_entity(world, resources, world_entity, component);
                }
            }
        }));
        self
    }
}

impl FromResources for Editor {
    fn from_resources(resources: &Resources) -> Self {
        let mut assets = resources.get_mut::<Assets<Scene>>().unwrap();
        let scene = assets.add(Scene::default());
        Self {
            entity_map: HashMap::new(),
            next_entity: 0,
            current_entity: None,
            scene,
        }
    }
}

fn main() {
    let mut builder = App::build();
    builder
        .add_default_plugins()
        .add_plugin(FlyCameraPlugin)
        .add_plugin(PickingPlugin)
        .register_component::<Asset<Mesh>>()
        .register_component::<IntoAsset<Color, StandardMaterial>>()
        .register_component::<DefaultComponent<GlobalTransform>>()
        .add_event::<EditorEvent>()
        .init_resource::<Editor>()
        .init_resource::<EditorCommands>()
        .init_resource::<EditorMode>()
        .init_resource::<DefaultBundles>()
        .init_resource::<DefaultProperties>()
        .add_startup_system(setup.system())
        .add_startup_system(setup_thread_local.thread_local_system())
        .add_system(save_system.system())
        .add_system(camera_system.system())
        .add_system(button_enter_system.system())
        .add_system(button_system.system())
        .add_system(text_button_system.system());
    let resources = builder.resources_mut();
    let input_system = InputSystem::default().system(resources);
    builder.add_system(input_system);
    let resources = builder.resources_mut();
    let update_system = UpdateSystem::default().system(resources);
    builder
        .add_system(update_system)
        .add_system_to_stage(stage::POST_UPDATE, apply_system.thread_local_system())
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
    _editor: ResMut<EditorCommands>,
    asset_server: Res<AssetServer>,
    registry: Res<TypeRegistry>,
    mut default_bundles: ResMut<DefaultBundles>,
    mut default_properties: ResMut<DefaultProperties>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server
        .load::<Font, _>("assets/TruenoLight-E2pg.ttf")
        .unwrap();

    materials.set(BUTTON_NONE_MATERIAL, Color::rgb(0.5, 0.5, 0.5).into());
    materials.set(BUTTON_HOVERED_MATERIAL, Color::rgb(0.6, 0.6, 0.6).into());
    materials.set(BUTTON_CLICKED_MATERIAL, Color::rgb(0.75, 0.75, 0.75).into());
    materials.set(BUTTON_TOGGLED_MATERIAL, Color::rgb(0.3, 0.3, 0.3).into());

    commands
        .spawn(Camera3dComponents::default())
        .with(FlyCamera::default())
        .with(PickSource::default())
        .spawn(UiCameraComponents::default())
        .spawn(NodeComponents {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexEnd,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeComponents {
                    style: Style {
                        size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                        flex_direction: FlexDirection::ColumnReverse,
                        flex_wrap: FlexWrap::Wrap,
                        align_self: AlignSelf::FlexStart,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ButtonComponents {
                            style: Style {
                                size: Size::new(Val::Percent(80.0), Val::Px(30.0)),
                                padding: Rect::all(Val::Percent(10.0)),
                                justify_content: JustifyContent::SpaceAround,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            material: BUTTON_NONE_MATERIAL,
                            ..Default::default()
                        })
                        .with(ButtonFunction::Save)
                        .with_children(|parent| {
                            parent.spawn(TextComponents {
                                text: Text {
                                    value: "Save".to_string(),
                                    font,
                                    style: TextStyle {
                                        font_size: 20.0,
                                        color: Color::rgb(0.8, 0.8, 0.8),
                                    },
                                },
                                ..Default::default()
                            });
                        })
                        .spawn(ButtonComponents {
                            style: Style {
                                size: Size::new(Val::Percent(80.0), Val::Px(30.0)),
                                padding: Rect::all(Val::Percent(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            material: BUTTON_NONE_MATERIAL,
                            ..Default::default()
                        })
                        .with(ButtonFunction::AddComponent)
                        .with(ButtonToggled::default())
                        .with_children(|parent| {
                            parent.spawn(TextComponents {
                                text: Text {
                                    value: "Add component".to_string(),
                                    font,
                                    style: TextStyle {
                                        font_size: 20.0,
                                        color: Color::rgb(0.8, 0.8, 0.8),
                                    },
                                },
                                ..Default::default()
                            });
                        });
                });
        });

    let bundles_path: &Path = "assets/editor_bundles.ron".as_ref();
    let properties_path: &Path = "assets/editor_properties.ron".as_ref();

    let property = registry.property.read();

    if bundles_path.exists() {
        let text = fs::read_to_string(bundles_path).unwrap();
        let mut deserializer = ::ron::Deserializer::from_str(&text).unwrap();
        let mut deserializer = erased_serde::Deserializer::erase(&mut deserializer);
        let dynamic = DynamicProperties::deserialize(&mut deserializer, &property).unwrap();
        *default_bundles =
            DefaultBundles::from_dynamic(&dynamic.as_properties().unwrap().to_dynamic());
    }

    if properties_path.exists() {
        let text = fs::read_to_string(properties_path).unwrap();
        let mut deserializer = ::ron::Deserializer::from_str(&text).unwrap();
        let mut deserializer = erased_serde::Deserializer::erase(&mut deserializer);
        let dynamic = DynamicProperties::deserialize(&mut deserializer, &property).unwrap();
        *default_properties =
            DefaultProperties::from_dynamic(&dynamic.as_properties().unwrap().to_dynamic());
    }

    // let mut pbr = DefaultBundle("PbrComponents").default().unwrap();
    // pbr.add(Asset::<Mesh>::new("assets/bed.gltf").to_dynamic());
    // pbr.add(IntoAsset::<_, StandardMaterial>::new(Color::rgb(1.0, 1.0, 1.0)).to_dynamic());
    // let mut light = DefaultBundle("LightComponents").default().unwrap();
    // light.add(Light::default().to_dynamic());
    // light.add(Transform::from_translation(Vec3::new(5.0, 5.0, 5.0)).to_dynamic());
    // light.add(DefaultComponent::<GlobalTransform>::default().to_dynamic());
    // editor.spawn(pbr).spawn(light);
}

fn setup_thread_local(world: &mut World, resources: &mut Resources) {
    let mut editor = resources.get_mut::<Editor>().unwrap();
    editor.read("assets/prefab.scn", world, resources).unwrap();
}

fn camera_system(input: Res<Input<KeyCode>>, mut query: Query<Mut<FlyCamera>>) {
    if input.just_pressed(KeyCode::Q) {
        for mut fly_camera in &mut query.iter() {
            fly_camera.enabled = !fly_camera.enabled;
        }
    }
}

fn button_system(
    mut query: Query<
        With<
            Button,
            (
                Mutated<Interaction>,
                Option<Mut<ButtonToggled>>,
                Mut<Handle<ColorMaterial>>,
                &ButtonFunction,
            ),
        >,
    >,
) {
    for (interaction, mut toggled, mut material, _) in &mut query.iter() {
        match *interaction {
            Interaction::Clicked => {
                if let Some(toggled) = &mut toggled {
                    toggled.0 = !toggled.0;
                    if toggled.0 {
                        *material = BUTTON_TOGGLED_MATERIAL
                    } else {
                        *material = BUTTON_CLICKED_MATERIAL
                    }
                } else {
                    *material = BUTTON_CLICKED_MATERIAL
                }
            }
            Interaction::Hovered => *material = BUTTON_HOVERED_MATERIAL,
            Interaction::None => *material = BUTTON_NONE_MATERIAL,
        }
    }
}

fn button_enter_system(
    input: Res<Input<KeyCode>>,
    mut query: Query<With<Button, (Mut<ButtonToggled>, &ButtonFunction)>>,
) {
    for (mut toggled, _) in &mut query.iter() {
        if toggled.0 {
            if input.just_pressed(KeyCode::Return) {
                toggled.0 = false;
            }
        }
    }
}

fn text_button_system(
    input: Res<Input<KeyCode>>,
    default_properties: Res<DefaultProperties>,
    mut editor: ResMut<EditorCommands>,
    mut query: Query<With<Button, (&ButtonToggled, &ButtonFunction, &Children)>>,
    mut mutated: Query<With<Button, (Mutated<ButtonToggled>, &ButtonFunction, &Children)>>,
    texts: Query<Mut<Text>>,
    mut selected: Query<(&Widget, &SelectablePickMesh)>,
) {
    for (toggled, function, children) in &mut mutated.iter() {
        if toggled.0 {
            match function {
                ButtonFunction::Save => {}
                ButtonFunction::AddComponent => {
                    for &child in children.iter() {
                        texts.get_mut::<Text>(child).unwrap().value.clear();
                    }
                }
            }
        } else {
            match function {
                ButtonFunction::Save => {}
                ButtonFunction::AddComponent => {
                    for &child in children.iter() {
                        let mut component_name = "Add component".to_string();
                        mem::swap(
                            &mut texts.get_mut::<Text>(child).unwrap().value,
                            &mut component_name,
                        );
                        for (widget, selected) in &mut selected.iter() {
                            if selected.selected() {
                                let component = default_properties.get(&component_name).unwrap();
                                editor.insert_one(widget.0, component);
                                editor.sync_one_to_world(widget.0, component_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    for (toggled, function, children) in &mut query.iter() {
        if toggled.0 {
            match function {
                ButtonFunction::Save => {}
                ButtonFunction::AddComponent => {
                    let mut text = String::new();
                    for keycode in input.get_just_pressed() {
                        text.push_str(keycode.display().as_str());
                    }
                    if input.pressed(KeyCode::LShift) {
                        text.make_ascii_uppercase()
                    }
                    for &child in children.iter() {
                        texts.get_mut::<Text>(child).unwrap().value.push_str(&text);
                    }
                    if input.just_pressed(KeyCode::Back) {
                        for &child in children.iter() {
                            texts.get_mut::<Text>(child).unwrap().value.pop();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformMode {
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct EditorMode {
    mouse: bool,
    transform: Option<TransformMode>,
    axis: Option<Axis>,
    decimal: Option<i32>,
    value: f32,
}

struct InputSystem {
    multiplier: f32,
    motion: EventReader<MouseMotion>,
    wheel: EventReader<MouseWheel>,
}

impl InputSystem {
    pub fn system(self, resources: &mut Resources) -> Box<dyn System> {
        let system = input_system.system();
        resources.insert_local(system.id(), self);
        system
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self {
            multiplier: 0.001,
            motion: Default::default(),
            wheel: Default::default(),
        }
    }
}

fn input_system(
    mut state: Local<InputSystem>,
    input: Res<Input<KeyCode>>,
    mut mode: ResMut<EditorMode>,
    mut events: ResMut<Events<EditorEvent>>,
    motion: Res<Events<MouseMotion>>,
    wheel: Res<Events<MouseWheel>>,
) {
    if input.just_pressed(KeyCode::T) {
        if mode.transform == Some(TransformMode::Translate) {
            mode.mouse = !mode.mouse;
        }
        mode.transform = Some(TransformMode::Translate);
    }
    if input.just_pressed(KeyCode::R) {
        if mode.transform == Some(TransformMode::Rotate) {
            mode.mouse = !mode.mouse;
        }
        mode.transform = Some(TransformMode::Rotate);
    }
    if input.just_pressed(KeyCode::S) {
        if mode.transform == Some(TransformMode::Scale) {
            mode.mouse = !mode.mouse;
        }
        mode.transform = Some(TransformMode::Scale);
    }
    if input.just_pressed(KeyCode::Return) {
        mode.mouse = false;
        mode.transform = None;
        mode.axis = None;
        mode.decimal = None;
        mode.value = 0.0;
    }
    if input.just_pressed(KeyCode::X) {
        mode.axis = Some(Axis::X);
    }
    if input.just_pressed(KeyCode::Y) {
        mode.axis = Some(Axis::Y);
    }
    if input.just_pressed(KeyCode::Z) {
        mode.axis = Some(Axis::Z);
    }
    if input.just_pressed(KeyCode::Period) {
        mode.decimal = Some(0);
    }

    let mouse_mode = mode.mouse;

    if mouse_mode {
        let mut transform = |delta| {
            mode.value += delta;
            match mode.transform {
                Some(TransformMode::Translate) => {
                    let value = match mode.axis {
                        Some(Axis::X) => Vec3::new(delta, 0.0, 0.0),
                        Some(Axis::Y) => Vec3::new(0.0, delta, 0.0),
                        Some(Axis::Z) => Vec3::new(0.0, 0.0, delta),
                        None => Vec3::zero(),
                    };
                    let event = EditorEvent::Translate(value);
                    events.send(event);
                }
                Some(TransformMode::Rotate) => {
                    let value = match mode.axis {
                        Some(Axis::X) => Quat::from_rotation_x(delta),
                        Some(Axis::Y) => Quat::from_rotation_y(delta),
                        Some(Axis::Z) => Quat::from_rotation_z(delta),
                        None => Quat::identity(),
                    };
                    let event = EditorEvent::Rotate(value);
                    events.send(event);
                }
                Some(TransformMode::Scale) => {
                    let value = match mode.axis {
                        Some(Axis::X) => Vec3::new(delta, 1.0, 1.0),
                        Some(Axis::Y) => Vec3::new(1.0, delta, 1.0),
                        Some(Axis::Z) => Vec3::new(1.0, 1.0, delta),
                        None => Vec3::zero(),
                    };
                    let event = EditorEvent::Scale(value);
                    events.send(event);
                }
                None => {}
            }
        };

        for wheel in state.wheel.iter(&wheel) {
            let delta = match wheel.unit {
                MouseScrollUnit::Line => wheel.y * 0.003,
                MouseScrollUnit::Pixel => wheel.y * 0.00025,
            };
            state.multiplier += delta;
            state.multiplier = state.multiplier.max(0.001);
        }

        for motion in state.motion.iter(&motion) {
            transform(motion.delta.x() * state.multiplier);
        }
    } else {
        let mut transform = |digit| {
            let prev_value = mode.value;
            match &mut mode.decimal {
                Some(digits) => {
                    *digits += 1;
                    mode.value += 10.0_f32.powi(-*digits) * digit;
                }
                None => {
                    mode.value *= 10.0;
                    mode.value += digit;
                }
            }
            let delta = mode.value - prev_value;
            match mode.transform {
                Some(TransformMode::Translate) => {
                    let value = match mode.axis {
                        Some(Axis::X) => Vec3::new(delta, 0.0, 0.0),
                        Some(Axis::Y) => Vec3::new(0.0, delta, 0.0),
                        Some(Axis::Z) => Vec3::new(0.0, 0.0, delta),
                        None => Vec3::zero(),
                    };
                    let event = EditorEvent::Translate(value);
                    events.send(event);
                }
                Some(TransformMode::Rotate) => {
                    let value = match mode.axis {
                        Some(Axis::X) => Quat::from_rotation_x(delta),
                        Some(Axis::Y) => Quat::from_rotation_y(delta),
                        Some(Axis::Z) => Quat::from_rotation_z(delta),
                        None => Quat::identity(),
                    };
                    let event = EditorEvent::Rotate(value);
                    events.send(event);
                }
                Some(TransformMode::Scale) => {
                    let value = match mode.axis {
                        Some(Axis::X) => Vec3::new(delta, 1.0, 1.0),
                        Some(Axis::Y) => Vec3::new(1.0, delta, 1.0),
                        Some(Axis::Z) => Vec3::new(1.0, 1.0, delta),
                        None => Vec3::zero(),
                    };
                    let event = EditorEvent::Scale(value);
                    events.send(event);
                }
                None => {}
            }
        };

        if input.just_pressed(KeyCode::Key1) {
            transform(1.0);
        }
        if input.just_pressed(KeyCode::Key2) {
            transform(2.0);
        }
        if input.just_pressed(KeyCode::Key3) {
            transform(3.0);
        }
        if input.just_pressed(KeyCode::Key4) {
            transform(4.0);
        }
        if input.just_pressed(KeyCode::Key5) {
            transform(5.0);
        }
        if input.just_pressed(KeyCode::Key6) {
            transform(6.0);
        }
        if input.just_pressed(KeyCode::Key7) {
            transform(7.0);
        }
        if input.just_pressed(KeyCode::Key8) {
            transform(8.0);
        }
        if input.just_pressed(KeyCode::Key9) {
            transform(9.0);
        }
        if input.just_pressed(KeyCode::Key0) {
            transform(0.0);
        }
    }
}

#[derive(Default)]
struct UpdateSystem {
    reader: EventReader<EditorEvent>,
}

impl UpdateSystem {
    pub fn system(self, resources: &mut Resources) -> Box<dyn System> {
        let system = update_system.system();
        resources.insert_local(system.id(), self);
        system
    }
}

fn update_system(
    mut state: Local<UpdateSystem>,
    mut editor: ResMut<EditorCommands>,
    events: Res<Events<EditorEvent>>,
    mut query: Query<(&Widget, &SelectablePickMesh, Mut<Transform>)>,
) {
    for (widget, select, mut transform) in &mut query.iter() {
        if select.selected() {
            for event in state.reader.iter(&events) {
                match event {
                    EditorEvent::Translate(value) => {
                        transform.translate(*value);
                        editor.insert_one(widget.0, transform.to_dynamic());
                    }
                    EditorEvent::Rotate(value) => {
                        transform.rotate(*value);
                        editor.insert_one(widget.0, transform.to_dynamic());
                    }
                    EditorEvent::Scale(value) => {
                        transform.apply_non_uniform_scale(*value);
                        editor.insert_one(widget.0, transform.to_dynamic());
                    }
                }
            }
            break;
        }
    }
}

fn apply_system(world: &mut World, resources: &mut Resources) {
    let mut commands = resources.get_mut::<EditorCommands>().unwrap();
    commands.apply(world, resources);
}

fn save_system(
    input: Res<Input<KeyCode>>,
    editor: Res<Editor>,
    registry: Res<TypeRegistry>,
    assets: Res<Assets<Scene>>,
    default_bundles: Res<DefaultBundles>,
    default_properties: Res<DefaultProperties>,
    mut query: Query<With<Button, (&ButtonFunction, Mutated<Interaction>)>>,
) {
    if input.pressed(KeyCode::LControl) && input.just_pressed(KeyCode::S) {
        editor
            .write("assets/prefab.scn", &registry, &assets)
            .unwrap();

        let file = File::create("assets/editor_bundles.ron").unwrap();
        let mut serializer = ::ron::Serializer::new(file, Some(Default::default()), false).unwrap();
        let mut serializer = erased_serde::Serializer::erase(&mut serializer);
        default_bundles
            .to_dynamic()
            .serializable(&registry.property.read())
            .borrow()
            .erased_serialize(&mut serializer)
            .unwrap();

        let file = File::create("assets/editor_properties.ron").unwrap();
        let mut serializer = ::ron::Serializer::new(file, Some(Default::default()), false).unwrap();
        let mut serializer = erased_serde::Serializer::erase(&mut serializer);
        default_properties
            .to_dynamic()
            .serializable(&registry.property.read())
            .borrow()
            .erased_serialize(&mut serializer)
            .unwrap();
    }
    for (function, interaction) in &mut query.iter() {
        match *interaction {
            Interaction::Clicked => match function {
                ButtonFunction::Save => {
                    editor
                        .write("assets/prefab.scn", &registry, &assets)
                        .unwrap();

                    let file = File::create("assets/editor_bundles.ron").unwrap();
                    let mut serializer =
                        ::ron::Serializer::new(file, Some(Default::default()), false).unwrap();
                    let mut serializer = erased_serde::Serializer::erase(&mut serializer);
                    default_bundles
                        .to_dynamic()
                        .serializable(&registry.property.read())
                        .borrow()
                        .erased_serialize(&mut serializer)
                        .unwrap();

                    let file = File::create("assets/editor_properties.ron").unwrap();
                    let mut serializer =
                        ::ron::Serializer::new(file, Some(Default::default()), false).unwrap();
                    let mut serializer = erased_serde::Serializer::erase(&mut serializer);
                    default_properties
                        .to_dynamic()
                        .serializable(&registry.property.read())
                        .borrow()
                        .erased_serialize(&mut serializer)
                        .unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    }
}
