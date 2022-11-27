# macpitch

A lazy way to get nice sound pitch shifting (WSOLA via [libsoundtouch](https://www.surina.net/soundtouch/)) on MacOS for "refreshing" music listening.

Works for just about any audio playback program, like Apple Music, Spotify, Youtube, etc.

The only downside is that it pitch shifts the entire system's audio at once, so if you have multiple apps playing audio and you only want some of them to be pitch shifted, you'd have to use something like [Rogue Amoeba SoundSource](https://rogueamoeba.com/soundsource/).

Assumes you're running a modern version of MacOS with an Intel or Apple Silicon CPU and can follow simple instructions in the terminal.

If you're running Windows/Linux or trying to do this on a mobile device (Chromebook, iOS or Android), this repo is not going to help you, sorry.

# Setup

## Install Loopback

Buy and install https://rogueamoeba.com/loopback/

Note: There may be other programs that do the same thing. We just need one CoreAudio loopback device. It doesn't matter how you get it.

On Apple Silicon, you have to reboot into the recovery environment to install Rogue Amoeba's ACE - they provide instructions when you install the program. Follow them.

If you can't satisfy this step to the point that you see a loopback audio device in your Mac sound options, you may as well stop here, as the rest of the instructions can't work without this.

## Install Xcode

Get it from the App Store.

## Install Xcode command line tools

Try `sudo xcode-select install`. If this fails, get the latest `Command Line Tools for Xcode` from https://developer.apple.com/download/all/?q=xcode and install using the GUI.

## Install Homebrew

Follow instructions on the homepage @ https://brew.sh/ 

The rest of the instructions are done from the Terminal.

## Install Open Source Dependencies

`brew install rust git gstreamer gst-plugins-base gst-plugins-good sound-touch gst-devtools`

## Fix gst-plugins-bad build

`EDITOR=nano brew edit gst-plugins-bad`

Change the config file: find where the Meson `args` is defined, and add a new item of `-Dsoundtouch=enabled`. For example, if the default says

```
args = %w[
      -Dgpl=enabled
      -Dintrospection=enabled
      -Dexamples=disabled
    ]
```

once you edit it, it should say:

```
args = %w[
      -Dgpl=enabled
      -Dintrospection=enabled
      -Dexamples=disabled
      -Dsoundtouch=enabled
    ]
```

Save the "file" and exit your editor (e.g. Ctrl+X and follow prompts in nano.)

## Compile gst-plugins-bad

`brew install gst-plugins-bad --build-from-source`

## Clone this repo

`git clone https://github.com/allquixotic/macpitch`

## Build the program

`cargo build --release`

## Create the config file

Create the file `stream_config.json` in the base directory of this repository, e.g. using `nano stream_config.json`

Example:

```
{
    "pitch": 0.99,
    "input_name": "Loopback Audio",
    "output_name": "MacBook Pro Speakers"
}
```

 - `pitch`: The desired multiplier of the pitch. See [pitch element documentation](https://gstreamer.freedesktop.org/documentation/soundtouch/pitch.html).
 - `input_name`: The name of your _audio loopback device_ (e.g. Rogue Amoeba Loopback). The default for the Loopback App is "Loopback Audio", so if you let it create the default loopback device, enter the name "Loopback Audio" here. You can find this name in the sound preferences pane of MacOS.
 - `output_name`: The name of your _physical audio device_ (your speakers or headset) as it shows in the MacOS sound preferences pane. This is where the audio will be played back.

## Run this program

`./run.sh` or `./target/release/macpitch`

This will:

 - Set your system default audio output device to the provided loopback device. Most programs should pick this up without restarting.
 - Use gstreamer to pitch shift the audio on the loopback device and play it back to your physical audio device.

Leave the terminal open, or you can set this up as a background task, e.g. with `screen` or `tmux` as you prefer.

If the Python script receives a SIGINT, SIGTERM or SIGHUP signal, it will gracefully set your default audio device back to what it was before the program started. In any other case, like if the Python script is forcibly killed with a SIGKILL (e.g. `kill -9`), or if your Mac loses power suddenly, the default audio device will remain on the loopback. You'll need to change it back to a physical sound device to hear sound again.

Normally, it will receive SIGINT if you press Ctrl+C on the terminal. It will receive SIGTERM if you use the `kill` command with the default signal. It will receive SIGHUP if you are connected to your Mac over SSH and terminate the SSH session.

# Caveats

 - Doesn't work with AirPlay devices out of the box, as they don't seem to work with CoreAudio. Maybe you can set up a second loopback device as the output, and use one of Rogue Amoeba's programs (SoundSource or Loopback) to stream from the input of the second loopback to an AirPlay device?

 - Requires a fair bit of setup. Sorry, this is the easiest way I found to do this.

 - Volume may blast at 100% when it switches back to your non-loopback device. I have a workaround in the script that _should_ prevent this, but you have been warned.

 - Similarly, volume may be very low when switching to the loopback device. Just turn it up until you like the volume. Recommend playing to loopback at 100% and setting the volume of your physical device to a lower value.