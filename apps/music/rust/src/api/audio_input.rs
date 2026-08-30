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

//! Microphone capture glue — device, thread and FFI plumbing (change:
//! add-acoustic-piano-input).
//!
//! Everything host-testable lives in [`super::audio_input_core`]; this module
//! is the thin seam over cpal's input side, and is excluded from the coverage
//! gate like the rest of the hardware glue (`midi.rs`, `audio.rs`).
//!
//! Threading: cpal streams are not `Send`, so a dedicated capture thread owns
//! the stream — built on start, dropped on stop — exactly as the output side
//! keeps its stream on the audio thread. The capture callback's only job is
//! downmixing to mono `f32` and feeding whatever consumer is installed
//! (today: the calibration detector; later: note detection).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use flutter_rust_bridge::frb;

use super::audio_input_core::{
    CALIB_BASELINE_MS, CALIB_LISTEN_TIMEOUT_MS, CalibOutcome, CalibPhase, CalibrationDetector,
    CaptureLifecycle, CaptureTransition, DetectedNote, NoteDetector, classify_input_route,
    input_route_verdict, resolve_input_device,
};
use super::midi::{MidiEvent, MidiEventKind};
use super::platform_log;

/// How a capture route is connected, as the app reasons about it. The input
/// twin of the output side's `AudioRouteKind` — classified from the
/// platform's stable port-type identifier, never from a display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRouteKind {
    /// The device's own microphone.
    Builtin,
    /// A wired headset microphone or line input.
    Wired,
    /// A USB audio device — including a wireless mic's USB-C receiver, which
    /// enumerates as class-compliant USB audio and is wired-equivalent.
    Usb,
    /// A Bluetooth microphone (the HFP/SCO voice profile).
    Bluetooth,
    /// Anything the platform does not describe well enough to classify —
    /// including every desktop device, where cpal exposes no transport type.
    Other,
}

/// The acquisition verdict for a route: Bluetooth capture is refused outright
/// (spec: Bluetooth Input Refusal), everything else is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputRouteVerdict {
    /// The route may acquire.
    Accepted,
    /// A Bluetooth microphone: voice-profile bandwidth and jitter are
    /// incompatible with note timing. Hard refusal, with copy in the app.
    RefusedBluetooth,
}

/// An audio input the engine can capture from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioInputInfo {
    /// The host's name for the device — display + persistence handle.
    pub name: String,
    /// How it is connected (desktop enumeration: always [`InputRouteKind::Other`]).
    pub kind: InputRouteKind,
}

/// The terminal result of one calibration run (spec: Measured Input-Offset
/// Calibration). `detected == false` means the click was never heard — the
/// app shows guidance and stores nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct InputCalibrationResult {
    /// Whether the reference click was detected at the microphone.
    pub detected: bool,
    /// The measured emission→capture round trip, when detected.
    pub latency_ms: Option<f64>,
}

/// The engine-wide capture lifecycle. The core decides; this module executes.
static LIFECYCLE: Mutex<CaptureLifecycle> = Mutex::new(CaptureLifecycle::Idle);

/// Signal channel to the capture thread: any message (or disconnect) stops it.
static CAPTURE_STOP: Mutex<Option<Sender<()>>> = Mutex::new(None);

/// The rate of the running capture stream, published by the capture thread
/// once the device opens (0 = not yet known / not capturing).
static INPUT_SAMPLE_RATE: AtomicU32 = AtomicU32::new(0);

/// The calibration consumer of captured mono frames, installed for the
/// duration of one calibration run.
static CALIB: Mutex<Option<CalibrationDetector>> = Mutex::new(None);

/// The note-detection consumer (change: add-acoustic-piano-input). Installed
/// by the capture thread once the device rate is known, whenever
/// [`DETECT_REQUESTED`] stands.
static DETECT: Mutex<Option<NoteDetector>> = Mutex::new(None);

/// Whether the app wants note detection running. The capture thread reads it
/// after opening the device (the detector needs the real sample rate).
static DETECT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// The score's current expected-pitch window, kept so a detector installed
/// later starts with the live set.
static EXPECTED: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// The input device the user pinned (spec: Desktop Capture Device Selection);
/// `None` follows the system default. Resolved at open time with a fallback
/// to the default when absent.
static SELECTED_INPUT: Mutex<Option<String>> = Mutex::new(None);

/// The device the running capture actually opened, published by the capture
/// thread — reported instead of the requested name so the UI shows reality
/// after a fallback (the output side's exact convention).
static ACTIVE_INPUT: Mutex<Option<String>> = Mutex::new(None);

