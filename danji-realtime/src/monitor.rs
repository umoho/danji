//! 输出设备监听模块。
//!
//! 通过 CoreAudio 设备列表变化检测耳机插拔，通知 engine 切换输出设备。
//!
//! ---
//!
//! Output device monitor module.
//!
//! Detects headphone plug/unplug via CoreAudio device list changes,
//! and notifies the engine to switch output devices.

#[cfg(target_os = "macos")]
pub fn run_device_monitor(
    host: cpal::Host,
    cmd_tx: std::sync::mpsc::Sender<crate::params::MainCommand>,
) {
    use coreaudio_sys::{
        kAudioHardwarePropertyDevices, kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
        AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID,
        AudioObjectPropertyAddress,
    };
    use cpal::traits::{DeviceTrait, HostTrait};
    use std::collections::HashSet;
    use std::mem;
    use std::thread;
    use std::time::Duration;

    use crate::params::MainCommand;

    /// 获取所有音频设备 ID 列表。
    ///
    /// Get all audio device IDs from the system.
    unsafe fn get_all_device_ids() -> Vec<AudioObjectID> {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: 0,
        };
        let mut data_size: u32 = 0;
        AudioObjectGetPropertyDataSize(
            kAudioObjectSystemObject,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
        );
        let count = data_size as usize / mem::size_of::<AudioObjectID>();
        let mut devices = vec![0u32; count];
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
            devices.as_mut_ptr() as *mut std::ffi::c_void,
        );
        devices
    }

    // 获取初始设备集合
    let initial_devices: HashSet<AudioObjectID> =
        unsafe { get_all_device_ids() }.into_iter().collect();

    log::info!(
        "Output device monitor started ({} initial devices)",
        initial_devices.len()
    );

    let mut known_devices = initial_devices;

    loop {
        thread::sleep(Duration::from_millis(200));

        let current_devices: HashSet<AudioObjectID> =
            unsafe { get_all_device_ids() }.into_iter().collect();

        let added: Vec<AudioObjectID> = current_devices
            .difference(&known_devices)
            .copied()
            .collect();
        let removed: Vec<AudioObjectID> = known_devices
            .difference(&current_devices)
            .copied()
            .collect();

        if !added.is_empty() || !removed.is_empty() {
            log::info!("Device list changed: +{} -{}", added.len(), removed.len());

            // 设备增加 → 耳机插入 → 找外置设备
            // 设备减少 → 耳机拔出 → 切回内置扬声器
            let target_device = if !added.is_empty() {
                log::info!("Device added, switching to external output");
                host.devices().ok().and_then(|devices| {
                    devices.into_iter().find(|d| {
                        d.default_output_config().is_ok()
                            && d.description()
                                .map(|desc| {
                                    let name = desc.name().to_string();
                                    !name.contains("BlackHole")
                                        && !name.contains("多输出")
                                        && !name.contains("Aggregate")
                                        && !name.contains("MacBook")
                                        && !name.contains("扬声器")
                                        && !name.contains("Speakers")
                                })
                                .unwrap_or(false)
                    })
                })
            } else {
                log::info!("Device removed, switching to built-in speakers");
                host.devices().ok().and_then(|devices| {
                    devices.into_iter().find(|d| {
                        d.default_output_config().is_ok()
                            && d.description()
                                .map(|desc| {
                                    let name = desc.name().to_string();
                                    !name.contains("BlackHole")
                                        && !name.contains("多输出")
                                        && !name.contains("Aggregate")
                                        && (name.contains("MacBook")
                                            || name.contains("扬声器")
                                            || name.contains("Speakers"))
                                })
                                .unwrap_or(false)
                    })
                })
            };

            if let Some(device) = target_device {
                let name = device
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_default();
                log::info!("Switching output to: {name}");
                let _ = cmd_tx.send(MainCommand::SwitchOutput { device });
            }

            known_devices = current_devices;
        }
    }
}

/// 非 macOS 平台的空实现。
///
/// No-op implementation for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub fn run_device_monitor(
    _host: cpal::Host,
    _cmd_tx: std::sync::mpsc::Sender<crate::params::MainCommand>,
) {
}
