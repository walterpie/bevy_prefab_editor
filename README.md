# bevy_prefab_editor
My idea for a prefab editor and a new prefab format for Bevy

## Update

My ideas were, for the most part, already present in Bevy.  By addint tiny amounts
of my own sauce to it, I managed to get a working example of saving and loading a
prefab with a single "pbr" entity.

I will continue this work by creating a bevy prefab editor that can edit:
- [ ] 2D bevy_sprite-based scenes
- [ ] 3D bevy_pbr-based scenes
- [ ] UI?
All of these editors should allow the user to load arbitrary components, so the API
has to be generic over the components used.

The example that is currently in place creates a separate world and type registry
and serializes everything within it.

The "actual" editor will most likely keep track of a `Scene` and update it on the
fly, as well as keep track of its own entities with a `SpriteComponents`,
`PbrComponents` or `UiComponents` and co. which will be stored in the main world.

So any operation will update two-fold: the internal `Scene` as well as the external
world visible to the user.
