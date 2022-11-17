import soundcard as sc
import signal, sys, os, json

expected_options = ["input_name", "output_name", "pitch"]
missing_options = []
config_file_name = "stream_config.json"
outputs = { x.name : x.id for x in sc.all_speakers() }
inputs = { x.name : x.id for x in sc.all_microphones() }
input_device_id = None
output_device_id = None
prev_device = sc.default_speaker().id #Current default output device

print("outputs:")
print(json.dumps(outputs, indent=4))
print("inputs:")
print(json.dumps(inputs, indent=4))

def set_default_output(id):
    os.system(f"SwitchAudioSource -t output -i {id}")

def reset_default_device():
    if prev_device is not None:
        print(f"Setting default device back to {prev_device}")
        set_default_output(prev_device)

with open('stream_config.json') as f:
    options = json.load(f)
    for option in expected_options:
        if option not in options:
            missing_options.append(option)
    if len(missing_options) > 0:
        print(f"ERROR: Missing options in {config_file_name}: {json.dumps(missing_options, indent=4)}")
        exit(1)
    input_name = options['input_name']
    output_name = options['output_name']
    pitch = options['pitch']
    if input_name != "default":
        if input_name not in inputs:
            print(f"ERROR: Can't find input device {input_name}")
            exit(1)
        else:
            input_device_id = inputs[input_name]
    if output_name != "default":
        if output_name not in outputs:
            print(f"ERROR: Can't find output device {output_name}")
            exit(1)
        else:
            output_device_id = outputs[output_name]
    if not pitch.replace('.', '', 1).isdigit():
        print(f"ERROR: pitch {pitch} is not a number!")
        exit(1)

    def signal_handler(sig, frame):
        reset_default_device()
        os.system("osascript -e 'set Volume 2'")
        exit(0)

    for i in [signal.SIGINT, signal.SIGHUP, signal.SIGTERM]:
        signal.signal(i, signal_handler)

    set_default_output(outputs[input_name])
    
    cmd = f"gst-launch-1.0 osxaudiosrc device={input_device_id} ! audioconvert ! pitch pitch={pitch} ! audioconvert ! queue ! osxaudiosink device={output_device_id}"
    os.system(cmd)
    reset_default_device()

    #Yikes! Loud volume!
    os.system("osascript -e 'set Volume 2'")