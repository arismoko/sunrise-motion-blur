//! The data the shader reads, laid out as its `Params` block declares it (std140)
//! (`tests/shaders.rs` holds them to it).

use bytemuck::{Pod, Zeroable};

/// `Params` in the shader: one frame's cameras and blur settings.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BlurParams {
    /// Camera-relative world to clip space, without TAA's jitter.
    pub view_proj: [f32; 16],
    /// Combined `previous_view_proj * translate(camera_motion) * inverse_view_proj`.
    pub reproject: [f32; 16],
    /// Last frame's `view_proj`.
    pub previous_view_proj: [f32; 16],
    /// Fourth row of `inverse_view_proj`, for 1-dot linear view depth.
    pub inv_view_proj_row3: [f32; 4],
    /// The picture's size and the world depth's, in pixels.
    pub size: [f32; 2],
    pub depth_size: [f32; 2],
    /// How much of a frame's motion becomes the streak.
    pub exposure: f32,
    /// The longest blur radius, in pixels.
    pub max_radius: f32,
    pub tile_size: u32,
    pub samples: u32,
    /// Monotonic frame counter for temporal noise dithering.
    pub frame: u32,
    /// Non-zero when the color buffer is non-sRGB Unorm and should be linearized during blending.
    pub linearize: u32,
    pub _pad: [u32; 2],
}

impl BlurParams {
    pub fn bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
