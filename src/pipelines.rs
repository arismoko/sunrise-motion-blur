//! The blur's render pipelines: one full-screen GLSL program per pass.

use mod_api::{
    Identifier, PipelineBlend, RegisterRenderPipelinesEvent, RenderAssets, RenderPipeline,
    RenderPipelineRef, SamplerAddress, SamplerFilter, WorldDepth,
};
use std::sync::{PoisonError, RwLock};

/// Bytes of `Params` (`BlurParams` in `motion-blur-core`).
const PARAMS_SIZE: u32 = std::mem::size_of::<motion_blur_core::BlurParams>() as u32;

/// What a pass reads: its pipeline, then the texture names its GLSL samples.
pub struct Pass {
    pub id: &'static str,
    pub samplers: &'static [(&'static str, SamplerFilter)],
}

pub const MOTION: Pass = Pass {
    id: "motion_blur:motion",
    samplers: &[("WorldDepth", SamplerFilter::Nearest)],
};
pub const TILE_ROWS: Pass = Pass {
    id: "motion_blur:tile_rows",
    samplers: &[("Motion", SamplerFilter::Nearest)],
};
pub const TILE_COLUMNS: Pass = Pass {
    id: "motion_blur:tile_columns",
    samplers: &[("TileRows", SamplerFilter::Nearest)],
};
pub const NEIGHBOR_MAX: Pass = Pass {
    id: "motion_blur:neighbor_max",
    samplers: &[("Tiles", SamplerFilter::Nearest)],
};
pub const RECONSTRUCT: Pass = Pass {
    id: "motion_blur:reconstruct",
    samplers: &[
        ("Scene", SamplerFilter::Linear),
        ("Motion", SamplerFilter::Nearest),
        ("NeighborTiles", SamplerFilter::Nearest),
    ],
};

const PASSES: [&Pass; 5] = [
    &MOTION,
    &TILE_ROWS,
    &TILE_COLUMNS,
    &NEIGHBOR_MAX,
    &RECONSTRUCT,
];

/// The resolved handles, in `PASSES` order.
static RESOLVED: RwLock<Option<[RenderPipelineRef; 5]>> = RwLock::new(None);

fn id(text: &str) -> Option<Identifier> {
    Identifier::parse(text).ok()
}

pub fn register(event: &mut RegisterRenderPipelinesEvent<'_>) {
    let Some(vertex) = id("motion_blur:core/fullscreen") else {
        return;
    };
    for pass in PASSES {
        let name = pass.id.trim_start_matches("motion_blur:");
        let (Some(pipeline_id), Some(shader)) =
            (id(pass.id), id(&format!("motion_blur:core/{name}")))
        else {
            continue;
        };
        // The passes write colour only: no world depth is tested or written.
        let mut pipeline = RenderPipeline::new(
            pipeline_id,
            shader,
            PipelineBlend::Opaque,
            WorldDepth::ThroughWalls,
        )
        .vertex_shader(vertex.clone())
        .depth_stencil(None)
        .cull(false)
        .uniform_buffer("Params", PARAMS_SIZE);
        for &(sampler, filter) in pass.samplers {
            pipeline = pipeline.sampler(sampler, filter, SamplerAddress::ClampToEdge);
        }
        event.register(pipeline);
    }
}

/// Looks the pipelines up again, at setup and after each resource reload.
pub fn resolve(assets: &dyn RenderAssets) {
    let handles = PASSES.map(|pass| id(pass.id).and_then(|id| assets.render_pipeline(&id).ok()));
    let resolved = match handles {
        [Some(a), Some(b), Some(c), Some(d), Some(e)] => Some([a, b, c, d, e]),
        _ => None,
    };
    *RESOLVED.write().unwrap_or_else(PoisonError::into_inner) = resolved;
}

/// The handle of the pass at `index` in `PASSES` order, `None` until all resolved.
pub fn handle(pass: &Pass) -> Option<RenderPipelineRef> {
    let index = PASSES.iter().position(|p| std::ptr::eq(*p, pass))?;
    RESOLVED
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .map(|handles| handles[index])
}
