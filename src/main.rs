use std::process::{Command};
use coreaudio::sys::*;
use coreaudio::audio_unit::macos_helpers::*;
use signal_hook::consts::{SIGINT, SIGTERM, SIGHUP};
use signal_hook::iterator::Signals;
use std::collections::HashMap;
use std::{ffi::c_void, fs::File};
use std::{mem, thread};
use std::ptr::null_mut;
use serde::{Deserialize, Serialize};
use std::io::BufReader;

struct AudioDevice {
    id: u32,
    name: String,
    has_input: bool,
    has_output: bool
}

#[derive(Serialize, Deserialize)]
struct ProgramOptions {
    pitch: f32,
    input_name: String,
    output_name: String
}

fn set_default_device(input: bool, mut id: u32) {
    println!("Setting default {} device to {}", if input { "input" } else { "output" }, id);
    let property = match input {
        true => kAudioHardwarePropertyDefaultInputDevice,
        false => kAudioHardwarePropertyDefaultOutputDevice
    };
    
    let mut addr = AudioObjectPropertyAddress {
        mSelector: property,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster
    };

    let status;
    unsafe {
        status = AudioObjectSetPropertyData(kAudioObjectSystemObject, &mut addr, 0, null_mut(), mem::size_of::<AudioDeviceID>() as u32, &mut id as *mut _ as *mut c_void);
    }

    if status != 0 {
        panic!("Can't set default device to {}; OSError {}", id, status);
    }
}

fn get_all_audio_devices() -> HashMap<String, AudioDevice> {
    let mut retval = HashMap::new();
    let device_ids = get_audio_device_ids().unwrap();
    for device_id in device_ids {
        let has_output: bool;
        let has_input: bool;
        let name = get_device_name(device_id).unwrap();
        let mut input_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreams,
            mScope: kAudioDevicePropertyScopeInput,
            mElement: 0
        };
        let mut output_addr = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreams,
            mScope: kAudioDevicePropertyScopeOutput,
            mElement: 0
        };
        let mut answer: u32 = 0;
        let mut status;
        unsafe {
            status = AudioObjectGetPropertyDataSize(device_id, &mut input_addr, 0, null_mut(), &mut answer as *const _ as *mut _);
        }
        if status != 0 {
            panic!("Failed to get input streams count on {}", device_id);
        }
        if answer > 0 {
            has_input = true;
        }
        else {
            has_input = false;
        }
        unsafe {
            status = AudioObjectGetPropertyDataSize(device_id, &mut output_addr, 0, null_mut(), &mut answer as *const _ as *mut _);
        }   
        if status != 0 {
            panic!("Failed to get output streams count on {}", device_id);
        }
        if answer > 0 {
            has_output = true;
        }
        else {
            has_output = false;
        }
        retval.insert(name.clone(), AudioDevice { id: device_id, name: name, has_input: has_input, has_output: has_output });
    }
    return retval;
}

fn main() {
    let default_output_id = get_default_device_id(false).expect("Couldn't get default output audio device!");
    let default_input_id = get_default_device_id(true).expect("Couldn't get default input audio device!");
    let devices = get_all_audio_devices();

    let f = File::open("stream_config.json").expect("Config file stream_config.json should exist and be defined like the stream_config.json.example.");
    let opts: ProgramOptions = serde_json::from_reader(BufReader::new(f)).expect("Format of stream_config.json should match stream_config.json.example.");

    if !devices.get(&opts.input_name).expect(&format!("Couldn't find input device '{}'. The parameter is case-sensitive!", &opts.input_name)).has_input {
        panic!("Specified input device '{}' does not actually support audio capture!", opts.input_name);
    }

    if !devices.get(&opts.output_name).expect(&format!("Couldn't find output device '{}'. The parameter is case-sensitive!", &opts.output_name)).has_output {
        panic!("Specified output device '{}' does not actually support audio playback!", opts.output_name);
    }

    let mut input_device_id = None;
    let mut output_device_id = None;

    println!("Devices:");
    for device in devices.values() {
        let default_str = match device.id == default_output_id {
            true => match device.id == default_input_id {
                true => ", DEFAULT OUTPUT AND INPUT",
                false => ", DEFAULT OUTPUT"
            },
            false => match device.id == default_input_id {
                true => ", DEFAULT INPUT",
                false => ""
            }
        };
        println!("{}: {}, input: {}, output: {}{}", device.id, device.name, device.has_input, device.has_output, default_str);
        if device.name == opts.input_name {
            input_device_id = Some(device.id);
        }
        if device.name == opts.output_name {
            output_device_id = Some(device.id);
        }
    }

    if opts.pitch < 0.0 {
        panic!("option 'pitch' provided was {}. It should be greater than 0!", &opts.pitch);
    }

    let exec_str = format!(
        "osxaudiosrc device={} ! audioconvert ! pitch pitch={} ! audioconvert ! queue ! osxaudiosink device={}", 
        input_device_id.expect(&format!("Couldn't find input device {}", &opts.input_name)),
        opts.pitch, 
        output_device_id.expect(&format!("Couldn't find output device {}", &opts.output_name)));


    set_default_device(false, input_device_id.unwrap());

    println!("Running command: {}", exec_str);
    let mut child = Command::new("gst-launch-1.0").args(exec_str.split(" ")).spawn().unwrap();
    let mut signals = Signals::new(&[SIGINT, SIGTERM, SIGHUP]).unwrap();

    let thr = thread::spawn(move || {
        'outer: loop {
            let iter = signals.wait();
            for _sig in iter {
                set_default_device(false, default_output_id);
                //TODO: do this using CoreAudio instead of an osascript that might get deprecated
                let mut inner_child = Command::new("osascript").args(["-e", "set Volume 3"]).spawn().unwrap();
                match inner_child.wait() {
                    Ok(_) => println!("Set volume to 3"),
                    Err(_) => println!("WARN: osascript errored")
                };
                break 'outer;
            }
        }
    });

    match child.wait() {
        Ok(_) => (),
        Err(_) => println!("WARN: gstreamer pipeline errored"),
    };

    thr.join().unwrap();
}
