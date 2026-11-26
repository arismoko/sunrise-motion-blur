//! What each frame's blur needs: the camera's motion, decided on the CPU, and the passes that
//! turn it into a blur, recorded in the frame graph.
//!
//! The passes run on the GPU in this order, each reading the one before:
//!
//! 1. `motion`: every pixel's blur radius and view depth, from the world's depth and last
//!    frame's camera;
//! 2. `tile_rows`, `tile_columns`, `neighbor_max`: the longest motion in each tile and its
//!    neighbours.
//!
//! When the camera stands still there are no passes, and the GPU does nothing.

use crate::{pipelines, settings};
use mod_api::{
    ClientHost, DrawBindings, DrawRange, FrameGraphSetupEvent, FramePassDescriptor, FrameTarget,
    MeshDraw, RenderLevelStage, RenderTargetDescriptor, RenderTargetFormat, TextureSource,
};
use motion_blur_core::{BlurParams, CameraHistory, FrameClock, Mat4, SAMPLES};
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

/// The tile size bounds: the longest radius the blur reaches is one tile.
const MIN_TILE: u32 = 8;
const MAX_TILE: u32 = 64;

/// Bytes of one `PositionTexColor` vertex: `Position` (3 x f32), `UV0` (2 x f32), `Color` (4 x u8).
const VERTEX_BYTES: usize = 24;

/// One oversized triangle that covers the picture, as three `PositionTexColor` vertices.
static TRIANGLE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut bytes = Vec::with_capacity(3 * VERTEX_BYTES);
    for corner in [[-1.0f32, -1.0], [3.0, -1.0], [-1.0, 3.0]] {
        for value in [corner[0], corner[1], 0.0, 0.0, 0.0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&[255; 4]);
    }
    bytes
});

static FRAMES: LazyLock<Mutex<Frames>> = LazyLock::new(Default::default);

#[derive(Default)]
struct Frames {
    clock: FrameClock,
    camera: CameraHistory,
    frame: u32,
}

impl Frames {
    fn reset(&mut self) {
        self.clock.reset();
        self.camera.reset();
    }
}

fn frames() -> MutexGuard<'static, Frames> {
    FRAMES.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Forgets the last frame, so the next one starts afresh (a world opened or closed).
pub fn reset() {
    frames().reset();
}

/// Records this frame's blur passes, when the camera moved.
pub fn setup(host: &dyn ClientHost, event: &mut FrameGraphSetupEvent<'_>) {
    let settings = settings::current();
    let mut frames = frames();
    if !settings.active() {
        frames.reset();
        return;
    }
    let Some(size) = host
        .window_size()
        .filter(|size| size.iter().all(|&n| n > 0))
    else {
        return;
    };

    let camera = event.view.camera;
    let view_proj = Mat4(camera.projection).mul(&Mat4(camera.view));
    let frame_seconds = frames.clock.tick(Instant::now());
    let reprojection = frames.camera.advance(view_proj, camera.position);
    let (Some(frame_seconds), Some(reprojection)) = (frame_seconds, reprojection) else {
        return;
    };
    let Some(inverse) = view_proj.inverse() else {
        return;
    };
    if reprojection.is_still(&view_proj) {
        return;
    }

    let tile_size = (settings.max_radius(size[1]).ceil() as u32).clamp(MIN_TILE, MAX_TILE);
    let reproject = Mat4::reproject(
        &reprojection.previous_view_proj,
        reprojection.camera_motion,
        &inverse,
    );
    frames.frame = frames.frame.wrapping_add(1);
    let params = BlurParams {
        view_proj: view_proj.0,
        reproject: reproject.0,
        previous_view_proj: reprojection.previous_view_proj.0,
        inv_view_proj_row3: inverse.row(3),
        size: size.map(|n| n as f32),
        depth_size: size.map(|n| n as f32),
        exposure: settings.exposure(frame_seconds),
        max_radius: settings.max_radius(size[1]).min(tile_size as f32),
        tile_size,
        samples: SAMPLES,
        frame: frames.frame,
        linearize: 0,
        _pad: [0; 2],
    };
    drop(frames);
    record(event, size, &params);
}

fn record(event: &mut FrameGraphSetupEvent<'_>, size: [u32; 2], params: &BlurParams) {
    let tiles_wide = size[0].div_ceil(params.tile_size);
    let tiles_high = size[1].div_ceil(params.tile_size);
    let transient = |size: [u32; 2]| {
        event.graph.create_transient(
            &RenderTargetDescriptor::new(size).color_target(RenderTargetFormat::Rgba16Float),
        )
    };
    let (Ok(motion), Ok(tile_rows), Ok(tiles), Ok(neighbors)) = (
        transient(size),
        transient([tiles_wide, size[1]]),
        transient([tiles_wide, tiles_high]),
        transient([tiles_wide, tiles_high]),
    ) else {
        return;
    };

    let bytes = params.bytes();
    let steps = [
        (
            "motion_blur_motion",
            &pipelines::MOTION,
            motion,
            vec![("WorldDepth", TextureSource::WorldDepth)],
            vec![FrameTarget::MainDepth],
        ),
        (
            "motion_blur_tile_rows",
            &pipelines::TILE_ROWS,
            tile_rows,
            vec![("Motion", TextureSource::TransientTargetColor(motion, 0))],
            vec![FrameTarget::Transient(motion)],
        ),
        (
            "motion_blur_tile_columns",
            &pipelines::TILE_COLUMNS,
            tiles,
            vec![(
                "TileRows",
                TextureSource::TransientTargetColor(tile_rows, 0),
            )],
            vec![FrameTarget::Transient(tile_rows)],
        ),
        (
            "motion_blur_neighbor_max",
            &pipelines::NEIGHBOR_MAX,
            neighbors,
            vec![("Tiles", TextureSource::TransientTargetColor(tiles, 0))],
            vec![FrameTarget::Transient(tiles)],
        ),
    ];
    let mut previous = None;
    for (label, pass, target, textures, reads) in steps {
        let Some(pipeline) = pipelines::handle(pass) else {
            return;
        };
        let mut descriptor = FramePassDescriptor::new(label, FrameTarget::Transient(target))
            .clear_color_0([0.0; 4])
            .after_stage(RenderLevelStage::AfterTranslucentBlocks)
            .disable_culling();
        for read in reads {
            descriptor = descriptor.reads(read);
        }
        if let Some(previous) = previous {
            descriptor = descriptor.after_pass(previous);
        }
        let result = event.graph.add_pass(descriptor, &mut |context| {
            let mut bindings = DrawBindings::default().inline_uniform("Params", bytes);
            for (name, source) in &textures {
                bindings = bindings.texture(name, source.clone());
            }
            let draw = MeshDraw {
                range: DrawRange::Vertices(0..3),
                instances: 0..1,
            };
            if let Err(error) =
                context.draw_vertices(pipeline, &[TRIANGLE.as_slice()], None, draw, &bindings)
            {
                eprintln!("motion_blur: {label} was not drawn: {error:?}");
            }
        });
        match result {
            Ok(id) => previous = Some(id),
            Err(error) => {
                eprintln!("motion_blur: {label} was refused: {error}");
                return;
            }
        }
    }
}
