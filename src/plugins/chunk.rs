use crate::block::BlockBundle;
use crate::chunk::{Chunk, INDICES_CAPACITY, VERTICES_CAPACITY};
use crate::chunk::{CHUNK_AREA, CHUNK_VOLUME, CHUNK_WIDTH};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

pub(crate) struct ChunkPlugin;

impl ChunkPlugin {
    fn spawn_chunk(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
        let pbr = PbrBundle {
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                unlit: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        commands.spawn((pbr, Chunk)).with_children(|chunk| {
            for i in 0..CHUNK_VOLUME {
                chunk.spawn(BlockBundle::new(
                    Vec3 {
                        x: (i % CHUNK_WIDTH) as f32,
                        y: (i / CHUNK_AREA) as f32,
                        z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH) as f32,
                    },
                    i < CHUNK_AREA * 4,
                ));
            }
        });
    }

    fn mesh_chunks(
        mut q_chunk: Query<(&mut Handle<Mesh>, &Children), (With<Chunk>, Added<Handle<Mesh>>)>,
        q_block: Query<(&Transform, &Visibility)>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (mut mesh_handle, blocks) in q_chunk.iter_mut() {
            let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
            let mut indices = Vec::with_capacity(INDICES_CAPACITY);
            Chunk::mesh(blocks, &q_block, &mut positions, &mut indices);

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.set_indices(Some(Indices::U32(indices)));
            *mesh_handle = meshes.add(mesh);
        }
    }
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ChunkPlugin::spawn_chunk)
            .add_systems(Update, ChunkPlugin::mesh_chunks);
    }
}
