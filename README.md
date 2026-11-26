# Motion Blur for Sunrise

A [Sunrise](https://github.com/sunrise-org/sunrise) mod that adds smooth, per-pixel motion blur.

- **The camera.** Turning and moving blur the world by how far each pixel travelled, found from
  the world's depth and last frame's camera. The hand, the HUD and menus are meant to stay sharp.
- **The same at any frame rate.** The streak a motion leaves doesn't depend on your FPS.
- **Lightweight.** Off, or with the camera standing still, it records no GPU work at all.

## Known gaps

The mod is written on the typed mod API alone, in GLSL render pipelines and the frame graph. Three
things it needs are not in the API yet, so the picture is not blurred in game:

- sampling the finished frame (vanilla's main colour target) from a mod pass, and writing the
  result back to it, which the last pass (`reconstruct.fsh`) needs;
- the mobs' previous-frame vertex positions, so a mob blurs by its own motion and not only the
  camera's;
- a way to tell where the hand is drawn, so it stays sharp.

The motion and tile passes run; `reconstruct.fsh` is written and registered, and waits for the first.

## Settings

**Options > Mods > Motion Blur > Config**

| Setting | Default | |
|---|---|---|
| Motion Blur | On | Off turns the mod off completely. |
| Strength | 100 | How strong the blur is, from 0 to 200. |

## How it works

The blur follows McGuire et al., *A Reconstruction Filter for Plausible Motion Blur* (2012), with
Jimenez's refinements (2014):

1. **Motion.** Each pixel's position last frame comes from the depth and last frame's camera. Moving
   mobs are drawn again with each vertex's previous position, wherever the world's depth shows them.
2. **Tiles.** The longest motion in each tile, so every pixel knows how far a streak can reach it.
3. **Reconstruction.** Each pixel gathers a few samples along that motion, weighted by depth, so a
   moving thing smears over what is behind it and never over what is in front.

```
src/lib.rs        the mod's hooks
src/pipelines.rs  the GLSL render pipelines, one per pass
src/frame.rs      the camera's motion each frame and the frame graph passes
src/settings.rs   the Options > Mods page
src/core/         timing, camera history and the shader data layout: plain Rust, tested
assets/motion_blur/shaders/   the GLSL programs
```

## Building

You need [`cargo sunrise-mod`](https://sunrise-org.github.io/sunrise/start/install-tool.html).

```sh
cargo test --manifest-path src/core/Cargo.toml   # the logic and the shader data layout
cargo sunrise-mod build                          # target/sunrise-mod/motion_blur.smod
cargo sunrise-mod run --resources <prepared resources> -- --world Test --creative
```

Every push builds the mod on GitHub Actions too, and uploads the `.smod` and a game with the mod in it.

Build with the tool from the SDK pinned in `sunrise-mod.toml`. The package tracks the API it uses, so Sunrise can rebuild it for compatible game builds. Check a build with
`cargo sunrise-mod check target/sunrise-mod/motion_blur.smod --build <build>`.

## License

MIT, see [LICENSE](LICENSE).
