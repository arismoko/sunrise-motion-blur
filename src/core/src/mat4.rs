/// A 4×4 matrix in column-major order, as GLSL's `mat4` lays it out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4(pub [f32; 16]);

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4([
        1.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// The inverse, or `None` when the matrix has none (a broken camera).
    ///
    /// Cofactor expansion in f64: a projection's inverse loses digits fast in f32, and this runs
    /// once a frame.
    pub fn inverse(&self) -> Option<Mat4> {
        let m: [f64; 16] = self.0.map(f64::from);
        let mut inv = [0.0f64; 16];
        inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
            + m[9] * m[7] * m[14]
            + m[13] * m[6] * m[11]
            - m[13] * m[7] * m[10];
        inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
            - m[8] * m[7] * m[14]
            - m[12] * m[6] * m[11]
            + m[12] * m[7] * m[10];
        inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
            + m[8] * m[7] * m[13]
            + m[12] * m[5] * m[11]
            - m[12] * m[7] * m[9];
        inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
            - m[8] * m[6] * m[13]
            - m[12] * m[5] * m[10]
            + m[12] * m[6] * m[9];
        inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
            - m[9] * m[3] * m[14]
            - m[13] * m[2] * m[11]
            + m[13] * m[3] * m[10];
        inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
            + m[8] * m[3] * m[14]
            + m[12] * m[2] * m[11]
            - m[12] * m[3] * m[10];
        inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
            - m[8] * m[3] * m[13]
            - m[12] * m[1] * m[11]
            + m[12] * m[3] * m[9];
        inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
            + m[8] * m[2] * m[13]
            + m[12] * m[1] * m[10]
            - m[12] * m[2] * m[9];
        inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
            + m[5] * m[3] * m[14]
            + m[13] * m[2] * m[7]
            - m[13] * m[3] * m[6];
        inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
            - m[4] * m[3] * m[14]
            - m[12] * m[2] * m[7]
            + m[12] * m[3] * m[6];
        inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
            + m[4] * m[3] * m[13]
            + m[12] * m[1] * m[7]
            - m[12] * m[3] * m[5];
        inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
            - m[4] * m[2] * m[13]
            - m[12] * m[1] * m[6]
            + m[12] * m[2] * m[5];
        inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
            - m[5] * m[3] * m[10]
            - m[9] * m[2] * m[7]
            + m[9] * m[3] * m[6];
        inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
            + m[4] * m[3] * m[10]
            + m[8] * m[2] * m[7]
            - m[8] * m[3] * m[6];
        inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
            - m[4] * m[3] * m[9]
            - m[8] * m[1] * m[7]
            + m[8] * m[3] * m[5];
        inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
            + m[4] * m[2] * m[9]
            + m[8] * m[1] * m[6]
            - m[8] * m[2] * m[5];

        let det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        Some(Mat4(inv.map(|v| (v / det) as f32)))
    }

    /// `self * other`.
    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let (a, b) = (&self.0, &other.0);
        let mut out = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                out[col * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[col * 4 + k]).sum();
            }
        }
        Mat4(out)
    }

    /// A translation matrix by `[x, y, z]`.
    pub fn translation([x, y, z]: [f32; 3]) -> Mat4 {
        Mat4([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            x, y, z, 1.0,
        ])
    }

    /// Row `r` (`0..4`) of this column-major matrix.
    pub fn row(&self, r: usize) -> [f32; 4] {
        [self.0[r], self.0[4 + r], self.0[8 + r], self.0[12 + r]]
    }

    /// Combined clip-to-previous-clip reprojection matrix:
    /// `previous_view_proj * translation(camera_motion) * inverse_view_proj`,
    /// multiplied in `f64` to preserve precision for distant geometry and sky (`depth == 0`).
    pub fn reproject(previous: &Mat4, camera_motion: [f32; 3], inverse: &Mat4) -> Mat4 {
        let mul64 = |a: &[f64; 16], b: &[f64; 16]| {
            let mut out = [0.0f64; 16];
            for col in 0..4 {
                for row in 0..4 {
                    out[col * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[col * 4 + k]).sum();
                }
            }
            out
        };
        let p = previous.0.map(f64::from);
        let t = Mat4::translation(camera_motion).0.map(f64::from);
        let inv = inverse.0.map(f64::from);
        Mat4(mul64(&mul64(&p, &t), &inv).map(|v| v as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reversed-Z, infinite-far perspective like the game's, times a turned camera.
    fn camera() -> Mat4 {
        let f = 1.0 / (35f32.to_radians()).tan();
        let projection = Mat4([
            f / 1.7,
            0.0,
            0.0,
            0.0, //
            0.0,
            f,
            0.0,
            0.0, //
            0.0,
            0.0,
            0.0,
            -1.0, //
            0.0,
            0.0,
            0.05,
            0.0,
        ]);
        let (s, c) = 0.6f32.sin_cos();
        let view = Mat4([
            c, 0.0, -s, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            s, 0.0, c, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        projection.mul(&view)
    }

    #[test]
    fn a_camera_times_its_inverse_is_the_identity() {
        let m = camera();
        let product = m.mul(&m.inverse().unwrap());
        for (got, want) in product.0.iter().zip(Mat4::IDENTITY.0) {
            assert!((got - want).abs() < 1e-5, "{product:?}");
        }
    }

    #[test]
    fn a_flat_matrix_has_no_inverse() {
        assert_eq!(Mat4([0.0; 16]).inverse(), None);
    }

    fn mul_vec4(m: &Mat4, v: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0; 4];
        for row in 0..4 {
            out[row] = (0..4).map(|k| m.0[k * 4 + row] * v[k]).sum();
        }
        out
    }

    #[test]
    fn single_matrix_reproject_matches_two_step_for_world_and_sky() {
        let curr = camera();
        let inv = curr.inverse().unwrap();
        let (s, c) = 0.03f32.sin_cos();
        let turn = Mat4([
            c, 0.0, -s, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            s, 0.0, c, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ]);
        let prev = curr.mul(&turn);
        let motion = [0.15, -0.05, 0.4];
        let reproject = Mat4::reproject(&prev, motion, &inv);

        // World point: compare single-matrix reproject against two-step unproject + translate + project.
        let ndc = [0.25, -0.4, 0.01, 1.0];
        let p = mul_vec4(&inv, ndc);
        let world = [p[0] / p[3], p[1] / p[3], p[2] / p[3]];
        let two_step = mul_vec4(
            &prev,
            [
                world[0] + motion[0],
                world[1] + motion[1],
                world[2] + motion[2],
                1.0,
            ],
        );
        let one_step = mul_vec4(&reproject, ndc);
        assert!((two_step[0] / two_step[3] - one_step[0] / one_step[3]).abs() < 1e-4);
        assert!((two_step[1] / two_step[3] - one_step[1] / one_step[3]).abs() < 1e-4);

        // Linear depth: 1 / dot(inv_row3, ndc) matches clip.w of the world point.
        let clip = mul_vec4(&curr, [world[0], world[1], world[2], 1.0]);
        let row3 = inv.row(3);
        let inv_w: f32 = (0..4).map(|i| row3[i] * ndc[i]).sum();
        assert!((1.0 / inv_w - clip[3]).abs() < 1e-3);

        // Sky (depth = 0): translation has no effect, matches pure rotation.
        let sky_ndc = [0.25, 0.5, 0.0, 1.0];
        let far = mul_vec4(&inv, [0.25, 0.5, 1e-6, 1.0]);
        let two_step_sky = mul_vec4(
            &prev,
            [far[0] / far[3], far[1] / far[3], far[2] / far[3], 0.0],
        );
        let one_step_sky = mul_vec4(&reproject, sky_ndc);
        assert!(
            (two_step_sky[0] / two_step_sky[3] - one_step_sky[0] / one_step_sky[3]).abs() < 1e-4
        );
        assert!(
            (two_step_sky[1] / two_step_sky[3] - one_step_sky[1] / one_step_sky[3]).abs() < 1e-4
        );
    }
}
