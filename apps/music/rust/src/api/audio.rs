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

//! Polyphonic SoundFont piano synthesis via `rustysynth`, rendered with `cpal`
//! to a **selectable** output device (the system default until one is chosen).
//!
//! This is the hardware/thread/FFI glue — kept out of the coverage gate like
//! [`super::midi`]. All genuinely testable logic (event model, MIDI mapping,
//! voice bookkeeping, output resolution and route classification) lives in
//! [`super::audio_core`].
//!
//! Threading model: `cpal::Stream` is not `Send` on CoreAudio, so it must be
//! created and dropped on the same thread. [`audio_init`] therefore spawns one
//! dedicated audio thread that owns the stream for the whole process; the FFI
//! entry points only push lock-free [`AudioEvent`]s onto an `mpsc` channel that
//! the audio callback drains each block (no locks/allocation on the hot path).
//!
//! Device changes cannot ride that queue — a stream cannot rebuild itself from
//! inside its own callback — so they travel on a second channel to the audio
//! thread's control loop, which rebuilds the stream and synthesizer around the
//! *same* event queue (change: add-audio-output-routing).
//!
//! cpal backends: CoreAudio (macOS/iOS), WASAPI (Windows), ALSA (Linux), AAudio
//! (Android — using the NDK context initialized in `JNI_OnLoad`, see lib.rs).

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use flutter_rust_bridge::frb;
use rustysynth::SoundFont;

use super::audio_core::{
    AudioEvent, OutputChoice, decode_wav_pcm, is_valid_soundfont, order_outputs,
    preferred_buffer_frames, resolve_output_device, route_kind_of,
};
use super::platform_log;
use super::renderer::{RenderCommand, Renderer};

/// How an audio output is connected, as the app reasons about it (change:
/// add-audio-output-routing). The **kind** — never the device's name — is what
/// drives the wireless warning, so a host reporting a connection this build does
/// not know degrades to [`AudioRouteKind::Other`] instead of breaking the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRouteKind {
    /// The machine's own speakers / integrated audio.
    Builtin,
    /// Wired headphones or a wired headset.
    Headphones,
    /// A wireless (Bluetooth) route — delayed, hence warned about.
    Bluetooth,
    /// A USB audio device (interface, USB-audio piano, USB headset).
    Usb,
    /// Anything the host does not describe well enough to classify.
    Other,
}

/// An audio output the engine can render to: what to show the user, and what
/// kind of connection it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioOutputInfo {
    /// The host's name for the device — also the handle selection is persisted
    /// under, since that is the only stable-ish one `cpal` offers.
    pub name: String,
    /// How it is connected (see [`AudioRouteKind`]).
    pub kind: AudioRouteKind,
}

/// A lifecycle request the **real-time callback cannot serve itself**: a stream
/// cannot be rebuilt from inside its own callback, so device changes travel on
/// their own channel, straight to the audio thread's control loop, while
/// [`RenderCommand`]s keep flowing lock-free into the callback.
#[cfg_attr(target_os = "android", allow(dead_code))] // cpal path: unused on Android
enum DeviceCommand {
    /// Rebuild the output stream on the named device (`None` = follow the
    /// system default).
    SetOutput(Option<String>),
    /// Rebuild on the *current* request — the device in use went away, so
    /// re-resolving falls the app back to the system default instead of leaving
    /// it silent.
    Reopen,
}

/// The queue the audio callback drains, shared so a rebuilt stream keeps reading
/// the *same* queue: notes pressed across a device change are neither lost nor
/// reordered. Only the callback ever locks it, so it is uncontended.
#[cfg_attr(target_os = "android", allow(dead_code))] // cpal path: unused on Android
type SharedEvents = Arc<Mutex<Receiver<RenderCommand>>>;

/// Sender used by the FFI entry points to hand commands to the audio thread.
/// Published as soon as [`audio_init`] starts (so note events queue while the
/// device spins up) and cleared if setup fails, so calls without a working device
/// are silently dropped — graceful degradation.
static EVENT_TX: Mutex<Option<Sender<RenderCommand>>> = Mutex::new(None);

/// Sender for [`DeviceCommand`]s. Published and cleared alongside [`EVENT_TX`],
/// so selecting an output before the engine is up is a silent no-op rather than
/// an error.
static DEVICE_TX: Mutex<Option<Sender<DeviceCommand>>> = Mutex::new(None);

