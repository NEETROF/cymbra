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

use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use flutter_rust_bridge::frb;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use super::audio_core::{AudioEvent, ClickVoice, PIANO_CHANNEL, VoiceTracker, is_valid_soundfont};

/// A message handed from the UI/FFI thread to the audio thread over the lock-free
/// queue. Most are plain control [`AudioEvent`]s (tiny, `Copy`); a runtime
/// SoundFont swap additionally carries an already-parsed instrument.
///
/// Routing the swap through the *same* queue keeps it in FIFO order with the
/// notes around it: notes queued before the swap sound on the old instrument,
/// notes after it on the new one, and the swap itself applies an all-notes-off so
/// nothing hangs across it.
enum AudioCommand {
    /// A note/all-off/click control event (see [`AudioEvent`]).
    Control(AudioEvent),
    /// Replace the active synthesizer with one built from this SoundFont,
    /// silencing every sounding voice across the swap. The heavy parse already
    /// happened off the audio thread (see [`audio_load_soundfont`]); the callback
    /// only builds the synth and swaps it in.
    ReplaceSynth(Arc<SoundFont>),
}

/// Sender used by the FFI entry points to hand commands to the audio thread.
/// Published as soon as [`audio_init`] starts (so note events queue while the
/// device spins up) and cleared if setup fails, so calls without a working device
/// are silently dropped — graceful degradation.
static EVENT_TX: Mutex<Option<Sender<AudioCommand>>> = Mutex::new(None);

/// Guards against launching more than one audio engine. Reset on setup failure
/// so a later call can retry.
static INIT_STARTED: AtomicBool = AtomicBool::new(false);