/// Capture-thread generation. A device swap (stop + start) can leave the OLD
/// thread still unwinding while the NEW one is already publishing state; the
/// exit cleanup only applies when its generation is still the current one, so
/// a stale thread can never idle the fresh lifecycle or clear its device.
static CAPTURE_GEN: AtomicU32 = AtomicU32::new(0);

/// Starts microphone capture, returning whether capture is running afterwards.
/// Idempotent: calling while already capturing changes nothing. Failure to
/// open a device (no input hardware, permission denied at the OS layer) leaves
/// the lifecycle idle and returns `false` — graceful degradation, no panic.
#[frb(sync)]
pub fn audio_input_start_capture() -> bool {
    let mut l = LIFECYCLE.lock().unwrap();
    match l.request_start() {
        CaptureTransition::Open => {
            let (tx, rx) = channel::<()>();
            *CAPTURE_STOP.lock().unwrap() = Some(tx);
            let generation = CAPTURE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
            thread::Builder::new()
                .name("audio-input".into())
                .spawn(move || {
                    if let Err(e) = capture_thread(rx) {
                        platform_log::log_line("cymbra-audio-in", &format!("capture failed: {e}"));
                    }
                    // Honest state whichever way the thread ended: device
                    // gone, open failure, or a regular stop — but only while
                    // this thread is still the current generation (a device
                    // swap may already be running its replacement).
                    if CAPTURE_GEN.load(Ordering::SeqCst) == generation {
                        INPUT_SAMPLE_RATE.store(0, Ordering::Relaxed);
                        *ACTIVE_INPUT.lock().unwrap() = None;
                        let _ = LIFECYCLE.lock().unwrap().request_stop();
                    }
                })
                .expect("spawn audio-input thread");
        }
        CaptureTransition::Close | CaptureTransition::None => {}
    }
    l.is_running()
}

/// Stops microphone capture and releases the device. Idempotent.
#[frb(sync)]
pub fn audio_input_stop_capture() {
    let mut l = LIFECYCLE.lock().unwrap();
    match l.request_stop() {
        CaptureTransition::Close => {
            // Dropping the sender disconnects the thread's recv() and ends it.
            *CAPTURE_STOP.lock().unwrap() = None;
        }
        CaptureTransition::Open | CaptureTransition::None => {}
    }
}

/// Whether microphone capture is currently running.
#[frb(sync)]
pub fn audio_input_is_capturing() -> bool {
    LIFECYCLE.lock().unwrap().is_running()
}

/// Names of the available capture devices, default first. Desktop hosts
/// expose no transport type, so every kind is [`InputRouteKind::Other`];
/// mobile route kinds come from the platform session via
/// [`classify_input_route_token`].
#[frb(sync)]
pub fn list_audio_inputs() -> Vec<AudioInputInfo> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.description().ok().map(|d| d.name().to_string()));
    let mut out: Vec<AudioInputInfo> = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(desc) = device.description() {
                out.push(AudioInputInfo {
                    name: desc.name().to_string(),
                    kind: InputRouteKind::Other,
                });
            }
        }
    }
    if let Some(default_name) = default_name {
        out.sort_by_key(|i| i.name != default_name);
    }
    out
}

/// Classifies a platform port-type identifier (iOS `AVAudioSession.Port` raw
/// value, Android `AudioDeviceInfo` type name) into a route kind. Unknown
/// tokens degrade to [`InputRouteKind::Other`].
#[frb(sync)]
pub fn classify_input_route_token(token: String) -> InputRouteKind {
    classify_input_route(&token)
}

/// The acquisition verdict for a route kind.
#[frb(sync)]
pub fn input_route_verdict_for(kind: InputRouteKind) -> InputRouteVerdict {
    input_route_verdict(kind)
}

/// Starts acoustic note detection (spec: Detected Notes Enter The Standard
/// Input Stream): ensures capture runs and installs the detector as soon as
/// the device rate is known. Returns whether capture is running. Detection
/// emits nothing until [`set_expected_pitches`] provides a non-empty window.
#[frb(sync)]
pub fn audio_input_start_detection() -> bool {
    DETECT_REQUESTED.store(true, Ordering::Relaxed);
    let running = audio_input_start_capture();
    let rate = INPUT_SAMPLE_RATE.load(Ordering::Relaxed);
    if running && rate != 0 {
        install_detector(rate);
    }
    running
}

