//! The mod's page in Options > Mods, and the copy of it the render thread reads each frame.

use mod_api::{Host, Registrar, SettingKey, SettingSpec};
use motion_blur_core::Settings;
use std::sync::{OnceLock, PoisonError, RwLock};

static CURRENT: RwLock<Settings> = RwLock::new(Settings::DEFAULT);

const ENABLED: &str = "enabled";
const STRENGTH: &str = "strength";

/// The handles of the registered settings: the enabled switch and the strength.
static KEYS: OnceLock<(SettingKey, SettingKey)> = OnceLock::new();

pub fn register(registrar: &mut dyn Registrar) {
    let specs = [
        SettingSpec::toggle(
            ENABLED,
            "motion_blur.setting.enabled",
            Settings::DEFAULT.enabled,
        ),
        SettingSpec::range(
            STRENGTH,
            "motion_blur.setting.strength",
            0.0,
            200.0,
            5.0,
            100.0,
        )
        .tooltip("motion_blur.setting.strength.tooltip"),
    ];
    let mut keys = Vec::new();
    for spec in specs {
        let name = spec.key.clone();
        match registrar.register_setting(spec) {
            Ok(key) => keys.push(key),
            Err(error) => eprintln!("motion_blur: the {name} setting was refused: {error:?}"),
        }
    }
    if let [enabled, strength] = keys[..] {
        let _ = KEYS.set((enabled, strength));
    }
}

/// Reads the player's choices into the copy the renderer uses.
pub fn load(host: &dyn Host) {
    let Some(&(enabled, strength)) = KEYS.get() else {
        return;
    };
    let defaults = Settings::DEFAULT;
    let settings = Settings {
        enabled: host.setting_bool(enabled).unwrap_or(defaults.enabled),
        strength: host
            .setting_f64(strength)
            .map_or(defaults.strength, |percent| (percent / 100.0) as f32),
    };
    *CURRENT.write().unwrap_or_else(PoisonError::into_inner) = settings;
}

pub fn current() -> Settings {
    *CURRENT.read().unwrap_or_else(PoisonError::into_inner)
}
