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

use super::*;

pub trait ComponentsExt {
    fn add(&mut self, component: DynamicProperties);

    fn add_bundle<I: IntoIterator<Item = DynamicProperties>>(&mut self, other: I) {
        for component in other {
            self.add(component);
        }
    }
}

impl ComponentsExt for Vec<DynamicProperties> {
    fn add(&mut self, component: DynamicProperties) {
        for other in &mut *self {
            if component.type_name == other.type_name {
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

    pub fn add(&mut self, component: DynamicProperties) -> &mut Self {
        self.components.add(component);
        self
    }

    pub fn add_bundle<I: IntoIterator<Item = DynamicProperties>>(&mut self, other: I) -> &mut Self {
        self.components.add_bundle(other);
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
                bundle.add(Light::default().to_dynamic());
                bundle.add(Transform::default().to_dynamic());
                bundle.add(DefaultComponent::<GlobalTransform>::default().to_dynamic());
                bundle
            }
            "PbrComponents" => {
                let mut bundle = EditorBundle::default();
                bundle.add(Asset::<Mesh>::default().to_dynamic());
                bundle.add(IntoAsset::<Color, StandardMaterial>::default().to_dynamic());
                bundle.add(Draw::default().to_dynamic());
                bundle.add(MainPass::default().to_dynamic());
                bundle.add(
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
                bundle.add(Transform::default().to_dynamic());
                bundle.add(DefaultComponent::<GlobalTransform>::default().to_dynamic());
                bundle
            }
            _ => return None,
        };
        Some(bundle)
    }
}
