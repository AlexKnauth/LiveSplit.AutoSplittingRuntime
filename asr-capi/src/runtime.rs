use crate::{settings_map::SettingsMap, user_settings::UserSettings};

#[cfg(target_pointer_width = "64")]
use crate::{log, setting_value::SettingValue, str, CTimer};
#[cfg(target_pointer_width = "64")]
use livesplit_auto_splitting::{Config, ConfigSettings};
#[cfg(target_pointer_width = "64")]
use std::{fs, sync::Arc};

#[cfg(target_pointer_width = "64")]
pub struct Runtime {
    runtime: livesplit_auto_splitting::Runtime<CTimer>,
}

#[cfg(not(target_pointer_width = "64"))]
pub type Runtime = ();

/// # Safety
/// TODO:
#[no_mangle]
pub unsafe extern "C" fn Runtime_new(
    _path_ptr: *const u8,
    _settings_map: Option<Box<SettingsMap>>,
    _legacy_xml: *const u8,
    _state: unsafe extern "C" fn() -> i32,
    _start: unsafe extern "C" fn(),
    _split: unsafe extern "C" fn(),
    _skip_split: unsafe extern "C" fn(),
    _undo_split: unsafe extern "C" fn(),
    _reset: unsafe extern "C" fn(),
    _set_game_time: unsafe extern "C" fn(i64),
    _pause_game_time: unsafe extern "C" fn(),
    _resume_game_time: unsafe extern "C" fn(),
    _log: unsafe extern "C" fn(*const u8, usize),
) -> Option<Box<Runtime>> {
    #[cfg(target_pointer_width = "64")]
    {
        let path = str(_path_ptr);
        let file = match fs::read(path) {
            Ok(file) => file,
            Err(err) => {
                log(
                    _log,
                    format_args!(
                        "{:?}",
                        anyhow::Error::from(err)
                            .context("Failed reading the file for the auto splitter."),
                    ),
                );
                return None;
            }
        };

        let mut config = Config::default();
        config.settings = config_settings(_settings_map, _legacy_xml);

        match livesplit_auto_splitting::Runtime::new(
            &file,
            CTimer {
                state: _state,
                start: _start,
                split: _split,
                skip_split: _skip_split,
                undo_split: _undo_split,
                reset: _reset,
                set_game_time: _set_game_time,
                pause_game_time: _pause_game_time,
                resume_game_time: _resume_game_time,
                log: _log,
            },
            config,
        ) {
            Ok(runtime) => Some(Box::new(Runtime { runtime })),
            Err(err) => {
                log(
                    _log,
                    format_args!(
                        "{:?}",
                        anyhow::Error::from(err).context("Failed loading the auto splitter."),
                    ),
                );
                None
            }
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    Some(Box::new(()))
}

#[no_mangle]
pub extern "C" fn Runtime_drop(_: Box<Runtime>) {}

#[no_mangle]
pub extern "C" fn Runtime_step(_this: &Runtime) -> bool {
    #[cfg(target_pointer_width = "64")]
    {
        _this.runtime.lock().update().is_ok()
    }
    #[cfg(not(target_pointer_width = "64"))]
    true
}

#[no_mangle]
pub extern "C" fn Runtime_tick_rate(_this: &Runtime) -> u64 {
    const TICKS_PER_SEC: u64 = 10_000_000;
    const NANOS_PER_SEC: u64 = 1_000_000_000;
    const NANOS_PER_TICK: u64 = NANOS_PER_SEC / TICKS_PER_SEC;

    #[cfg(target_pointer_width = "64")]
    let tick_rate = _this.runtime.tick_rate();
    #[cfg(not(target_pointer_width = "64"))]
    let tick_rate = std::time::Duration::new(1, 0) / 120;

    let (secs, nanos) = (tick_rate.as_secs(), tick_rate.subsec_nanos());

    secs * TICKS_PER_SEC + nanos as u64 / NANOS_PER_TICK
}

#[no_mangle]
pub extern "C" fn Runtime_get_user_settings(_this: &Runtime) -> Box<UserSettings> {
    #[cfg(target_pointer_width = "64")]
    {
        Box::new(UserSettings {
            inner: _this.runtime.settings_widgets(),
        })
    }
    #[cfg(not(target_pointer_width = "64"))]
    Box::new(())
}

/// # Safety
/// TODO:
#[no_mangle]
pub unsafe extern "C" fn Runtime_settings_map_set_bool(
    _this: &Runtime,
    _key: *const u8,
    _value: bool,
) {
    #[cfg(target_pointer_width = "64")]
    {
        let key = Arc::<str>::from(str(_key));
        loop {
            let mut map = _this.runtime.settings_map();
            let old = map.clone();
            map.insert(key.clone(), SettingValue::Bool(_value));
            if _this.runtime.set_settings_map_if_unchanged(&old, map) {
                break;
            }
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    panic!("Index out of bounds")
}

/// # Safety
/// TODO:
#[no_mangle]
pub unsafe extern "C" fn Runtime_settings_map_set_string(
    _this: &Runtime,
    _key: *const u8,
    _value: *const u8,
) {
    #[cfg(target_pointer_width = "64")]
    {
        let key = Arc::<str>::from(str(_key));
        let value = Arc::<str>::from(str(_value));
        loop {
            let mut map = _this.runtime.settings_map();
            let old = map.clone();
            map.insert(key.clone(), SettingValue::String(value.clone()));
            if _this.runtime.set_settings_map_if_unchanged(&old, map) {
                break;
            }
        }
    }
    #[cfg(not(target_pointer_width = "64"))]
    panic!("Index out of bounds")
}

#[no_mangle]
pub extern "C" fn Runtime_get_settings_map(_this: &Runtime) -> Box<SettingsMap> {
    #[cfg(target_pointer_width = "64")]
    {
        Box::new(_this.runtime.settings_map())
    }
    #[cfg(not(target_pointer_width = "64"))]
    Box::new(())
}

#[no_mangle]
pub extern "C" fn Runtime_set_settings_map(_this: &Runtime, _settings_map: Box<SettingsMap>) {
    #[cfg(target_pointer_width = "64")]
    {
        _this.runtime.set_settings_map(*_settings_map);
    }
}

#[no_mangle]
pub extern "C" fn Runtime_are_settings_changed(
    _this: &Runtime,
    _previous_settings_map: &SettingsMap,
    _previous_user_settings: &UserSettings,
) -> bool {
    #[cfg(target_pointer_width = "64")]
    {
        !_this
            .runtime
            .settings_map()
            .is_unchanged(_previous_settings_map)
            || !Arc::ptr_eq(
                &_this.runtime.settings_widgets(),
                &_previous_user_settings.inner,
            )
    }
    #[cfg(not(target_pointer_width = "64"))]
    false
}

#[cfg(target_pointer_width = "64")]
fn config_settings(_settings_map: Option<Box<SettingsMap>>, _legacy_xml: *const u8) -> ConfigSettings {
    if let Some(settings_map) = _settings_map {
        let m = *settings_map;
        if !m.is_empty() {
            return ConfigSettings::Map(m);
        }
    }
    let legacy_xml = unsafe { str(_legacy_xml) }.trim();
    if !legacy_xml.is_empty() {
        return ConfigSettings::LegacyXML(legacy_xml.to_string());
    }
    ConfigSettings::None
}