/// Initializes the synthesizer from a SoundFont (`.sf2`) file path and starts
/// the audio output. Idempotent: a second call keeps the first engine.
///
/// Returns immediately — the heavy work (reading + parsing the ~50 MB SoundFont
/// and opening the device) runs on the dedicated audio thread, reading straight
/// from disk so the UI isolate never blocks and no large buffer crosses the
/// bridge. If the font is missing/invalid or no device can be opened the engine
/// stays silent (note events become no-ops); the app keeps working.
#[frb(sync)]
pub fn audio_init(sf2_path: String) {
    if INIT_STARTED.swap(true, Ordering::SeqCst) {
        return; // already initialized (or initializing)
    }

    let (tx, rx) = mpsc::channel::<AudioCommand>();
    // Publish the sender now so notes pressed during startup queue up; they are
    // drained once the stream's callback begins.
    *EVENT_TX.lock().unwrap() = Some(tx);

    thread::spawn(move || match run_audio_thread(sf2_path, rx) {
        Ok(stream) => {
            eprintln!("[cymbra-audio] output started");
            // Keep the stream (and its callback) alive for the process lifetime.
            let _stream = stream;
            loop {
                thread::park();
            }
        }
        Err(e) => {
            eprintln!("[cymbra-audio] disabled: {e}");
            // Drop the sender so further note events are silent no-ops, and let
            // a future call retry.
            *EVENT_TX.lock().unwrap() = None;
            INIT_STARTED.store(false, Ordering::SeqCst);
        }
    });
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

/// Swaps the synthesizer's active SoundFont at runtime from a `.sf2` file path,
/// so a newly chosen piano sounds for every later note **without** tearing down
/// or re-acquiring the audio output stream.
///
/// A silent no-op if the engine is not running. Returns immediately: the heavy
/// read/parse of the ~27–296 MB SoundFont runs on a short-lived worker thread
/// (never the UI isolate, never the real-time audio callback), reading straight
/// from disk exactly like [`audio_init`] — no large buffer crosses the bridge.
/// Once parsed, the instrument is handed to the audio thread, which applies an
/// all-notes-off and installs it. If the file is missing/invalid the current
/// piano is kept (graceful fallback); the queued note stream is undisturbed.
#[frb(sync)]
pub fn audio_load_soundfont(sf2_path: String) {
    // Snapshot the sender; if the engine never started (or failed), do nothing.
    let tx = match EVENT_TX.lock().unwrap().as_ref() {
        Some(tx) => tx.clone(),
        None => return,
    };
    thread::spawn(move || match load_sound_font(&sf2_path) {
        Ok(sound_font) => {
            let _ = tx.send(AudioCommand::ReplaceSynth(sound_font));
        }
        Err(e) => eprintln!("[cymbra-audio] soundfont swap skipped, keeping current: {e}"),
    });
}

/// Sounds a short metronome click — a synthesized tick mixed into the output
/// independently of the piano SoundFont. `accent` marks the downbeat (higher and
/// louder). Self-terminating: there is no matching off.
#[frb(sync)]
pub fn metronome_click(accent: bool) {
    send(AudioEvent::Click { accent });
}

/// Pushes a control event to the audio thread if the engine is running;
/// otherwise a silent no-op.
fn send(event: AudioEvent) {
    if let Some(tx) = EVENT_TX.lock().unwrap().as_ref() {
        let _ = tx.send(AudioCommand::Control(event));
    }
}

/// Reads and parses a SoundFont (`.sf2`) from `path`, cheaply rejecting a file
/// whose RIFF/`sfbk` header is wrong before the full parse. Shared by
/// [`audio_init`] (initial load) and [`audio_load_soundfont`] (runtime swap);
/// streams from disk via a `BufReader` so no multi-MB buffer is held in memory.
fn load_sound_font(path: &str) -> Result<Arc<SoundFont>> {
    // Fast pre-check: read just the 12-byte preamble and reject non-SoundFonts
    // (e.g. a WAV, a truncated file) without parsing the whole bank.
    let mut header = [0u8; 12];
    let read = File::open(path)
        .and_then(|mut f| f.read(&mut header))
        .map_err(|e| anyhow!("open SoundFont {path}: {e}"))?;
    if !is_valid_soundfont(&header[..read]) {
        return Err(anyhow!("not a SoundFont (bad RIFF/sfbk header): {path}"));
    }
    let file = File::open(path).map_err(|e| anyhow!("open SoundFont {path}: {e}"))?;
    let mut reader = BufReader::new(file);
    let sound_font = SoundFont::new(&mut reader).map_err(|e| anyhow!("invalid SoundFont: {e}"))?;
    Ok(Arc::new(sound_font))
}

/// Reads and parses the SoundFont from `sf2_path`, opens the default output
/// device and builds the synth stream for its native sample format. Runs on the
/// dedicated audio thread, so the multi-second SoundFont read/parse never blocks
/// the UI.
fn run_audio_thread(sf2_path: String, rx: Receiver<AudioCommand>) -> Result<cpal::Stream> {
    let sound_font = load_sound_font(&sf2_path)?;

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
    rx: Receiver<AudioCommand>,
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
    // The currently sounding metronome click, if any. A click is a short one-shot
    // mixed in on top of the synth; a new click event simply replaces it.
    let mut click: Option<ClickVoice> = None;
    // Reused scratch buffers so the callback never allocates on the steady path.
    let mut left: Vec<f32> = Vec::new();
    let mut right: Vec<f32> = Vec::new();

    let stream = device.build_output_stream(
        *config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // Apply every queued command in FIFO order.
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    AudioCommand::Control(ev) => match ev {
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
                        AudioEvent::Click { accent } => {
                            click = Some(ClickVoice::new(accent, sample_rate as f32));
                        }
                    },
                    // Runtime SoundFont swap: silence every voice across the swap
                    // (the tracker mirror and the outgoing synth), then install a
                    // synth built from the new instrument. If the build fails, keep
                    // the current one so audio never drops out.
                    AudioCommand::ReplaceSynth(new_sound_font) => {
                        tracker.clear_for_swap();
                        synth.note_off_all(true);
                        let settings = SynthesizerSettings::new(sample_rate);
                        match Synthesizer::new(&new_sound_font, &settings) {
                            Ok(next) => synth = next,
                            Err(e) => eprintln!(
                                "[cymbra-audio] swap synth build failed, keeping current: {e}"
                            ),
                        }
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

            // Mix the metronome click on top of the synth render. It is the same
            // mono signal in both channels and decays on its own; drop it once
            // finished so the steady path does nothing.
            if let Some(voice) = click.as_mut() {
                for i in 0..frames {
                    let s = voice.next_sample();
                    l[i] += s;
                    r[i] += s;
                }
                if !voice.is_active() {
                    click = None;
                }
            }

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
