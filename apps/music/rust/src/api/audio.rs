// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Polyphonic SoundFont piano synthesis via `rustysynth`, rendered to the
//! system's default output device with `cpal`.
//!
//! This is the hardware/thread/FFI glue — kept out of the coverage gate like
//! [`super::midi`]. All genuinely testable logic (event model, MIDI mapping,
//! voice bookkeeping) lives in [`super::audio_core`].
//!
//! Threading model: `cpal::Stream` is not `Send` on CoreAudio, so it must be
//! created and dropped on the same thread. [`audio_init`] therefore spawns one
//! dedicated audio thread that owns the stream for the whole process and parks;
//! the FFI entry points only push lock-free [`AudioEvent`]s onto an `mpsc`
//! channel that the audio callback drains each block (no locks/allocation on the
//! hot path).
//!
//! cpal backends: CoreAudio (macOS/iOS), WASAPI (Windows), ALSA (Linux), AAudio
//! (Android — using the NDK context initialized in `JNI_OnLoad`, see lib.rs).

use std::io::Cursor;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use flutter_rust_bridge::frb;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use super::audio_core::{AudioEvent, PIANO_CHANNEL, VoiceTracker};

/// Sender used by the FFI entry points to hand control events to the audio
/// thread. `None` until [`audio_init`] succeeds (so calls before/without a
/// working device are silently dropped — graceful degradation).
static EVENT_TX: Mutex<Option<Sender<AudioEvent>>> = Mutex::new(None);

/// Initializes the synthesizer from SoundFont (`.sf2`) bytes and starts the
/// audio output. Idempotent: a second call keeps the first engine.
///
/// Returns an error if the SoundFont is invalid or no audio device can be
/// opened; the Flutter seam turns that into a silent no-op so the app keeps
/// working without sound.
#[frb(sync)]
pub fn audio_init(sf2_bytes: Vec<u8>) -> Result<()> {
    if EVENT_TX.lock().unwrap().is_some() {
        return Ok(()); // already initialized
    }

    // Parse the SoundFont up front so a bad/missing font fails fast, on the
    // caller's thread, before we touch any audio device.
    let mut cursor = Cursor::new(sf2_bytes);
    let sound_font =
        Arc::new(SoundFont::new(&mut cursor).map_err(|e| anyhow!("invalid SoundFont: {e}"))?);

    let (tx, rx) = mpsc::channel::<AudioEvent>();
    // The audio thread reports device-setup success/failure back so we can
    // surface a real init error synchronously (errors there can't be returned).
    let (ready_tx, ready_rx) = mpsc::channel::<std::result::Result<(), String>>();

    thread::spawn(move || match run_audio_thread(sound_font, rx) {
        Ok(stream) => {
            let _ = ready_tx.send(Ok(()));
            // Keep the stream (and its callback) alive for the process lifetime.
            let _stream = stream;
            loop {
                thread::park();
            }
        }
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
        }
    });

    match ready_rx.recv() {
        Ok(Ok(())) => {
            *EVENT_TX.lock().unwrap() = Some(tx);
            eprintln!("[cymbra-audio] output started");
            Ok(())
        }
        Ok(Err(e)) => Err(anyhow!("audio device unavailable: {e}")),
        Err(e) => Err(anyhow!("audio thread did not start: {e}")),
    }
}

/// Sounds a piano voice for `pitch` at `velocity` (both 7-bit MIDI; 0 velocity
/// is treated as a default mezzo-forte for sources without pressure).
#[frb(sync)]
pub fn note_on(pitch: u8, velocity: u8) {
    send(AudioEvent::note_on(pitch, velocity));
}

/// Releases the voice for `pitch` (it enters the SoundFont's release stage).
#[frb(sync)]
pub fn note_off(pitch: u8) {
    send(AudioEvent::note_off(pitch));
}

/// Releases every sounding voice (stop / restart / seek / loop).
#[frb(sync)]
pub fn all_notes_off() {
    send(AudioEvent::AllOff);
}

/// Pushes an event to the audio thread if the engine is running; otherwise a
/// silent no-op.
fn send(event: AudioEvent) {
    if let Some(tx) = EVENT_TX.lock().unwrap().as_ref() {
        let _ = tx.send(event);
    }
}

/// Opens the default output device and builds the synth stream for its native
/// sample format. Runs on the dedicated audio thread.
fn run_audio_thread(sound_font: Arc<SoundFont>, rx: Receiver<AudioEvent>) -> Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("no default output device"))?;
    let supported = device.default_output_config()?;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.config();

    match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, sound_font, rx),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, sound_font, rx),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, sound_font, rx),
        other => Err(anyhow!("unsupported sample format: {other:?}")),
    }
}

/// Builds and starts a `cpal` output stream whose callback drains control
/// events into the synthesizer and renders the next block of audio.
fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sound_font: Arc<SoundFont>,
    rx: Receiver<AudioEvent>,
) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate as i32;
    let channels = (config.channels as usize).max(1);

    let settings = SynthesizerSettings::new(sample_rate);
    let mut synth =
        Synthesizer::new(&sound_font, &settings).map_err(|e| anyhow!("synth init: {e}"))?;
    let mut tracker = VoiceTracker::new();
    // Reused scratch buffers so the callback never allocates on the steady path.
    let mut left: Vec<f32> = Vec::new();
    let mut right: Vec<f32> = Vec::new();

    let stream = device.build_output_stream(
        *config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // Apply every queued control event in FIFO order.
            while let Ok(ev) = rx.try_recv() {
                match ev {
                    AudioEvent::NoteOn { pitch, velocity } => {
                        tracker.apply(ev);
                        synth.note_on(PIANO_CHANNEL, pitch as i32, velocity as i32);
                    }
                    AudioEvent::NoteOff { .. } => {
                        for pitch in tracker.apply(ev) {
                            synth.note_off(PIANO_CHANNEL, pitch as i32);
                        }
                    }
                    AudioEvent::AllOff => {
                        tracker.apply(ev);
                        synth.note_off_all(true);
                    }
                }
            }

            let frames = output.len() / channels;
            if left.len() < frames {
                left.resize(frames, 0.0);
                right.resize(frames, 0.0);
            }
            let l = &mut left[..frames];
            let r = &mut right[..frames];
            synth.render(l, r);

            // Interleave the stereo render into the device's frame layout. Mono
            // devices get the left channel; >2 channels mirror L/R.
            for (frame, out) in output.chunks_mut(channels).enumerate() {
                for (ch, sample) in out.iter_mut().enumerate() {
                    let v = if ch % 2 == 1 { r[frame] } else { l[frame] };
                    *sample = T::from_sample(v);
                }
            }
        },
        |e| eprintln!("[cymbra-audio] stream error: {e}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}
