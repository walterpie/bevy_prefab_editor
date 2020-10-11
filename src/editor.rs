use bevy::prelude::*;
use bevy_mod_picking::*;

#[derive(Default, Debug, Clone, Copy)]
pub struct Widget(pub u32);

#[derive(Bundle)]
pub struct WidgetComponents {
    pub widget: Widget,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub pickable: PickableMesh,
    pub highlight: HighlightablePickMesh,
    pub select: SelectablePickMesh,
}

impl WidgetComponents {
    pub fn new(e: u32) -> Self {
        Self {
            widget: Widget(e),
            transform: Default::default(),
            global_transform: Default::default(),
            pickable: Default::default(),
            highlight: Default::default(),
            select: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EditorEvent {
    Translate(Vec3),
    Rotate(Quat),
    Scale(Vec3),
}