/// The output the engine is **actually** rendering to, published by the audio
/// thread after each successful open. Reported instead of the requested name so
/// the UI shows reality after a fallback.
static ACTIVE_OUTPUT: Mutex<Option<AudioOutputInfo>> = Mutex::new(None);

/// Publishes the Android output's render rate for the drift monitor. Called by
/// [`super::android_output`] whenever the platform (re)opens at a rate.
#[cfg(target_os = "android")]
pub(crate) fn set_stream_rate(rate: u32) {
    STREAM_SAMPLE_RATE.store(rate, Ordering::Relaxed);
}

/// Guards against launching more than one audio engine. Reset on setup failure
/// so a later call can retry.
static INIT_STARTED: AtomicBool = AtomicBool::new(false);

/// Frames the callback has rendered since the engine started.
///
/// The one number that says whether the engine is keeping time: divided by
/// elapsed wall time it gives the rate the output is actually pulling us at,
/// which must match the stream's sample rate. Bumped with a relaxed add — the
/// only work the real-time callback is allowed to do for it.
static RENDERED_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Sample rate of the stream currently open, for the drift check. 0 = none yet.
static STREAM_SAMPLE_RATE: AtomicU32 = AtomicU32::new(0);

/// Records frames rendered, for the drift monitor. The cpal callback bumps
/// [`RENDERED_FRAMES`] directly; this is the same probe for the Android pull
/// path, which lives in another module ([`super::android_output`]).
#[cfg(target_os = "android")]
pub(crate) fn note_frames_rendered(frames: u64) {
    RENDERED_FRAMES.fetch_add(frames, Ordering::Relaxed);
}

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

    let (tx, rx) = mpsc::channel::<RenderCommand>();
    let (device_tx, device_rx) = mpsc::channel::<DeviceCommand>();
    // Publish the senders now so notes pressed — and an output remembered from
    // the last session — queue up during startup; they are applied once the
    // audio thread is up.
    *EVENT_TX.lock().unwrap() = Some(tx);
    *DEVICE_TX.lock().unwrap() = Some(device_tx);

    spawn_drift_monitor();
    // Android: the platform's `AudioTrack` owns the stream and pulls from the
    // engine (see [`super::android_output`]), so no `cpal` thread is started.
    #[cfg(target_os = "android")]
    {
        let _ = device_rx;
        thread::spawn(move || {
            // The renderer's rate is chosen by the platform side (it knows the
            // route), so install hands the parsed font to `android_output`,
            // which builds — and can rebuild — the renderer at that rate.
            let installed = load_sound_font(&sf2_path)
                .map_err(|e| e.to_string())
                .and_then(|sound_font| super::android_output::install(sound_font, rx));
            if let Err(e) = installed {
                platform_log::log_line("cymbra-audio", &format!("disabled: {e}"));
                *EVENT_TX.lock().unwrap() = None;
                INIT_STARTED.store(false, Ordering::SeqCst);
            }
        });
    }

    #[cfg(not(target_os = "android"))]
    thread::spawn(move || {
        if let Err(e) = run_audio_thread(sf2_path, Arc::new(Mutex::new(rx)), device_rx) {
            platform_log::log_line("cymbra-audio", &format!("disabled: {e}"));
            // Drop the senders so further events are silent no-ops, and let a
            // future call retry.
            *EVENT_TX.lock().unwrap() = None;
            *DEVICE_TX.lock().unwrap() = None;
            *ACTIVE_OUTPUT.lock().unwrap() = None;
            INIT_STARTED.store(false, Ordering::SeqCst);
        }
    });
}

/// Lists the host's audio output devices, the system default first (change:
/// add-audio-output-routing). Empty when the host cannot be reached — a caller
/// showing an empty list is the honest outcome, not an error.
///
/// Enumeration only: it neither opens nor changes the current output.
#[frb(sync)]
pub fn list_audio_outputs() -> Vec<AudioOutputInfo> {
    let host = cpal::default_host();
    let described = describe_outputs(&host);
    let default_name = host.default_output_device().and_then(|d| device_name(&d));
    let names: Vec<String> = described.iter().map(|(n, _)| n.clone()).collect();
    order_outputs(&names, default_name.as_deref())
        .into_iter()
        .map(|name| {
            let kind = described
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, k)| *k)
                .unwrap_or(AudioRouteKind::Other);
            AudioOutputInfo { name, kind }
        })
        .collect()
}

