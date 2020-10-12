use bevy::pbr::{
    prelude::{Light, StandardMaterial},
    render_graph::FORWARD_PIPELINE_HANDLE,
};
use bevy::prelude::*;
use bevy::property::DynamicProperties;
use bevy::render::{
    draw::Draw,
    mesh::Mesh,
    pipeline::{DynamicBinding, PipelineSpecialization, RenderPipeline, RenderPipelines},
    render_graph::base::MainPass,
};
use bevy::type_registry::*;

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

#[derive(Default)]
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
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultBundle<'a>(pub &'a str);

impl<'a> DefaultBundle<'a> {
    pub fn default(self) -> Option<EditorBundle> {
        let bundle = match self.0 {
            "LightComponents" => {
                let mut bundle = EditorBundle::default();
                bundle.components.push(Light::default().to_dynamic());
                bundle.components.push(Transform::default().to_dynamic());
                bundle
                    .components
                    .push(DefaultComponent::<GlobalTransform>::default().to_dynamic());
                bundle
            }
            "PbrComponents" => {
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
            }
            _ => return None,
        };
        Some(bundle)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultProperty<'a>(pub &'a str);

impl<'a> DefaultProperty<'a> {
    pub fn default(self) -> Option<DynamicProperties> {
        let component = match self.0 {
            "Light" => Light::default().to_dynamic(),
            "Transform" => Transform::default().to_dynamic(),
            "GlobalTransform" => DefaultComponent::<GlobalTransform>::default().to_dynamic(),
            "Asset<Mesh>" => Asset::<Mesh>::default().to_dynamic(),
            "Into<Color, StandardMaterial>" => {
                IntoAsset::<Color, StandardMaterial>::default().to_dynamic()
            }
            "Draw" => Draw::default().to_dynamic(),
            "MainPass" => MainPass::default().to_dynamic(),
            "RenderPipelines" => RenderPipelines::default().to_dynamic(),
            _ => return None,
        };
        Some(component)
    }
}
