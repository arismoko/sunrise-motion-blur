#![deny(unsafe_code)]
//! Motion blur for Sunrise: every pixel is smeared along the way the camera moved it since the
//! last frame.
//!
//! The mod is written wholly on the typed mod API, with no native rows:
//!
//! - `pipelines` registers the GLSL render pipelines (`assets/motion_blur/shaders/`);
//! - `frame` reads the camera each frame and records the blur's passes in the frame graph;
//! - what doesn't need the game is in `motion-blur-core` (`src/core/`), where it is tested.

mod frame;
mod pipelines;
mod settings;

use mod_api::{
    ClientHost, ClientLevelLoadEvent, ClientLevelUnloadEvent, ClientReloadHost,
    ClientResourcesReloadedEvent, ClientSetupContext, ClientSetupEvent, FrameGraphSetupEvent,
    Hooks, Host, RegisterRenderPipelinesEvent, Registrar, RegistrationContext, SettingChangedEvent,
};

#[derive(Default)]
struct MotionBlur;

#[mod_api::sunrise_mod]
impl Hooks for MotionBlur {
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        settings::register(registrar);
    }

    fn register_render_pipelines(
        &self,
        _: &RegistrationContext,
        event: &mut RegisterRenderPipelinesEvent<'_>,
    ) {
        pipelines::register(event);
    }

    fn client_setup(&self, context: &mut dyn ClientSetupContext, _: &mut ClientSetupEvent) {
        pipelines::resolve(context.render_assets());
    }

    /// A reload builds every pipeline again, and the old handles go stale.
    fn client_resources_reloaded(
        &self,
        context: &dyn ClientReloadHost,
        _: &mut ClientResourcesReloadedEvent,
    ) {
        pipelines::resolve(context.render_assets());
    }

    /// Settings can be read once a world is open: read them each time one opens.
    fn client_level_load(&self, host: &mut dyn ClientHost, _: &mut ClientLevelLoadEvent<'_>) {
        settings::load(&*host);
        frame::reset();
    }

    fn client_level_unload(&self, _: &mut dyn ClientHost, _: &mut ClientLevelUnloadEvent<'_>) {
        frame::reset();
    }

    fn setting_changed(&self, host: &dyn Host, _: &mut SettingChangedEvent<'_>) {
        settings::load(host);
    }

    fn frame_graph_setup(&self, host: &dyn ClientHost, event: &mut FrameGraphSetupEvent<'_>) {
        frame::setup(host, event);
    }
}