/// Chooses the audio output every app sound is rendered to: `None` follows the
/// system default, a name selects that device (change:
/// add-audio-output-routing).
///
/// A silent no-op if the engine is not running. Returns immediately: the audio
/// thread rebuilds its stream and synthesizer on the new device — the SoundFont
/// stays in memory, so the swap does not re-read it. **A device that will not
/// open leaves the working stream untouched**; the app is never torn down to
/// chase a broken device. Read [`active_audio_output`] afterwards to see what is
/// actually in use.
#[frb(sync)]
pub fn set_audio_output(name: Option<String>) {
    if let Some(tx) = DEVICE_TX.lock().unwrap().as_ref() {
        let _ = tx.send(DeviceCommand::SetOutput(name));
    }
}

/// The output the engine is currently rendering to, or `None` when audio is not
/// running. May differ from the last [`set_audio_output`] request — after a
/// fallback this reports the device actually in use.
#[frb(sync)]
pub fn active_audio_output() -> Option<AudioOutputInfo> {
    ACTIVE_OUTPUT.lock().unwrap().clone()
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

/// Releases every sounding voice on every channel (stop / restart / seek /
/// loop) — melodic and drum alike.
#[frb(sync)]
pub fn all_notes_off() {
    send(AudioEvent::AllOff);
}

/// Sounds a percussion stroke for General MIDI `key` at `velocity` on the drum
/// channel, where the active kit font's bank-128 presets resolve (change:
/// add-drum-audio-channel). The melodic pair is byte-for-byte untouched.
#[frb(sync)]
pub fn drum_on(key: u8, velocity: u8) {
    send(AudioEvent::drum_on(key, velocity));
}

/// Releases the drum voice for `key`. Kit voices are mostly self-terminating
/// one-shots; the release keeps the voice bookkeeping exact.
#[frb(sync)]
pub fn drum_off(key: u8) {
    send(AudioEvent::drum_off(key));
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
            let _ = tx.send(RenderCommand::ReplaceSynth(sound_font, None));
        }
        Err(e) => platform_log::log_line(
            "cymbra-audio",
            &format!("soundfont swap skipped, keeping current: {e}"),
        ),
    });
}

/// Swaps the SoundFont like [`audio_load_soundfont`], but resolves only once
/// the outcome is KNOWN (change: add-drum-audio-channel): `true` when the
/// incoming font is installed on the audio thread, `false` when the swap
/// failed (unreadable/invalid file, build failure, engine not running, or the
/// audio thread unresponsive) and the previous font was kept. The player's
/// percussion-readiness gate awaits this so a drum score never sounds through
/// the still-loaded piano font. Runs on a worker (not `#[frb(sync)]`), so the
/// Dart side gets a Future without blocking the UI isolate.
pub fn audio_load_soundfont_awaited(sf2_path: String) -> bool {
    let tx = match EVENT_TX.lock().unwrap().as_ref() {
        Some(tx) => tx.clone(),
        None => return false,
    };
    let sound_font = match load_sound_font(&sf2_path) {
        Ok(f) => f,
        Err(e) => {
            platform_log::log_line(
                "cymbra-audio",
                &format!("soundfont swap skipped, keeping current: {e}"),
            );
            return false;
        }
    };
    let (ack_tx, ack_rx) = std::sync::mpsc::channel();
    if tx
        .send(RenderCommand::ReplaceSynth(sound_font, Some(ack_tx)))
        .is_err()
    {
        return false;
    }
    // The audio callback drains the queue once per block (~10 ms); a few
    // seconds means the thread is gone — report not-installed rather than
    // hang the caller.
    match ack_rx.recv_timeout(std::time::Duration::from_secs(3)) {
        Ok(installed) => installed,
        Err(_) => {
            platform_log::log_line(
                "cymbra-audio",
                "soundfont swap ack timed out; treating as not installed",
            );
            false
        }
    }
}

