use coreaudio_sys::{
    kAudioDevicePropertyDataSource, kAudioDevicePropertyTransportType,
    kAudioDeviceTransportTypeBuiltIn, kAudioHardwarePropertyDevices,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, AudioObjectGetPropertyData,
    AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectPropertyAddress,
};
use std::collections::HashSet;
use std::mem;
use std::thread;
use std::time::{Duration, Instant};

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

unsafe fn get_transport_type(device_id: AudioObjectID) -> u32 {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyTransportType,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: 0,
    };
    let mut transport_type: u32 = 0;
    let mut data_size = mem::size_of::<u32>() as u32;
    AudioObjectGetPropertyData(
        device_id,
        &address,
        0,
        std::ptr::null(),
        &mut data_size,
        &mut transport_type as *mut _ as *mut std::ffi::c_void,
    );
    transport_type
}

unsafe fn get_data_source(device_id: AudioObjectID) -> u32 {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDataSource,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: 0,
    };
    let mut source_id: u32 = 0;
    let mut data_size = mem::size_of::<u32>() as u32;
    AudioObjectGetPropertyData(
        device_id,
        &address,
        0,
        std::ptr::null(),
        &mut data_size,
        &mut source_id as *mut _ as *mut std::ffi::c_void,
    );
    source_id
}

fn fourcc(value: u32) -> String {
    let bytes = value.to_be_bytes();
    format!(
        "{}{}{}{}",
        bytes[0] as char, bytes[1] as char, bytes[2] as char, bytes[3] as char
    )
}

fn print_device_list(label: &str) {
    let devices = unsafe { get_all_device_ids() };
    let hdpn = u32::from_be_bytes(*b"hdpn");
    let ispk = u32::from_be_bytes(*b"ispk");

    println!("[{label}] {} devices:", devices.len());
    for &dev_id in &devices {
        let transport = unsafe { get_transport_type(dev_id) };
        let is_builtin = transport == kAudioDeviceTransportTypeBuiltIn;
        let ds = unsafe { get_data_source(dev_id) };
        let ds_label = match ds {
            x if x == hdpn => "HEADPHONES",
            x if x == ispk => "SPEAKERS",
            0 => "none",
            _ => "other",
        };
        let tag = if is_builtin { "builtin" } else { "other" };
        println!(
            "  ID={dev_id} [{tag}] ds={:#010x} ({}) [{ds_label}]",
            ds,
            fourcc(ds)
        );
    }
}

fn main() {
    println!("=== CoreAudio Device Monitor Test v4 ===");
    println!("Approach: monitor device list changes (add/remove)");
    println!();

    let hdpn = u32::from_be_bytes(*b"hdpn");
    let ispk = u32::from_be_bytes(*b"ispk");
    println!("FourCC: ISPK={:#010x} HDPN={:#010x}", ispk, hdpn);
    println!();

    print_device_list("initial");
    println!();

    let initial_devices: HashSet<AudioObjectID> =
        unsafe { get_all_device_ids() }.into_iter().collect();
    let mut known_devices = initial_devices;

    println!("Polling every 200ms... Plug/unplug headphones to test.");
    println!("Press Ctrl+C to stop.");
    println!();

    let start = Instant::now();
    let mut tick = 0u64;

    loop {
        thread::sleep(Duration::from_millis(200));
        tick += 1;

        let current_devices: HashSet<AudioObjectID> =
            unsafe { get_all_device_ids() }.into_iter().collect();

        let added: Vec<AudioObjectID> =
            current_devices.difference(&known_devices).copied().collect();
        let removed: Vec<AudioObjectID> =
            known_devices.difference(&current_devices).copied().collect();

        if !added.is_empty() || !removed.is_empty() {
            let elapsed = start.elapsed().as_millis();
            println!(
                "[{:6}ms] tick={tick} DEVICE LIST CHANGED: +{} -{}",
                elapsed,
                added.len(),
                removed.len()
            );

            for &id in &added {
                let transport = unsafe { get_transport_type(id) };
                let ds = unsafe { get_data_source(id) };
                let ds_label = match ds {
                    x if x == hdpn => "HEADPHONES",
                    x if x == ispk => "SPEAKERS",
                    0 => "none",
                    _ => "other",
                };
                let tag = if transport == kAudioDeviceTransportTypeBuiltIn {
                    "builtin"
                } else {
                    "other"
                };
                println!(
                    "  + ID={id} [{tag}] ds={:#010x} ({}) [{ds_label}]",
                    ds,
                    fourcc(ds)
                );
            }
            for &id in &removed {
                println!("  - ID={id}");
            }

            let has_headphone = unsafe {
                current_devices.iter().any(|&id| {
                    get_transport_type(id) == kAudioDeviceTransportTypeBuiltIn
                        && get_data_source(id) == hdpn
                })
            };
            println!("  Headphone present: {has_headphone}");

            known_devices = current_devices;
        } else if tick.is_multiple_of(25) {
            let elapsed = start.elapsed().as_millis();
            println!(
                "[{:6}ms] tick={tick} no change ({} devices)",
                elapsed,
                current_devices.len()
            );
        }
    }
}
