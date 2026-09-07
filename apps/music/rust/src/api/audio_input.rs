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

/// What the capture thread can be told while running. Dropping the sender
/// (see [`audio_input_stop_capture`]) is the stop signal.
enum CaptureMsg {
    /// The running stream was invalidated (e.g. iOS "Audio route changed",
    /// fired by our own capture-session flip): drop it, let the route settle,
    /// and rebuild — the input twin of the output side's route self-healing.
    Rebuild,
    /// Close the stream and end the thread. Explicit rather than a sender
    /// drop: the thread holds its own sender clones (the error callbacks), so
    /// a disconnect can never be observed from inside.
    Stop,
}

/// Signal channel to the capture thread.
static CAPTURE_STOP: Mutex<Option<Sender<CaptureMsg>>> = Mutex::new(None);

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
            let (tx, rx) = channel::<CaptureMsg>();
            let thread_tx = tx.clone();
            *CAPTURE_STOP.lock().unwrap() = Some(tx);
            let generation = CAPTURE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
            thread::Builder::new()
                .name("audio-input".into())
                .spawn(move || {
                    if let Err(e) = capture_thread(rx, thread_tx) {
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
            if let Some(tx) = CAPTURE_STOP.lock().unwrap().take() {
                let _ = tx.send(CaptureMsg::Stop);
            }
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
    // The clip channel LOOPS by design (it exists for soundfont previews the
    // UI stops): the reference beep must be stopped explicitly, shortly after
    // it sounded — its silent tail makes the stop land noiselessly.
    let mut beep_at: Option<Instant> = None;
    while Instant::now() < deadline {
        if let Some(at) = beep_at
            && at.elapsed() > Duration::from_millis(200)
        {
            super::audio::stop_preview_clip();
            beep_at = None;
        }
        {
            let mut guard = CALIB.lock().unwrap();
            if let Some(detector) = guard.as_mut() {
                if detector.phase() == CalibPhase::ReadyToClick {
                    // A dedicated full-scale reference beep: the metronome's
                    // click sample is far too soft to be a reference (an iPad
                    // at max volume measured it near its own noise floor).
                    super::audio::play_preview_clip(reference_beep_wav());
                    beep_at = Some(Instant::now());
                    detector.click_emitted();
                }
                outcome = detector.take_outcome();
                if matches!(outcome, Some(CalibOutcome::TimedOut)) {
                    let (floor, peak, threshold) = detector.diagnostics();
                    platform_log::log_line(
                        "cymbra-audio-in",
                        &format!(
                            "calibration timeout: floor={floor:.5} peak={peak:.5} threshold={threshold:.5}"
                        ),
                    );
                }
            }
        }
        if outcome.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Whatever the exit, never leave the looping beep running.
    super::audio::stop_preview_clip();
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
fn capture_thread(
    rx: std::sync::mpsc::Receiver<CaptureMsg>,
    tx: Sender<CaptureMsg>,
) -> anyhow::Result<()> {
    let mut rebuilds_left = 5u8;
    let mut last_rebuild = Instant::now();
    loop {
        // Opened fresh on every pass: a rebuild re-resolves the device (the
        // route that killed the stream may have changed what "default" means)
        // and can come back at a different rate.
        let stream = open_capture(&tx)?;

        // Park until told otherwise; the stream lives on this stack frame.
        match rx.recv() {
            Ok(CaptureMsg::Rebuild) => {
                drop(stream);
                // The budget guards against an invalidation LOOP, not against
                // a long session: route flips arrive in clusters (every
                // calibration causes one), and a spent lifetime budget was
                // what silently killed detection minutes into real play.
                if last_rebuild.elapsed() > Duration::from_secs(10) {
                    rebuilds_left = 5;
                }
                last_rebuild = Instant::now();
                if rebuilds_left == 0 {
                    anyhow::bail!("input stream kept getting invalidated");
                }
                rebuilds_left -= 1;
                // Let the route change finish before reopening on its result;
                // then drain the burst (one flip can notify several times).
                thread::sleep(Duration::from_millis(250));
                loop {
                    use std::sync::mpsc::TryRecvError;
                    match rx.try_recv() {
                        Ok(CaptureMsg::Rebuild) => continue,
                        Ok(CaptureMsg::Stop) | Err(TryRecvError::Disconnected) => return Ok(()),
                        Err(TryRecvError::Empty) => break,
                    }
                }
            }
            // Stop (or a torn-down channel): the generation-guarded exit
            // cleanup in the spawn wrapper clears the published state.
            Ok(CaptureMsg::Stop) | Err(_) => return Ok(()),
        }
    }
}

/// Resolves the device, opens and starts the stream, publishes the state and
/// (re)installs the frame consumers at the actual rate.
fn open_capture(tx: &Sender<CaptureMsg>) -> anyhow::Result<cpal::Stream> {
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
        cpal::SampleFormat::F32 => build_capture::<f32>(&device, &config, channels, tx.clone())?,
        cpal::SampleFormat::I16 => build_capture::<i16>(&device, &config, channels, tx.clone())?,
        cpal::SampleFormat::U16 => build_capture::<u16>(&device, &config, channels, tx.clone())?,
        other => anyhow::bail!("unsupported input sample format {other:?}"),
    };
    stream.play()?;
    INPUT_SAMPLE_RATE.store(config.sample_rate, Ordering::Relaxed);
    *ACTIVE_INPUT.lock().unwrap() = device.description().ok().map(|d| d.name().to_string());
    platform_log::log_line(
        "cymbra-audio-in",
        &format!(
            "opened {:?} — {} Hz, {} ch",
            ACTIVE_INPUT.lock().unwrap().as_deref().unwrap_or("?"),
            config.sample_rate,
            channels
        ),
    );
    if DETECT_REQUESTED.load(Ordering::Relaxed) {
        install_detector(config.sample_rate);
    }
    // An in-flight calibration was feeding the dead stream: restart it at the
    // fresh rate — its poller sees a new baseline and re-emits the click, so
    // the run recovers instead of timing out on silence.
    {
        let mut calib = CALIB.lock().unwrap();
        if calib.is_some() {
            *calib = Some(CalibrationDetector::new(config.sample_rate));
        }
    }
    Ok(stream)
}

/// Builds the input stream for one sample format: downmix to mono `f32` and
/// feed the installed consumer.
fn build_capture<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    rebuild: Sender<CaptureMsg>,
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
                    if note.on {
                        platform_log::log_line(
                            "cymbra-detect",
                            &format!(
                                "emit p={} v={} at={}",
                                note.pitch, note.velocity, note.at_sample
                            ),
                        );
                    }
                    super::midi::emit_detected(to_midi_event(note, rate));
                }
                for line in detector.debug_log.drain(..) {
                    platform_log::log_line("cymbra-detect", &line);
                }
            }
        },
        move |e| {
            platform_log::log_line("cymbra-audio-in", &format!("stream error: {e}"));
            // A dead stream never recovers on its own (iOS invalidates it on
            // every route change — including the one our own capture-session
            // flip causes): ask the capture thread for a rebuild.
            let _ = rebuild.send(CaptureMsg::Rebuild);
        },
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

