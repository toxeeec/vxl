use bevy::prelude::*;

use crate::{
    materials::BlockOverlayMaterial,
    sets::GameplaySet,
    state::AppState,
    textures::{BlockOverlayTexture, BlocksTexture},
    world::Chunks,
};

const CUBOID_UVS: [[[f32; 2]; 4]; 6] = [
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
    [[1.0, 1.0], [1.0, 0.0], [0.0, 0.0], [0.0, 1.0]],
    [[1.0, 1.0], [1.0, 0.0], [0.0, 0.0], [0.0, 1.0]],
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
];

#[derive(Component, Debug)]
struct BlockOverlay;

pub(super) struct BlockOverlayPlugin;

impl Plugin for BlockOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), Self::spawn_overlay)
            .add_systems(Update, Self::update_overlay.in_set(GameplaySet));
    }
}

impl BlockOverlayPlugin {
    const REACH: f32 = 4.5;

    fn spawn_overlay(
        mut commands: Commands,
        overlay: Res<BlockOverlayTexture>,
        blocks: Res<BlocksTexture>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<BlockOverlayMaterial>>,
    ) {
        let mesh = Mesh::from(Cuboid::from_length(1.0))
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, CUBOID_UVS.as_flattened().to_vec());

        commands.spawn((
            MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: materials.add(BlockOverlayMaterial::new(&overlay.0, &blocks.0)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            BlockOverlay,
        ));
    }

    fn update_overlay(
        mut set: ParamSet<(
            Query<&Transform, With<Camera>>,
            Query<
                (
                    &mut Transform,
                    &mut Visibility,
                    &Handle<BlockOverlayMaterial>,
                ),
                With<BlockOverlay>,
            >,
        )>,

        chunks: Res<Chunks>,
        mut materials: ResMut<Assets<BlockOverlayMaterial>>,
    ) {
        let q_camera = set.p0();
        let camera = q_camera.single();
        let translation = camera.translation;
        let direction = camera.rotation * Vec3::NEG_Z;

        let mut q_overlay = set.p1();
        let (mut transform, mut visibility, handle) = q_overlay.single_mut();

        match chunks.traverse(Ray3d::new(translation, direction), Self::REACH) {
            Some((pos, block_id)) => {
                *transform = Transform::from_translation(pos.as_vec3() + Vec3::splat(0.5));
                *visibility = Visibility::Visible;
                let material = materials.get_mut(handle).unwrap();
                material.block_id = block_id as u32;
            }
            None => {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
