//! The blur's GLSL declares the `Params` block `BlurParams` lays out, and every program is whole.

use motion_blur_core::BlurParams;

const PARAMS: &str = include_str!("../../../assets/motion_blur/shaders/include/params.glsl");
const PROGRAMS: [(&str, &str); 6] = [
    (
        "fullscreen.vsh",
        include_str!("../../../assets/motion_blur/shaders/core/fullscreen.vsh"),
    ),
    (
        "motion.fsh",
        include_str!("../../../assets/motion_blur/shaders/core/motion.fsh"),
    ),
    (
        "tile_rows.fsh",
        include_str!("../../../assets/motion_blur/shaders/core/tile_rows.fsh"),
    ),
    (
        "tile_columns.fsh",
        include_str!("../../../assets/motion_blur/shaders/core/tile_columns.fsh"),
    ),
    (
        "neighbor_max.fsh",
        include_str!("../../../assets/motion_blur/shaders/core/neighbor_max.fsh"),
    ),
    (
        "reconstruct.fsh",
        include_str!("../../../assets/motion_blur/shaders/core/reconstruct.fsh"),
    ),
];

/// The std140 size of a GLSL type, in bytes, with its alignment.
fn std140(ty: &str) -> (usize, usize) {
    match ty {
        "mat4" => (64, 16),
        "vec4" => (16, 16),
        "vec2" | "uvec2" => (8, 8),
        "float" | "uint" | "int" => (4, 4),
        other => panic!("no std140 size for {other}"),
    }
}

#[test]
fn params_match_blur_params() {
    let block = PARAMS
        .split("uniform Params {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("a Params block");
    let mut offset = 0;
    for member in block
        .lines()
        .filter_map(|line| line.trim().strip_suffix(';'))
    {
        let ty = member.split_whitespace().next().expect("a type");
        let (size, align) = std140(ty);
        offset = offset.next_multiple_of(align) + size;
    }
    assert_eq!(offset, std::mem::size_of::<BlurParams>());
}

#[test]
fn every_program_has_a_version_and_a_main() {
    for (name, source) in PROGRAMS {
        assert!(source.starts_with("#version 330\n"), "{name}");
        assert!(source.contains("void main()"), "{name}");
    }
}

#[test]
fn fragment_programs_include_the_params_and_write_one_colour() {
    for (name, source) in PROGRAMS.iter().filter(|(name, _)| name.ends_with(".fsh")) {
        assert!(
            source.contains("#include <motion_blur:params.glsl>"),
            "{name}"
        );
        assert!(source.contains("out vec4 fragColor;"), "{name}");
    }
}