/// Reads a local `.sf2`'s preset-bank family evidence (change:
/// add-drum-audio-channel): whether it declares bank-128 (kit) presets and/or
/// melodic presets — the app's import detection. `None` when the file cannot
/// be read or is not a well-formed SoundFont ("cannot verify", never a
/// family).
pub fn soundfont_family_evidence(sf2_path: String) -> Option<SoundFontFamilyEvidence> {
    let bytes = std::fs::read(&sf2_path).ok()?;
    let e = cymbra_sf2_meta::family_evidence(&bytes).ok()?;
    Some(SoundFontFamilyEvidence {
        has_percussion_presets: e.has_percussion_presets,
        has_melodic_presets: e.has_melodic_presets,
    })
}

/// A `.sf2`'s preset-bank evidence, bridged (change: add-drum-audio-channel).
pub struct SoundFontFamilyEvidence {
    /// At least one bank-128 (drum kit) preset.
    pub has_percussion_presets: bool,
    /// At least one melodic-bank preset.
    pub has_melodic_presets: bool,
}

/// Sounds a short metronome click — a synthesized tick mixed into the output
/// independently of the piano SoundFont. `accent` marks the downbeat (higher and
/// louder). Self-terminating: there is no matching off.
#[frb(sync)]
pub fn metronome_click(accent: bool) {
    send(AudioEvent::Click { accent });
}

/// Plays a server-rendered SoundFont **preview clip** (a 16-bit PCM WAV) by mixing it
/// into the engine's output, looping until [`stop_preview_clip`] (change:
/// add-soundfont-entitlement-previews). This lets the app audition a **locked** reward
/// font without downloading its `.sf2` — reusing the same cross-platform audio engine
/// as the synth (no third-party audio plugin). A silent no-op if the engine is not
/// running or the bytes are not a decodable WAV.
#[frb(sync)]
pub fn play_preview_clip(wav_bytes: Vec<u8>) {
    let Some((sample_rate, pcm)) = decode_wav_pcm(&wav_bytes) else {
        return;
    };
    if let Some(tx) = EVENT_TX.lock().unwrap().as_ref() {
        let _ = tx.send(RenderCommand::PlayClip { pcm, sample_rate });
    }
}

/// Stops the preview clip started by [`play_preview_clip`] (silent no-op if none).
#[frb(sync)]
pub fn stop_preview_clip() {
    if let Some(tx) = EVENT_TX.lock().unwrap().as_ref() {
        let _ = tx.send(RenderCommand::StopClip);
    }
}

/// Watches whether the output is **keeping time**, and says so when it is not.
///
/// A healthy output pulls the callback at exactly the stream's sample rate. Some
/// routes do not: AAudio driving a USB-audio device on a Samsung Tab S6 Lite
/// (Android 13) was measured pulling +45% one session and −16% the next, never
/// the stream's rate — so the engine either runs ahead until what is heard is
/// tens of seconds behind, or is starved. Either way the output is unusable, and
/// the failure is **invisible** from inside the app: no error is raised and the
/// stream reports itself healthy. Hence one line in the log when it happens.
///
/// Silent while the output behaves, and silent for the first period after a
/// (re)open, which is a legitimate burst while the buffer fills. Runs on its own
/// thread; the real-time callback only bumps a counter. One thread per process:
/// `audio_init` can be re-reached after a setup failure, and each retry must not
/// stack another monitor.
fn spawn_drift_monitor() {
    static MONITOR_STARTED: AtomicBool = AtomicBool::new(false);
    if MONITOR_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::spawn(|| {
        const PERIOD: Duration = Duration::from_secs(5);
        /// How far the pull rate may stray from the sample rate before it counts
        /// as drift rather than jitter.
        const TOLERANCE: f64 = 0.15;

        let mut last_frames = RENDERED_FRAMES.load(Ordering::Relaxed);
        let mut last_at = Instant::now();
        let mut last_rate = STREAM_SAMPLE_RATE.load(Ordering::Relaxed);
        let mut consecutive = 0u32;
        let mut reported = false;
        loop {
            thread::sleep(PERIOD);
            let frames = RENDERED_FRAMES.load(Ordering::Relaxed);
            let now = Instant::now();
            let elapsed = now.duration_since(last_at).as_secs_f64();
            let rendered = frames.saturating_sub(last_frames);
            let rate = STREAM_SAMPLE_RATE.load(Ordering::Relaxed);
            let reopened = rate != last_rate;
            last_frames = frames;
            last_at = now;
            last_rate = rate;

            // Nothing open yet, or the stream just changed: no verdict this round.
            if rate == 0 || rendered == 0 || elapsed <= 0.0 || reopened {
                consecutive = 0;
                reported = false;
                continue;
            }
            let pulled = rendered as f64 / elapsed;
            let drift = (pulled - rate as f64) / rate as f64;
            if drift.abs() <= TOLERANCE {
                consecutive = 0;
                reported = false;
                continue;
            }
            // A fresh stream legitimately bursts while its buffer fills, so one
            // bad period proves nothing: only a *sustained* mismatch is the
            // failure worth a line — and one line per episode, not per period.
            consecutive += 1;
            if consecutive >= 2 && !reported {
                platform_log::log_line(
                    "cymbra-audio",
                    &format!(
                        "output is not keeping time: pulled {pulled:.0} frames/s on a {rate} Hz \
                     stream ({:+.0}%) — {}",
                        drift * 100.0,
                        if drift > 0.0 {
                            "the engine is running ahead of the device and what is heard falls further \
                         behind every second"
                        } else {
                            "the device is starving the engine and the output will break up"
                        }
                    ),
                );
                reported = true;
            }
        }
    });
}

