use array_init::array_init;
use bevy::{asset::LoadState, prelude::*};
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use serde::Deserialize;
use splines::{Interpolation, Key, Spline};

use crate::{block::BlockId, toml_asset::TomlAsset};

use super::{Chunk, WorldPlugin, CHUNK_HEIGHT, CHUNK_WIDTH};

#[derive(Resource, Clone, Debug)]
pub(crate) struct Noise {
    density: Fbm<Perlin>,
    hilliness: Fbm<Perlin>,
}

#[derive(Resource, Clone, Debug)]
pub(crate) struct WorldgenParams {
    height_bias: f64,
    hilliness: Spline<f64, f64>,
}

#[derive(Resource, Debug)]
pub(super) struct LoadingWorldgenParams {
    is_loaded: bool,
    handle: Handle<TomlAsset>,
}

#[derive(Deserialize, Debug)]
struct Hilliness(Vec<[f64; 2]>);

impl Noise {
    fn new(density: Fbm<Perlin>, hilliness: Fbm<Perlin>) -> Self {
        Self { density, hilliness }
    }

    pub(crate) fn hilliness(&self) -> &Fbm<Perlin> {
        &self.hilliness
    }
}

impl Default for Noise {
    fn default() -> Self {
        let density = Fbm::<Perlin>::new(0).set_frequency(0.005);
        let hilliness = Fbm::<Perlin>::new(0).set_frequency(0.0005);
        Self::new(density, hilliness)
    }
}

impl WorldgenParams {
    pub(super) fn new(height_bias: f64, hilliness: Spline<f64, f64>) -> Self {
        Self {
            height_bias,
            hilliness,
        }
    }
}

impl Chunk {
    const MIN_HEIGHT: usize = 32;
    const MIN_GRASS_LAYERS: i32 = 3;
    const MAX_GRASS_LAYERS: i32 = 6;

    pub(super) fn generate(offset: IVec2, noise: &Noise, params: &WorldgenParams) -> Self {
        let mut hilliness_cache = vec![f64::MAX; CHUNK_HEIGHT * CHUNK_HEIGHT];
        let mut chunk = array_init(|i| {
            let x = i % CHUNK_WIDTH;
            let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
            let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

            let pos = IVec3::new(
                x as i32 + offset.x * CHUNK_WIDTH as i32,
                y as i32,
                z as i32 + offset.y * CHUNK_WIDTH as i32,
            );

            let mut hilliness = hilliness_cache[x + z * CHUNK_WIDTH];
            if hilliness == f64::MAX {
                hilliness = params
                    .hilliness
                    .clamped_sample((noise.hilliness.get([pos.x as f64, pos.z as f64]) + 1.0) / 2.0)
                    .unwrap();
                hilliness_cache[x + z * CHUNK_WIDTH] = hilliness;
            }

            let height_offset = hilliness * (CHUNK_HEIGHT - Self::MIN_HEIGHT) as f64;
            let elevation = pos.y as f64 / (Self::MIN_HEIGHT as f64 + height_offset);

            let mut density = noise.density.get(pos.as_dvec3().to_array());
            density -= ((elevation - 0.5) * params.height_bias).tanh();

            if density > 0.0 {
                BlockId::Stone
            } else {
                BlockId::Air
            }
        });

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let mut layer = Self::MAX_GRASS_LAYERS;
                for y in (0..CHUNK_HEIGHT).rev() {
                    let elevation = y as f64 / CHUNK_HEIGHT as f64;
                    let max_layers = (((1.0 - elevation) * Self::MAX_GRASS_LAYERS as f64).round()
                        as i32)
                        .clamp(Self::MIN_GRASS_LAYERS, Self::MAX_GRASS_LAYERS);
                    layer = layer.min(max_layers);

                    let i = x + (y * CHUNK_WIDTH * CHUNK_WIDTH) + z * CHUNK_WIDTH;
                    let block = chunk[i];
                    match block {
                        BlockId::Air => layer += 1,
                        BlockId::Stone => {
                            if layer > 0 {
                                chunk[i] = if chunk
                                    .get(i + CHUNK_WIDTH * CHUNK_WIDTH)
                                    .is_some_and(|block| block.is_opaque())
                                {
                                    BlockId::Dirt
                                } else {
                                    BlockId::Grass
                                };
                            }
                            layer -= 1
                        }
                        _ => unreachable!(),
                    };
                }
            }
        }

        Self(chunk)
    }
}

impl WorldPlugin {
    pub(super) fn setup_loading_worldgen_params(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands.insert_resource(LoadingWorldgenParams {
            is_loaded: false,
            handle: asset_server.load("worldgen.toml"),
        });
    }

    pub(super) fn load_worldgen_params(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut loading_params: ResMut<LoadingWorldgenParams>,
        mut toml_assets: ResMut<Assets<TomlAsset>>,
    ) {
        if loading_params.is_loaded
            || asset_server.load_state(loading_params.handle.clone()) != LoadState::Loaded
        {
            return;
        }
        loading_params.is_loaded = true;

        let table = toml_assets.get_mut(&loading_params.handle).unwrap();

        let height_bias = table
            .0
            .get("height_bias")
            .expect("`height_bias` is required")
            .as_float()
            .filter(|v| *v > 0.0)
            .expect("invalid value for `height_bias`");

        let hilliness = Hilliness::deserialize(
            table
                .0
                .get("hilliness")
                .expect("`hilliness` is required")
                .clone(),
        )
        .expect("invalid value for `hilliness`");

        for &[x, y] in &hilliness.0 {
            if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
                panic!("invalid value for `hilliness`");
            }
        }

        let hilliness = Spline::from_iter(
            hilliness
                .0
                .iter()
                .map(|&[x, y]| Key::new(x, y, Interpolation::Cosine)),
        );

        commands.insert_resource(WorldgenParams::new(height_bias, hilliness));
    }
}