/// The calibration reference sound: 80 ms of a loud 1 kHz sine, faded at
/// both ends (no speaker pop, no end click), followed by 400 ms of silence —
/// the clip channel loops, and the explicit stop then lands in the silent
/// tail instead of cutting a sine mid-phase. In-memory mono 16-bit WAV for
/// the engine's clip path.
fn reference_beep_wav() -> Vec<u8> {
    const RATE: u32 = 48_000;
    const BEEP_MS: u32 = 80;
    const TAIL_MS: u32 = 400;
    let n = (RATE * BEEP_MS / 1000) as usize;
    let tail = (RATE * TAIL_MS / 1000) as usize;
    let fade = (RATE * 3 / 1000) as usize; // 3 ms
    let mut pcm = Vec::with_capacity((n + tail) * 2);
    for i in 0..n {
        let t = i as f64 / f64::from(RATE);
        let ramp_in = ((i as f64) / fade as f64).min(1.0);
        let ramp_out = (((n - 1 - i) as f64) / fade as f64).min(1.0);
        let sample = (2.0 * std::f64::consts::PI * 1000.0 * t).sin() * 0.6 * ramp_in * ramp_out;
        let v = (sample * f64::from(i16::MAX)) as i16;
        pcm.extend_from_slice(&v.to_le_bytes());
    }
    pcm.extend(std::iter::repeat_n(0u8, tail * 2));
    let data_len = pcm.len() as u32;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 2).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(&pcm);
    wav
}