/// Pushes a control event to the audio thread if the engine is running;
/// otherwise a silent no-op.
pub(crate) fn send(event: AudioEvent) {
    if let Some(tx) = EVENT_TX.lock().unwrap().as_ref() {
        let _ = tx.send(RenderCommand::Control(event));
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
    // Choke groups a stereo pair shares are split on the way in (change:
    // add-drum-audio-channel — beta fix): `rustysynth` reuses one voice slot for
    // every region of a class, so an unsplit hi-hat loses the half its own
    // note-on started and comes out hard right (silent on a mono output). The
    // patch touches a few words of the `pdta` hydra, which is read into memory
    // — a few hundred KB at most — while the sample data still streams.
    let sound_font = match split_choke_groups(path) {
        Some((pdta_at, pdta)) => {
            let head = File::open(path)
                .map_err(|e| anyhow!("open SoundFont {path}: {e}"))?
                .take(pdta_at);
            let mut tail = File::open(path).map_err(|e| anyhow!("open SoundFont {path}: {e}"))?;
            tail.seek(SeekFrom::Start(pdta_at + pdta.len() as u64))
                .map_err(|e| anyhow!("read SoundFont {path}: {e}"))?;
            let mut reader = BufReader::new(head.chain(Cursor::new(pdta)).chain(tail));
            SoundFont::new(&mut reader)
        }
        None => {
            let file = File::open(path).map_err(|e| anyhow!("open SoundFont {path}: {e}"))?;
            let mut reader = BufReader::new(file);
            SoundFont::new(&mut reader)
        }
    }
    .map_err(|e| anyhow!("invalid SoundFont: {e}"))?;
    Ok(Arc::new(sound_font))
}

/// The font's `pdta` LIST body with [`cymbra_sf2_meta::stereo_exclusive_class_patches`]
/// applied, and the offset it starts at — the two things needed to stream the
/// file with that one chunk replaced.
///
/// `None` whenever there is nothing to do: no shared choke group, no readable
/// hydra, or a file that does not describe itself sanely. The font is then
/// loaded exactly as it is on disk, so a font this cannot parse is never
/// *worse* off than before.
fn split_choke_groups(path: &str) -> Option<(u64, Vec<u8>)> {
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    // Past "RIFF<size>sfbk", already validated by the caller.
    let mut pos = 12u64;
    while pos + 8 <= len {
        file.seek(SeekFrom::Start(pos)).ok()?;
        let mut header = [0u8; 12];
        file.read_exact(&mut header[..8]).ok()?;
        let size = u64::from(u32::from_le_bytes(header[4..8].try_into().ok()?));
        let body = pos + 8;
        if &header[..4] == b"LIST" && size >= 4 {
            file.read_exact(&mut header[8..12]).ok()?;
            if &header[8..12] == b"pdta" {
                // A chunk claiming more than the file holds is not a hydra.
                let body_len = usize::try_from(size - 4)
                    .ok()
                    .filter(|n| body + 4 + *n as u64 <= len)?;
                let mut pdta = vec![0u8; body_len];
                file.read_exact(&mut pdta).ok()?; // already positioned at the body
                let patches = cymbra_sf2_meta::stereo_exclusive_class_patches(&pdta);
                if patches.is_empty() {
                    return None;
                }
                for patch in &patches {
                    pdta.get_mut(patch.offset..patch.offset + 2)?
                        .copy_from_slice(&patch.value.to_le_bytes());
                }
                platform_log::log_line(
                    "cymbra-audio",
                    &format!("{} shared choke class(es) split in {path}", patches.len()),
                );
                return Some((body + 4, pdta));
            }
        }
        // Chunks are word-aligned: an odd size carries a pad byte.
        pos = body + size + (size & 1);
    }
    None
}

/// Reads and parses the SoundFont from `sf2_path`, opens an output device and
/// owns the stream for the process lifetime. Runs on the dedicated audio thread,
/// so the multi-second SoundFont read/parse never blocks the UI.
///
/// After the first stream is up the thread parks on the [`DeviceCommand`]
/// channel: `cpal::Stream` is neither `Send` nor rebuildable from inside its own
/// callback, so every device change is served here, on the thread that owns it.
/// Only a failure to start at all returns (the engine then stays silent).
#[cfg_attr(target_os = "android", allow(dead_code))] // cpal path: unused on Android
fn run_audio_thread(
    sf2_path: String,
    events: SharedEvents,
    device_rx: Receiver<DeviceCommand>,
) -> Result<()> {
    let sound_font = load_sound_font(&sf2_path)?;
    let host = cpal::default_host();

    let mut requested: Option<String> = None;
    let mut stream = open_output(&host, None, &sound_font, events.clone())?;
    stream.play()?;
    platform_log::log_line("cymbra-audio", "output started");

    /// How often, while a standing request is not honored, to check whether its
    /// device came back. The steady path (request honored, or none) does no
    /// enumeration at all.
    const REPLUG_POLL: Duration = Duration::from_secs(2);

    loop {
        let command = match device_rx.recv_timeout(REPLUG_POLL) {
            Ok(command) => command,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // The chosen device may have come back: a fallback keeps the
                // request standing (see below), and this is the only side that
                // can notice the return — the host pushes no device events.
                let Some(name) = requested.clone() else {
                    continue;
                };
                let honored = ACTIVE_OUTPUT
                    .lock()
                    .unwrap()
                    .as_ref()
                    .is_some_and(|active| active.name == name);
                if honored || !describe_outputs(&host).iter().any(|(n, _)| *n == name) {
                    continue;
                }
                platform_log::log_line(
                    "cymbra-audio",
                    &format!("{name:?} is back — reopening on it"),
                );
                DeviceCommand::SetOutput(Some(name))
            }
        };
        // A `Reopen` says the current stream is already dead (its device went
        // away, or the route moved under it). Retiring it *first* both frees the
        // old device and stops a corpse from holding the queue; a user-initiated
        // `SetOutput`, by contrast, must never trade working audio for a device
        // that might not open, so there the replacement is built first.
        let (next_request, stream_is_dead) = match command {
            DeviceCommand::SetOutput(name) => (name, false),
            DeviceCommand::Reopen => (requested.clone(), true),
        };
        let current = if stream_is_dead {
            drop(stream); // retire the dead device before asking for another
            None
        } else {
            Some(stream)
        };
        match open_output(&host, next_request.as_deref(), &sound_font, events.clone()) {
            Ok(next) => {
                requested = next_request;
                // Dropping the old stream stops its callback, so no voice is left
                // sounding on the previous device; the new synthesizer starts
                // empty on the new device's sample rate.
                drop(current);
                stream = next;
                if let Err(e) = stream.play() {
                    platform_log::log_line(
                        "cymbra-audio",
                        &format!("new output would not start: {e}"),
                    );
                }
            }
            Err(e) => {
                platform_log::log_line(
                    "cymbra-audio",
                    &format!("output change refused, keeping current: {e}"),
                );
                match current {
                    // The previous stream is still live: keep it, and keep
                    // reporting it as the active output.
                    Some(previous) => stream = previous,
                    // Nothing left to fall back to. Retry on the host's default
                    // so a lost device does not end the session in silence — but
                    // the request STANDS: the user chose that device, and the
                    // replug watcher above re-adopts it the moment it is back.
                    None => {
                        stream = open_output(&host, None, &sound_font, events.clone())?;
                        stream.play()?;
                    }
                }
            }
        }
    }
    // Every sender is gone (process teardown): keep the stream alive and park.
    let _stream = stream;
    loop {
        thread::park();
    }
}