/// Stops acoustic note detection. Capture is left to its own lifecycle — the
/// app owns when the microphone closes (spec: Microphone Capture Lifecycle).
#[frb(sync)]
pub fn audio_input_stop_detection() {
    DETECT_REQUESTED.store(false, Ordering::Relaxed);
    *DETECT.lock().unwrap() = None;
}

/// Replaces the expected-pitch window the presence stage evaluates (spec:
/// Score-Informed Presence Detection). The app pushes it on every playhead /
/// Wait-gate change; an empty list idles the detector.
#[frb(sync)]
pub fn set_expected_pitches(pitches: Vec<u8>) {
    *EXPECTED.lock().unwrap() = pitches.clone();
    if let Some(d) = DETECT.lock().unwrap().as_mut() {
        d.set_expected(pitches);
    }
}

/// Builds (or rebuilds) the detector at the given rate, seeded with the live
/// expected window.
fn install_detector(rate: u32) {
    let mut detector = NoteDetector::new(rate);
    detector.set_expected(EXPECTED.lock().unwrap().clone());
    *DETECT.lock().unwrap() = Some(detector);
}

/// Chooses the capture device (`None` = follow the system default) and
/// applies it to a capture already running by rebuilding the stream (spec:
/// Desktop Capture Device Selection). An absent name degrades to the default
/// at open time rather than failing.
#[frb(sync)]
pub fn set_audio_input(name: Option<String>) {
    *SELECTED_INPUT.lock().unwrap() = name;
    // Rebuild a running capture on the new device: the lifecycle stays open
    // from the caller's point of view — stop + start swaps the stream, and
    // the capture thread reinstalls the detector at the new device's rate.
    if audio_input_is_capturing() {
        audio_input_stop_capture();
        audio_input_start_capture();
    }
}

/// The device the running capture is actually acquiring from, or `None` when
/// idle. Reality, not the request: a fallback shows the default's name.
#[frb(sync)]
pub fn active_audio_input() -> Option<String> {
    ACTIVE_INPUT.lock().unwrap().clone()
}

/// The device a capture is acquiring from right now — or, when idle, the one
/// a capture WOULD open (the pinned selection resolved against what is
/// present, falling back to the system default). The calibration store keys
/// measurements by this name, so it must always describe the device that
/// actually answers.
#[frb(sync)]
pub fn resolved_audio_input() -> Option<String> {
    if let Some(active) = ACTIVE_INPUT.lock().unwrap().clone() {
        return Some(active);
    }
    let host = cpal::default_host();
    let available: Vec<String> = host
        .input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
                .collect()
        })
        .unwrap_or_default();
    let requested = SELECTED_INPUT.lock().unwrap().clone();
    if let Some(name) = resolve_input_device(requested.as_deref(), &available) {
        return Some(name.to_string());
    }
    host.default_input_device()
        .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()))
}

