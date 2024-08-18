use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage},
        view::{ViewTarget, ViewUniform, ViewUniforms},
        Extract, RenderApp,
    },
};
use binding_types::uniform_buffer;

use crate::textures::CrosshairTexture;

#[derive(Resource)]
struct BackgroundTexture {
    texture: Texture,
    vertices: RawBufferVec<[f32; 3]>,
    indices: RawBufferVec<u32>,
}

#[derive(Default, Debug)]
struct CrosshairNode;

#[derive(Hash, PartialEq, Eq, Clone, RenderLabel, Debug)]
struct CrosshairLabel;

#[derive(Resource, Debug)]
struct CrosshairPipeline {
    view_layout: BindGroupLayout,
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

#[derive(Debug)]
pub(super) struct CrosshairPlugin;

impl ViewNode for CrosshairNode {
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        view_target: QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(crosshair) = world.get_resource::<CrosshairTexture>() else {
            return Ok(());
        };

        let Some(background) = world.get_resource::<BackgroundTexture>() else {
            return Ok(());
        };

        let crosshair_pipeline = world.resource::<CrosshairPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache
            .get_render_pipeline(crosshair_pipeline.pipeline_id)
            .unwrap();

        let gpu_images = world.resource::<RenderAssets<GpuImage>>();
        let crosshair_image = gpu_images.get(&crosshair.0).unwrap();

        let view_uniforms = world.resource::<ViewUniforms>();
        let view_binding = view_uniforms.uniforms.binding().unwrap();

        let view_bind_group = render_context.render_device().create_bind_group(
            "crosshair_view_bind_group",
            &crosshair_pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        );

        copy_to_background_texture(render_context, view_target, &background.texture);

        let bind_group = render_context.render_device().create_bind_group(
            "crosshair_bind_group",
            &crosshair_pipeline.layout,
            &BindGroupEntries::sequential((
                &background
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
                &crosshair_pipeline.sampler,
                &crosshair_image.texture_view,
                &crosshair_image.sampler,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("crosshair_pass"),
            color_attachments: &[Some(view_target.get_color_attachment())],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);

        render_pass.set_bind_group(0, &view_bind_group, &[0]);
        render_pass.set_bind_group(1, &bind_group, &[]);

        render_pass.set_vertex_buffer(0, background.vertices.buffer().unwrap().slice(..));
        render_pass.set_index_buffer(
            background.indices.buffer().unwrap().slice(..),
            0,
            IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..background.indices.len() as u32, 0, 0..1);

        Ok(())
    }
}

impl FromWorld for CrosshairPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(
            "crosshair_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        let layout = render_device.create_bind_group_layout(
            "crosshair_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.load_asset("shaders/crosshair.wgsl");
        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("crosshair_pipeline".into()),
                    layout: vec![view_layout.clone(), layout.clone()],
                    vertex: VertexState {
                        shader: shader.clone(),
                        shader_defs: vec![],
                        entry_point: "vertex".into(),
                        buffers: vec![VertexBufferLayout::from_vertex_formats(
                            VertexStepMode::Vertex,
                            vec![VertexFormat::Float32x3],
                        )],
                    },
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                });

        Self {
            view_layout,
            layout,
            sampler,
            pipeline_id,
        }
    }
}

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_systems(
                ExtractSchedule,
                Self::create_background_texture.run_if(resource_exists::<CrosshairTexture>),
            )
            .add_render_graph_node::<ViewNodeRunner<CrosshairNode>>(Core3d, CrosshairLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    CrosshairLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<CrosshairPipeline>();
    }
}

impl CrosshairPlugin {
    fn create_background_texture(
        mut commands: Commands,
        query: Extract<Query<&Camera>>,
        background: Option<Res<BackgroundTexture>>,
        images: Extract<Res<Assets<Image>>>,
        crosshair: Res<CrosshairTexture>,
        render_device: Res<RenderDevice>,
        render_queue: Res<RenderQueue>,
    ) {
        if background.is_some() {
            return;
        }

        let Ok(camera) = query.get_single() else {
            return;
        };

        let crosshair_image = images.get(&crosshair.0).unwrap();
        let size = crosshair_image.size_f32();
        let scale_factor = camera.target_scaling_factor().unwrap_or(1.0);

        let texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: (crosshair_image.width() as f32 * scale_factor) as u32,
                height: (crosshair_image.height() as f32 * scale_factor) as u32,
                ..Default::default()
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut vertices = RawBufferVec::new(BufferUsages::VERTEX);
        let mut indices = RawBufferVec::new(BufferUsages::INDEX);

        let Vec2 { x: hw, y: hh } = size / 2.0;
        vertices.extend([
            [-hw, -hh, 0.0],
            [hw, -hh, 0.0],
            [hw, hh, 0.0],
            [-hw, hh, 0.0],
        ]);
        indices.extend([0, 1, 2, 0, 2, 3]);

        vertices.write_buffer(&render_device, &render_queue);
        indices.write_buffer(&render_device, &render_queue);

        commands.insert_resource(BackgroundTexture {
            texture,
            vertices,
            indices,
        });
    }
}

fn copy_to_background_texture(
    render_context: &mut RenderContext,
    view_target: &ViewTarget,
    background_texture: &Texture,
) {
    let main_texture = view_target.main_texture();

    let mut encoder =
        render_context
            .render_device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("background_texture_encoder"),
            });

    encoder.copy_texture_to_texture(
        ImageCopyTexture {
            texture: view_target.main_texture(),
            mip_level: 0,
            origin: Origin3d {
                x: main_texture.width() / 2 - background_texture.width() / 2,
                y: main_texture.height() / 2 - background_texture.height() / 2,
                z: 0,
            },
            aspect: TextureAspect::All,
        },
        ImageCopyTexture {
            texture: background_texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        background_texture.size(),
    );

    render_context.add_command_buffer(encoder.finish());
}