/// The host's name for `device`, or `None` when it cannot be described.
fn device_name(device: &cpal::Device) -> Option<String> {
    device.description().ok().map(|d| d.name().to_string())
}

/// Enumerates the host's output devices as `(name, kind)` pairs. An unreachable
/// host yields an empty list rather than an error: the caller degrades to "no
/// devices to offer".
fn describe_outputs(host: &cpal::Host) -> Vec<(String, AudioRouteKind)> {
    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };
    devices
        .filter_map(|device| {
            let description = device.description().ok()?;
            Some((
                description.name().to_string(),
                route_kind_of(description.interface_type(), description.device_type()),
            ))
        })
        .collect()
}

/// Resolves `requested` against what the host offers, opens that device and
/// builds a (not yet started) synth stream on it, publishing the device actually
/// opened in [`ACTIVE_OUTPUT`].
#[cfg_attr(target_os = "android", allow(dead_code))] // cpal path: unused on Android
fn open_output(
    host: &cpal::Host,
    requested: Option<&str>,
    sound_font: &Arc<SoundFont>,
    events: SharedEvents,
) -> Result<cpal::Stream> {
    let default_device = host.default_output_device();
    let available: Vec<(cpal::Device, String)> = host
        .output_devices()
        .map(|devices| {
            devices
                .filter_map(|d| device_name(&d).map(|n| (d, n)))
                .collect()
        })
        .unwrap_or_default();
    let names: Vec<String> = available.iter().map(|(_, n)| n.clone()).collect();

    // Follow the system default by opening the host's **default handle**, never
    // the enumerated device that happens to carry the default's name: on Android
    // the default handle is an unspecified device, which is precisely what lets
    // the stream move when the user changes route (a USB-audio piano, a headset).
    // A concrete device pins the stream to one output for good. See
    // [`OutputChoice`].
    let device = match resolve_output_device(requested, &names) {
        OutputChoice::SystemDefault => default_device,
        OutputChoice::Named(name) => available
            .into_iter()
            .find(|(_, n)| *n == name)
            .map(|(d, _)| d)
            .or(default_device),
    }
    .ok_or_else(|| anyhow!("no output device"))?;

    let supported = device.default_output_config()?;
    let sample_format = supported.sample_format();
    let mut config: cpal::StreamConfig = supported.config();

    // Ask for a short buffer (change: add-drum-input-mapping — beta fix): the
    // callback runs once per buffer, so this is the floor under everything the
    // player hears — including the engine's echo of their own strokes, which is
    // otherwise the only part of the input path we do not control.
    //
    // A **request**, never a demand: some devices refuse a fixed size outright
    // and fail to open at all, so a refusal falls straight back to the host's
    // default rather than leaving the app silent. The log line below reports
    // what was actually opened, which is the only number worth trusting.
    let preferred = preferred_buffer_frames(
        match supported.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => Some((*min, *max)),
            cpal::SupportedBufferSize::Unknown => None,
        },
        config.sample_rate,
    );
    if let Some(frames) = preferred {
        config.buffer_size = cpal::BufferSize::Fixed(frames);
    }

    let build = |config: &cpal::StreamConfig| match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, config, sound_font, events.clone()),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, config, sound_font, events.clone()),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, config, sound_font, events.clone()),
        other => Err(anyhow!("unsupported sample format: {other:?}")),
    };
    let stream = match build(&config) {
        Ok(stream) => stream,
        Err(e) if preferred.is_some() => {
            platform_log::log_line(
                "cymbra-audio",
                &format!("device refused a {preferred:?}-frame buffer ({e}); taking its default"),
            );
            config.buffer_size = cpal::BufferSize::Default;
            build(&config)?
        }
        Err(e) => return Err(e),
    };

    let active = device.description().ok().map(|d| AudioOutputInfo {
        name: d.name().to_string(),
        kind: route_kind_of(d.interface_type(), d.device_type()),
    });
    STREAM_SAMPLE_RATE.store(config.sample_rate, Ordering::Relaxed);
    platform_log::log_line(
        "cymbra-audio",
        &format!(
            "opened {:?} ({:?}) — {} Hz, {} ch, {sample_format:?}, buffer {:?}",
            active.as_ref().map(|a| a.name.as_str()).unwrap_or("?"),
            active.as_ref().map(|a| a.kind),
            config.sample_rate,
            config.channels,
            config.buffer_size,
        ),
    );
    *ACTIVE_OUTPUT.lock().unwrap() = active;
    Ok(stream)
}

