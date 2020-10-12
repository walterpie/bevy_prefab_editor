use bevy::pbr::{
    prelude::{Light, StandardMaterial},
    render_graph::FORWARD_PIPELINE_HANDLE,
};
use bevy::prelude::*;
use bevy::property::*;
use bevy::render::{
    draw::Draw,
    mesh::Mesh,
    pipeline::{DynamicBinding, PipelineSpecialization, RenderPipeline, RenderPipelines},
    render_graph::base::MainPass,
};
use bevy::type_registry::*;
use hashbrown::HashMap;

use super::*;

pub trait ComponentsExt {
    fn add(&mut self, component: DynamicProperties, registry: &ComponentRegistry);

    fn add_bundle<I: IntoIterator<Item = DynamicProperties>>(
        &mut self,
        other: I,
        registry: &ComponentRegistry,
    ) {
        for component in other {
            self.add(component, registry);
        }
    }
}

impl ComponentsExt for Vec<DynamicProperties> {
    fn add(&mut self, component: DynamicProperties, registry: &ComponentRegistry) {
        let a = registry.get_with_name(&component.type_name).unwrap().ty;
        for other in &mut *self {
            let b = registry.get_with_name(&other.type_name).unwrap().ty;
            if a == b {
                other.apply(&component);
                return;
            }
        }
        self.push(component);
    }
}

#[derive(Default, Debug)]
pub struct EditorBundle {
    components: Vec<DynamicProperties>,
}

impl EditorBundle {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, component: DynamicProperties, registry: &ComponentRegistry) -> &mut Self {
        self.components.add(component, registry);
        self
    }

    pub fn add_bundle<I: IntoIterator<Item = DynamicProperties>>(
        &mut self,
        other: I,
        registry: &ComponentRegistry,
    ) -> &mut Self {
        self.components.add_bundle(other, registry);
        self
    }

    pub fn into_inner(self) -> Vec<DynamicProperties> {
        self.components
    }

    pub fn to_dynamic(&self) -> DynamicProperties {
        let props = self
            .components
            .iter()
            .map(|comp| comp.clone_prop())
            .collect();
        let prop_names = vec![];
        let prop_indices = bevy_utils::HashMap::default();
        DynamicProperties {
            type_name: std::any::type_name::<Self>().to_string(),
            props,
            prop_names,
            prop_indices,
            property_type: PropertyType::Seq,
        }
    }

    pub fn from_dynamic(dynamic: &DynamicProperties) -> Self {
        // we don't check for type_name, because std::any::type_name is not reliable
        let components = dynamic
            .props
            .iter()
            .map(|prop| {
                let prop = prop.any().downcast_ref::<DynamicProperties>().unwrap();
                let type_name = prop.type_name.clone();
                let props = prop.props.iter().map(|prop| prop.clone_prop()).collect();
                let prop_names = prop.prop_names.clone();
                let prop_indices = prop.prop_indices.clone();
                let property_type = prop.property_type;
                DynamicProperties {
                    type_name,
                    props,
                    prop_names,
                    prop_indices,
                    property_type,
                }
            })
            .collect();
        Self { components }
    }
}

