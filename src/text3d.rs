use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::{Component, DespawnRecursiveExt, Entity},
};

use crate::{
    styling::SegmentStyle, Text3dBounds, Text3dDimensionOut, Text3dStyling, TextAtlasHandle,
};

/// A rich text component.
#[derive(Debug, Component)]
#[require(Text3dDimensionOut, Text3dBounds, TextAtlasHandle, Text3dStyling)]
#[component(on_remove = text_3d_on_remove)]
pub struct Text3d {
    pub segments: Vec<(Text3dSegment, SegmentStyle)>,
}

/// A string segment in [`Text3d`].
#[derive(Debug)]
pub enum Text3dSegment {
    String(String),
    Extract(Entity),
}

fn text_3d_on_remove(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let Ok(entity) = world.get_entity(entity) else {
        return;
    };
    let Some(text) = entity.get::<Text3d>() else {
        return;
    };
    let to_be_dropped: Vec<_> = text
        .segments
        .iter()
        .filter_map(|x| match &x.0 {
            Text3dSegment::String(_) => None,
            Text3dSegment::Extract(entity) => Some(*entity),
        })
        .collect();
    let mut commands = world.commands();
    for entity in to_be_dropped {
        commands.entity(entity).try_despawn_recursive();
    }
}

impl Text3d {
    /// Create a simple string without parsing.
    ///
    /// To parse rich text, see [`Text3d::parse`].
    pub fn new(s: impl ToString) -> Self {
        let string = s.to_string();
        Self {
            segments: vec![(Text3dSegment::String(string), Default::default())],
        }
    }
}