/// Runs one input-offset calibration: captures, observes the noise floor,
/// emits the reference click through the existing output path, and measures
/// when it arrives back at the microphone. Blocking — the bridge runs it off
/// the UI thread and hands Dart a future. Capture started here is stopped
/// here; capture already running stays running.
pub fn run_input_calibration() -> InputCalibrationResult {
    let was_capturing = audio_input_is_capturing();
    if !audio_input_start_capture() {
        return InputCalibrationResult {
            detected: false,
            latency_ms: None,
        };
    }

    // Wait for the device to open and publish its rate.
    let opened = wait_until(Duration::from_secs(2), || {
        INPUT_SAMPLE_RATE.load(Ordering::Relaxed) != 0
    });
    if !opened {
        if !was_capturing {
            audio_input_stop_capture();
        }
        return InputCalibrationResult {
            detected: false,
            latency_ms: None,
        };
    }

    *CALIB.lock().unwrap() = Some(CalibrationDetector::new(
        INPUT_SAMPLE_RATE.load(Ordering::Relaxed),
    ));

    // Poll the detector: emit the click when the baseline is done, then wait
    // for its outcome. The safety deadline covers a wedged capture stream —
    // the detector itself cannot time out if no frames arrive.
    let deadline =
        Instant::now() + Duration::from_millis(CALIB_BASELINE_MS + CALIB_LISTEN_TIMEOUT_MS + 3000);
    let mut outcome: Option<CalibOutcome> = None;
    while Instant::now() < deadline {
        {
            let mut guard = CALIB.lock().unwrap();
            if let Some(detector) = guard.as_mut() {
                if detector.phase() == CalibPhase::ReadyToClick {
                    super::audio::metronome_click(false);
                    detector.click_emitted();
                }
                outcome = detector.take_outcome();
            }
        }
        if outcome.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    *CALIB.lock().unwrap() = None;
    if !was_capturing {
        audio_input_stop_capture();
    }

    match outcome {
        Some(CalibOutcome::Detected { latency_ms }) => InputCalibrationResult {
            detected: true,
            latency_ms: Some(latency_ms),
        },
        Some(CalibOutcome::TimedOut) | None => InputCalibrationResult {
            detected: false,
            latency_ms: None,
        },
    }
}

fn wait_until(timeout: Duration, mut cond: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    cond()
}

/// Owns the cpal input stream for the whole capture: builds it, publishes the
/// rate, then parks until the stop signal (or sender drop) ends it.
fn capture_thread(rx: std::sync::mpsc::Receiver<()>) -> anyhow::Result<()> {
    let host = cpal::default_host();
    // Resolve the pinned device against what is present right now; anything
    // absent (or no pin) opens the system default — never a failure.
    let requested = SELECTED_INPUT.lock().unwrap().clone();
    let available: Vec<String> = host
        .input_devices()
        .map(|devices| {
            devices
                .filter_map(|d| d.description().ok().map(|desc| desc.name().to_string()))
                .collect()
        })
        .unwrap_or_default();
    let resolved = resolve_input_device(requested.as_deref(), &available).map(str::to_string);
    let device = match resolved {
        Some(name) => host
            .input_devices()?
            .find(|d| d.description().ok().is_some_and(|desc| desc.name() == name))
            .ok_or_else(|| anyhow::anyhow!("selected input disappeared"))?,
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("no input device"))?,
    };
    let supported = device.default_input_config()?;
    let config: cpal::StreamConfig = supported.config();
    let channels = config.channels as usize;

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => build_capture::<f32>(&device, &config, channels)?,
        cpal::SampleFormat::I16 => build_capture::<i16>(&device, &config, channels)?,
        cpal::SampleFormat::U16 => build_capture::<u16>(&device, &config, channels)?,
        other => anyhow::bail!("unsupported input sample format {other:?}"),
    };
    stream.play()?;
    INPUT_SAMPLE_RATE.store(config.sample_rate, Ordering::Relaxed);
    *ACTIVE_INPUT.lock().unwrap() = device.description().ok().map(|d| d.name().to_string());
    if DETECT_REQUESTED.load(Ordering::Relaxed) {
        install_detector(config.sample_rate);
    }

    // Any message or a dropped sender means stop; the stream drops with us.
    // The generation-guarded exit cleanup (in the spawn wrapper) clears the
    // published state.
    let _ = rx.recv();
    Ok(())
}

/// Builds the input stream for one sample format: downmix to mono `f32` and
/// feed the installed consumer.
fn build_capture<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
) -> anyhow::Result<cpal::Stream>
where
    T: cpal::SizedSample,
    f32: FromSample<T>,
{
    let mut mono: Vec<f32> = Vec::new();
    let stream = device.build_input_stream(
        *config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            mono.clear();
            mono.reserve(data.len() / channels.max(1));
            for frame in data.chunks_exact(channels.max(1)) {
                let mut acc = 0.0f32;
                for &s in frame {
                    acc += f32::from_sample(s);
                }
                mono.push(acc / channels.max(1) as f32);
            }
            if let Some(detector) = CALIB.lock().unwrap().as_mut() {
                detector.feed(&mono);
            }
            if let Some(detector) = DETECT.lock().unwrap().as_mut() {
                let rate = detector.sample_rate();
                for note in detector.feed(&mono) {
                    super::midi::emit_detected(to_midi_event(note, rate));
                }
            }
        },
        |e| platform_log::log_line("cymbra-audio-in", &format!("stream error: {e}")),
        None,
    )?;
    Ok(stream)
}

/// A detection emission as the normalized event every downstream consumer
/// reads. The timestamp is the onset's, on the capture stream's sample clock.
fn to_midi_event(note: DetectedNote, rate: u32) -> MidiEvent {
    MidiEvent {
        kind: if note.on {
            MidiEventKind::NoteOn
        } else {
            MidiEventKind::NoteOff
        },
        pitch: note.pitch,
        velocity: note.velocity,
        channel: 0,
        timestamp_ms: note.at_sample * 1000 / u64::from(rate),
    }
}
