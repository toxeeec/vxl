use crate::texture::{ChunkMaterial, ChunkTexture};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Reflect, Debug)]
enum DebugAction {
    WireframeMode,
}

fn handle_debug_hotkeys(
    query: Query<&ActionState<DebugAction>>,
    chunk_texture: Res<ChunkTexture>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    let action_state = query.single();
    let mat = materials.get_mut(&chunk_texture.material).unwrap();

    if action_state.just_pressed(DebugAction::WireframeMode) {
        mat.wireframe_mode = !mat.wireframe_mode;
    }
}

#[derive(Debug)]
pub(super) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<DebugAction>::default())
            .add_systems(Update, handle_debug_hotkeys);

        app.world.spawn(InputManagerBundle::<DebugAction> {
            input_map: InputMap::new([(QwertyScanCode::Z, DebugAction::WireframeMode)]),
            ..Default::default()
        });
    }
}