impl Clone for EditorBundle {
    fn clone(&self) -> Self {
        EditorBundle {
            components: self
                .components
                .iter()
                .map(Property::clone_prop)
                .map(|prop| prop.as_properties().unwrap().to_dynamic())
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefaultBundles {
    map: HashMap<String, EditorBundle>,
}

impl DefaultBundles {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("LightComponents".to_string(), {
            let mut bundle = EditorBundle::default();
            bundle.components.push(Light::default().to_dynamic());
            bundle.components.push(Transform::default().to_dynamic());
            bundle
                .components
                .push(DefaultComponent::<GlobalTransform>::default().to_dynamic());
            bundle
        });
        map.insert("PbrComponents".to_string(), {
            let mut bundle = EditorBundle::default();
            bundle
                .components
                .push(Asset::<Mesh>::default().to_dynamic());
            bundle
                .components
                .push(IntoAsset::<Color, StandardMaterial>::default().to_dynamic());
            bundle.components.push(Draw::default().to_dynamic());
            bundle.components.push(MainPass::default().to_dynamic());
            bundle.components.push(
                RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
                    FORWARD_PIPELINE_HANDLE,
                    PipelineSpecialization {
                        dynamic_bindings: vec![
                            // Transform
                            DynamicBinding {
                                bind_group: 2,
                                binding: 0,
                            },
                            // StandardMaterial_albedo
                            DynamicBinding {
                                bind_group: 3,
                                binding: 0,
                            },
                        ],
                        ..Default::default()
                    },
                )])
                .to_dynamic(),
            );
            bundle.components.push(Transform::default().to_dynamic());
            bundle
                .components
                .push(DefaultComponent::<GlobalTransform>::default().to_dynamic());
            bundle
        });
        Self { map }
    }

    pub fn get(&self, name: &str) -> Option<EditorBundle> {
        self.map.get(name).cloned()
    }

    pub fn to_dynamic(&self) -> DynamicProperties {
        let props = self
            .map
            .iter()
            .map(|(_, comp)| comp.to_dynamic().clone_prop())
            .collect();
        let prop_names = self
            .map
            .iter()
            .map(|(key, _)| key.to_string().into())
            .collect();
        let prop_indices = self
            .map
            .iter()
            .enumerate()
            .map(|(i, (key, _))| (key.to_string().into(), i))
            .collect();
        DynamicProperties {
            type_name: std::any::type_name::<Self>().to_string(),
            props,
            prop_names,
            prop_indices,
            property_type: PropertyType::Map,
        }
    }

    pub fn from_dynamic(dynamic: &DynamicProperties) -> Self {
        // we don't check for type_name, because std::any::type_name is not reliable
        let map = dynamic
            .props
            .iter()
            .map(|prop| {
                let prop = prop.any().downcast_ref::<DynamicProperties>().unwrap();
                (prop.type_name.clone(), EditorBundle::from_dynamic(prop))
            })
            .collect();
        Self { map }
    }
}

impl Default for DefaultBundles {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct DefaultProperties {
    map: HashMap<String, DynamicProperties>,
}

impl DefaultProperties {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("Light".to_string(), Light::default().to_dynamic());
        map.insert("Transform".to_string(), Transform::default().to_dynamic());
        map.insert(
            "GlobalTransform".to_string(),
            DefaultComponent::<GlobalTransform>::default().to_dynamic(),
        );
        map.insert(
            "Asset<Mesh>".to_string(),
            Asset::<Mesh>::default().to_dynamic(),
        );
        map.insert(
            "Into<Color, StandardMaterial>".to_string(),
            IntoAsset::<Color, StandardMaterial>::default().to_dynamic(),
        );
        map.insert("Draw".to_string(), Draw::default().to_dynamic());
        map.insert("MainPass".to_string(), MainPass::default().to_dynamic());
        map.insert(
            "RenderPipelines".to_string(),
            RenderPipelines::default().to_dynamic(),
        );
        Self { map }
    }

    pub fn get(&self, name: &str) -> Option<DynamicProperties> {
        self.map
            .get(name)
            .map(|props| props.clone_prop().as_properties().unwrap().to_dynamic())
    }

    pub fn to_dynamic(&self) -> DynamicProperties {
        let props = self.map.iter().map(|(_, comp)| comp.clone_prop()).collect();
        let prop_names = self
            .map
            .iter()
            .map(|(key, _)| key.to_string().into())
            .collect();
        let prop_indices = self
            .map
            .iter()
            .enumerate()
            .map(|(i, (key, _))| (key.to_string().into(), i))
            .collect();
        DynamicProperties {
            type_name: std::any::type_name::<Self>().to_string(),
            props,
            prop_names,
            prop_indices,
            property_type: PropertyType::Map,
        }
    }

    pub fn from_dynamic(dynamic: &DynamicProperties) -> Self {
        // we don't check for type_name, because std::any::type_name is not reliable
        let map = dynamic
            .props
            .iter()
            .map(|prop| {
                let prop = prop.any().downcast_ref::<DynamicProperties>().unwrap();
                let type_name = prop.type_name.clone();
                let props = prop.props.iter().map(|prop| prop.clone_prop()).collect();
                let prop_names = prop.prop_names.clone();
                let prop_indices = prop.prop_indices.clone();
                let property_type = prop.property_type;
                let prop = DynamicProperties {
                    type_name,
                    props,
                    prop_names,
                    prop_indices,
                    property_type,
                };
                (prop.type_name.clone(), prop)
            })
            .collect();
        Self { map }
    }
}

impl Clone for DefaultProperties {
    fn clone(&self) -> Self {
        DefaultProperties {
            map: self
                .map
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.clone_prop().as_properties().unwrap().to_dynamic(),
                    )
                })
                .collect(),
        }
    }
}

impl Default for DefaultProperties {
    fn default() -> Self {
        Self::new()
    }
}