/// Builds a `cpal` output stream whose callback drains control events into the
/// synthesizer and renders the next block of audio. The stream is returned
/// **stopped** so the caller can retire the previous one first.
#[cfg_attr(target_os = "android", allow(dead_code))] // cpal path: unused on Android
fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sound_font: &Arc<SoundFont>,
    events: SharedEvents,
) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = (config.channels as usize).max(1);
    let mut renderer = Renderer::new(sound_font, config.sample_rate as i32)?;

    let stream = device.build_output_stream(
        *config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            // The queue outlives any one stream (a device change rebuilds around
            // it), so it is reached through a mutex — uncontended, because only
            // this callback locks it.
            if let Ok(queue) = events.try_lock() {
                renderer.drain(&queue);
            }

            let frames = output.len() / channels;
            RENDERED_FRAMES.fetch_add(frames as u64, Ordering::Relaxed);
            let (l, r) = renderer.render(frames);

            // Interleave the stereo render into the device's frame layout. Mono
            // devices get the left channel; >2 channels mirror L/R.
            for (frame, out) in output.chunks_mut(channels).enumerate() {
                for (ch, sample) in out.iter_mut().enumerate() {
                    let v = if ch % 2 == 1 { r[frame] } else { l[frame] };
                    *sample = T::from_sample(v);
                }
            }
        },
        |e| {
            platform_log::log_line(
                "cymbra-audio",
                &format!("stream error ({:?}): {e}", e.kind()),
            );
            // The device in use vanished (unplugged, or its config was
            // invalidated): ask the audio thread to re-resolve, which falls back
            // to the system default instead of leaving the app silent. Every
            // other error is transient and the stream keeps running.
            let lost = matches!(
                e.kind(),
                cpal::ErrorKind::DeviceNotAvailable | cpal::ErrorKind::StreamInvalidated
            );
            if let Some(tx) = DEVICE_TX.lock().unwrap().as_ref().filter(|_| lost) {
                let _ = tx.send(DeviceCommand::Reopen);
            }
        },
        None,
    )?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustysynth::{Synthesizer, SynthesizerSettings};

    /// The kit font the app ships, from the crate root.
    fn kit_font() -> String {
        format!(
            "{}/../assets/soundfonts/FluidR3Drums-bank128.sf2",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    /// Peak level of each channel over one second of key `key` on the drum
    /// channel — the audibility question a player asks, answered in samples.
    fn peaks(font: &Arc<SoundFont>, key: i32) -> (f32, f32) {
        let mut synth = Synthesizer::new(font, &SynthesizerSettings::new(44_100)).expect("synth");
        synth.note_on(9, key, 100);
        let (mut left, mut right) = (vec![0f32; 44_100], vec![0f32; 44_100]);
        synth.render(&mut left, &mut right);
        let peak = |v: &[f32]| v.iter().fold(0f32, |a, b| a.max(b.abs()));
        (peak(&left), peak(&right))
    }

    /// The beta report this fixes: the hi-hat lit its pad and made no sound.
    /// Its two stereo halves shared a choke class, so `rustysynth` gave them one
    /// voice and the hard-right half won — inaudible on a mono output, and half
    /// the level of everything else on a stereo one.
    #[test]
    fn the_hi_hat_sounds_on_both_channels_like_every_other_piece() {
        let font = load_sound_font(&kit_font()).expect("kit font loads");
        for key in [42, 44, 46] {
            let (left, right) = peaks(&font, key);
            assert!(
                left > 0.0,
                "GM {key} must reach the left channel too (got {left} / {right})"
            );
        }
        // The control: pieces that never shared a class are untouched.
        let (left, right) = peaks(&font, 38);
        assert!(left > 0.0 && right > 0.0);
    }

    #[test]
    fn the_shipped_kit_font_declares_shared_choke_groups_to_split() {
        // Guards the test above against becoming vacuous if the asset is ever
        // repacked without exclusive classes.
        let (at, pdta) = split_choke_groups(&kit_font()).expect("the kit font is patched");
        assert!(at > 0 && !pdta.is_empty());
    }

    #[test]
    fn a_file_that_is_not_a_font_is_simply_not_patched() {
        assert!(split_choke_groups("/nonexistent/font.sf2").is_none());
    }
}
