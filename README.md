# bevy_prefab_editor
My idea for a prefab editor and a new prefab format for Bevy

## Requirements, ideas and postulate

### Prefabs

1. Postulate
 - Scenes are not built in this editor, prefabs are
  - Why? The designer should not build the status quo, they should build a recipe,
    assisted by code, which "generates" the scene. Imagine you want to build a forest scene:
    it would be a waste of time to place a thousand trees and a waste of space to generate
    those trees while editing. Instead the prefab should contain an entity with a component
    which upon loading the scene generates the trees. This approach could generate the same
    exact set of trees if we simply provide a seed value to this generator component.
2. Postulate
 - A prefab should be able to load an arbitrary component or resource by some unique 
   identifier, but not a plugin or a system and never from a dynamic library.
  - Why? Firstly prefabs should be able to define what components and resources they need to
    function properly. They should **not** define how other things might work, for example
    through loading plugins or systems. Secondly, allowing dynamic libraries to be loaded in
    prefabs is a security issue.

- Prefabs may be included in other prefabs, as long as there are no cyclic dependencies.
- Needs to support both 2D and 3D modes
- Blender-like transforms
 - Key + axis + number to transform precisely, and/or
 - Key makes a transform widget appear in the 2D/3D view
- Blender-like adding of entities
- Immediate visual feedback

#### The prefab (pseudo-) format

Comments are not necessarily part of the syntax, but they are provided for analysis.

```
# both asset and mesh are generic builtin "functions"
character_mesh: asset<Mesh>("assets/mesh/character.gltf"),
character_material: add<StandardMaterial>(StandardMaterial {
  albedo: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
  albedo_texture: Option<Handle<Texture>>::None,
  shaded: true,
}),
# character is the prefab internal name of this entity, it does not leave the prefab editor/loader
# 1 is the entity id
character: Entity<1> {
  components: [
    # `RigidBody` will initialize a default RigidBody
    RigidBody
      # this will set its mass to 1.0
      .with_mass(1.0)
      # this will set its position to vec3(5.0, 20.0, 0.0), where vec3 is a builtin "function"
      .with_position(vec3(5.0, 20.0, 0.0)),
    # anything that starts with an uppercase is a builtin component, Entity or other struct
    Transform,
    GlobalTransform,
    # anything that starts with a lowercase is a variable
    character_mesh,
    character_material,
    MainPass,
    Draw,
    RenderPipelines,
  ],
  children: [],
},
```
