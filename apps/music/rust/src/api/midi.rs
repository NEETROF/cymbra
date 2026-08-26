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

//! Raw USB-MIDI port listening via `midir` with real-time streaming to Flutter.
//!
//! A watcher thread automatically (re)connects to the first MIDI port as soon as
//! it appears (hot-plug), detects unplugging, and keeps the connected port name
//! up to date for the on-screen indicator.
//!
//! midir backends: CoreMIDI (macOS/iOS), ALSA (Linux), WinMM (Windows),
//! AMidi via NDK (Android — the `JavaVM` is provided by `JNI_OnLoad`, see lib.rs).

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use flutter_rust_bridge::frb;
use midir::{Ignore, MidiInput, MidiInputConnection};

use super::audio_core::echo_event;
use super::midi_core::{
    DuplicateGuard, MidiStreamParser, is_virtual_port, parse_midi, resolves_to_connected,
    sort_ports_virtual_last, stable_port_key,
};
use crate::frb_generated::StreamSink;

/// MIDI event type forwarded to Flutter.
pub enum MidiEventKind {
    NoteOn,
    NoteOff,
}

/// What the **engine itself** sounds for a live MIDI event, straight from the
/// MIDI callback (change: add-drum-input-mapping — beta fix "strong latency
/// between the hit and the sound").
///
/// A live note used to be sounded by Dart: the engine forwarded the event over
/// the bridge, the notifier handled it, and only then called back into the
/// engine to play it. Everything in that round trip rides the UI isolate's
/// event loop, so the delay a player hears is whatever the app happens to be
/// doing that frame — on a kit, where the stick has already left the head, that
/// is the difference between an instrument and a lag.
///
/// The app stays in charge of the *policy* — whether to sound at all
/// (instrument-sounds-itself), and on which channel — by pushing the mode here
/// ([`set_midi_echo`]); the engine only executes it, and the app then leaves
/// the sounding of MIDI notes alone so nothing is played twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiEcho {
    /// The engine sounds nothing: the app plays live notes itself, or the
    /// instrument already sounds them.
    Off,
    /// Sound on the melodic channel, with the matching release — a keyboard.
    Melodic,
    /// Sound on the drum channel as a one-shot, releases dropped, exactly as a
    /// percussion stroke is played (a kit's note-off arrives milliseconds after
    /// its attack and would cut the voice).
    Drum,
}

/// A normalized MIDI event, ready to be consumed by Flutter.
pub struct MidiEvent {
    pub kind: MidiEventKind,
    /// MIDI note number (0-127).
    pub pitch: u8,
    /// Velocity (0-127). 0 for a NoteOff.
    pub velocity: u8,
    /// Timestamp since the stream was opened, in milliseconds.
    pub timestamp_ms: u64,
}

// Active connection(s) kept alive (midir closes the port on drop).
static CONNECTIONS: Mutex<Vec<MidiInputConnection<()>>> = Mutex::new(Vec::new());
// Name of the currently connected port (None if none).
static CONNECTED_PORT: Mutex<Option<String>> = Mutex::new(None);
// Port chosen by the user (None = auto: first non-virtual port).
static SELECTED_PORT: Mutex<Option<String>> = Mutex::new(None);
// Last logged port list (so we only log changes).
static LAST_LOGGED_PORTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The ports the watcher last saw, served to Flutter instead of re-enumerating.
static KNOWN_PORTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
// Prevents launching multiple watcher threads.
static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);
// Ends the watcher's back-off nap early, so an explicit port change reconnects
// now instead of after the current (up to 5 s) sleep.
static WATCHER_WAKE: AtomicBool = AtomicBool::new(false);
// The current Flutter sink. Replaced on every `midi_event_stream` call so that
// re-subscription (screen re-entry, hot-plug after launch) routes events to the
// live listener instead of the first, now-dead, sink. Read by the input
// callback on each MIDI message and by the watcher thread.
static SINK: Mutex<Option<Arc<StreamSink<MidiEvent>>>> = Mutex::new(None);
/// The engine's live-echo mode ([`MidiEcho`]), as a code the input callback can
/// read without locking. Written only by [`set_midi_echo`].
static ECHO: AtomicU8 = AtomicU8::new(ECHO_OFF);
const ECHO_OFF: u8 = 0;
const ECHO_MELODIC: u8 = 1;
const ECHO_DRUM: u8 = 2;

/// Chooses what the engine sounds for live MIDI events from now on, and returns
/// immediately (see [`MidiEcho`]). Pushed by the app whenever its own answer
/// changes — the loaded score's family, the kit font becoming ready, the
/// instrument-sounds-itself setting, leaving the player — and `Off` is always a
/// safe value: it simply leaves the sounding to the app.
#[frb(sync)]
pub fn set_midi_echo(mode: MidiEcho) {
    ECHO.store(
        match mode {
            MidiEcho::Off => ECHO_OFF,
            MidiEcho::Melodic => ECHO_MELODIC,
            MidiEcho::Drum => ECHO_DRUM,
        },
        Ordering::Relaxed,
    );
}

/// The mode the input callback is running under.
pub(crate) fn midi_echo() -> MidiEcho {
    match ECHO.load(Ordering::Relaxed) {
        ECHO_MELODIC => MidiEcho::Melodic,
        ECHO_DRUM => MidiEcho::Drum,
        _ => MidiEcho::Off,
    }
}

/// Lists the names of available MIDI input ports (UI selection).
/// Virtual ports ("Midi Through", rtpmidi…) are placed last.
#[frb(sync)]
pub fn list_midi_ports() -> Result<Vec<String>> {
    // Served from the watcher's cache, never by re-enumerating.
    //
    // Flutter polls this once a second for the connection indicator. Asking the
    // platform every time means recreating a `MidiInput` and querying
    // `MidiManager` twice a second forever — which on a composite USB instrument
    // (MIDI and audio on one device) is enough to knock the whole device off the
    // bus, audio included. The watcher already refreshes this list; reading its
    // copy costs nothing.
    let mut names = KNOWN_PORTS.lock().unwrap().clone();
    sort_ports_virtual_last(&mut names);
    Ok(names)
}

/// Name of the currently connected MIDI port, or `None` if no device.
/// Polled periodically by Flutter for the connection indicator.
#[frb(sync)]
pub fn connected_port() -> Option<String> {
    CONNECTED_PORT.lock().ok().and_then(|g| g.clone())
}

/// Chooses the MIDI device to listen to (by name). `None` = auto mode
/// (first non-virtual port). Forces an immediate reconnection to the new port.
#[frb(sync)]
pub fn set_midi_port(name: Option<String>) {
    let connected = CONNECTED_PORT.lock().unwrap().clone();
    // Already listening to what is being asked for? Then do **nothing**.
    //
    // Tearing the connection down and letting the watcher rebuild it is not free
    // on Android: the platform's own USB MIDI driver counts a port that our
    // dropped connection does not actually release, so every needless churn
    // leaks one. After one or two, that driver dies ("Unexpected response") and
    // takes the **whole composite USB device** down with it — the piano's audio
    // interfaces included, which is why the sound vanished a few seconds after
    // touching this setting. Choosing the device already in use, or "auto" when
    // auto already resolves to it, must therefore cost nothing.
    if resolves_to_connected(name.as_deref(), connected.as_deref()) {
        *SELECTED_PORT.lock().unwrap() = name;
        super::platform_log::log_line(
            "cymbra-midi",
            "selection already connected — keeping the port open",
        );
        return;
    }

    super::platform_log::log_line(
        "cymbra-midi",
        &format!(
            "select port {name:?} (open connections: {})",
            open_connections()
        ),
    );
    *SELECTED_PORT.lock().unwrap() = name;
    // Release the current connection and wake the watcher, which reconnects to
    // the desired port on its next pass — now, not after its back-off nap.
    CONNECTIONS.lock().unwrap().clear();
    *CONNECTED_PORT.lock().unwrap() = None;
    WATCHER_WAKE.store(true, Ordering::SeqCst);
}

/// How many MIDI input connections the engine is holding open.
///
/// The number that matters on Android: each open port is counted by the
/// platform's own USB MIDI driver, and once too many accumulate it fails with
/// "Cannot queue request" and drops the **whole composite USB device** — audio
/// interfaces included. So this must never climb.
fn open_connections() -> usize {
    CONNECTIONS.lock().unwrap().len()
}

/// Starts MIDI watching and streams NoteOn/NoteOff into `sink`.
///
/// The thread connects to the first available port, reconnects on hot-plug,
/// and releases the connection on unplug.
pub fn midi_event_stream(sink: StreamSink<MidiEvent>) -> Result<()> {
    // Always register the latest sink. Flutter re-subscribes on every screen
    // (re-)entry; the watcher and input callback read this global, so events
    // follow the live listener instead of being stuck on the first sink.
    *SINK.lock().unwrap() = Some(Arc::new(sink));

    // A single watcher thread for the entire process lifetime.
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let start = Instant::now();

    // Prime the served port list before the watcher's first pass: both Dart
    // consumers list the ports synchronously right after subscribing, and an
    // empty cache there reads as "no device" until something refreshes it. One
    // enumeration, once per process — not the twice-a-second polling this cache
    // exists to end.
    *KNOWN_PORTS.lock().unwrap() = current_port_names();

    super::platform_log::log_line("cymbra-midi", "watcher started");
    thread::spawn(move || {
        loop {
            let ports = current_port_names();
            *KNOWN_PORTS.lock().unwrap() = ports.clone();

            // Log only when the port list changes.
            {
                let mut last = LAST_LOGGED_PORTS.lock().unwrap();
                if *last != ports {
                    super::platform_log::log_line(
                        "cymbra-midi",
                        &format!("detected ports = {ports:?}"),
                    );
                    *last = ports.clone();
                }
            }

            let connected = CONNECTED_PORT.lock().unwrap().clone();

            let selected = SELECTED_PORT.lock().unwrap().clone();

            match connected {
                // Connected: check that the port is still there.
                Some(name) if !ports.contains(&name) => {
                    CONNECTIONS.lock().unwrap().clear();
                    *CONNECTED_PORT.lock().unwrap() = None;
                    super::platform_log::log_line("cymbra-midi", &format!("Unplugged: {name}"));
                }
                // Connected to a port an *explicit* selection no longer resolves
                // to: an in-flight try_connect can land after `set_midi_port`
                // cleared the connection, re-installing the old port. Release it
                // here so the next pass honors the selection instead of keeping
                // the stale connection until unplug. Auto (None) is exempt — it
                // accepts any real port, and releasing a virtual-only connection
                // under auto would churn connect/release forever.
                Some(name)
                    if selected.is_some()
                        && !resolves_to_connected(selected.as_deref(), Some(name.as_str())) =>
                {
                    CONNECTIONS.lock().unwrap().clear();
                    *CONNECTED_PORT.lock().unwrap() = None;
                    super::platform_log::log_line(
                        "cymbra-midi",
                        &format!("releasing {name}: selection changed to {selected:?}"),
                    );
                }
                // Not connected: try to connect to the first port.
                None => {
                    if let Err(e) = try_connect(start) {
                        super::platform_log::log_line(
                            "cymbra-midi",
                            &format!("Connection failed: {e}"),
                        );
                    }
                }
                _ => {}
            }

            // Hunt quickly for a device while there is none — hot-plug should feel
            // immediate — then back right off once connected. Enumerating a
            // composite USB instrument twice a second for the whole session is
            // what the working bench never does, and what appears to bring the
            // device down.
            //
            // The back-off sleep is sliced so an explicit port change does not
            // wait out a 5 s nap before reconnecting: `set_midi_port` raises
            // [`WATCHER_WAKE`] and the next slice ends the pass early. Checking a
            // flag every 250 ms enumerates nothing, so the churn budget holds.
            let idle = CONNECTED_PORT.lock().unwrap().is_none();
            let nap = if idle {
                Duration::from_millis(700)
            } else {
                Duration::from_secs(5)
            };
            let slice = Duration::from_millis(250);
            let mut slept = Duration::ZERO;
            while slept < nap && !WATCHER_WAKE.swap(false, Ordering::SeqCst) {
                let step = slice.min(nap - slept);
                thread::sleep(step);
                slept += step;
            }
        }
    });

    Ok(())
}

/// Names of the MIDI input ports currently present.
fn current_port_names() -> Vec<String> {
    match MidiInput::new("cymbra-poll") {
        Ok(midi_in) => midi_in
            .ports()
            .iter()
            .map(|p| {
                midi_in
                    .port_name(p)
                    .unwrap_or_else(|_| "<unknown>".to_string())
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Tries to connect to the first available port and wires up the callback.
/// The callback reads the global [`SINK`] on each message, so events always
/// reach the live listener even after the Flutter side re-subscribes.
fn try_connect(start: Instant) -> Result<()> {
    let mut midi_in = MidiInput::new("cymbra-input")?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    if ports.is_empty() {
        return Ok(()); // no port: we'll retry on the next pass
    }

    let desired = SELECTED_PORT.lock().unwrap().clone();
    let port = match &desired {
        // Explicitly chosen port: we find it by name.
        // Matched on the stable key, so a device that came back under a new
        // instance number is still recognised as the one that was chosen.
        Some(name) => ports.iter().find(|p| {
            midi_in
                .port_name(p)
                .ok()
                .is_some_and(|n| stable_port_key(&n) == stable_port_key(name))
        }),
        // Auto mode: we ignore virtual "Through" ports (ALSA Midi Through,
        // etc.) and take the first real device; otherwise the first port.
        None => ports
            .iter()
            .find(|p| !is_virtual_port(&midi_in.port_name(p).unwrap_or_default()))
            .or_else(|| ports.first()),
    };
    let Some(port) = port else {
        return Ok(()); // the desired port is not (yet) there
    };
    let name = midi_in.port_name(port).unwrap_or_default();

    // Per-connection state. The guard stops a backend that re-delivers the same
    // payload from flooding the Flutter sink; the parser splits that payload
    // into messages, because Android hands us a byte stream rather than one
    // message per callback (see `MidiStreamParser`).
    let mut guard = DuplicateGuard::new();
    let mut parser = MidiStreamParser::new();
    let mut messages: Vec<[u8; 3]> = Vec::new();

    let conn = midi_in
        .connect(
            port,
            "cymbra-read",
            move |_timestamp_us, payload, _| {
                let elapsed = start.elapsed();
                if !guard.accept(payload, elapsed.as_micros() as u64) {
                    return;
                }
                messages.clear();
                parser.push(payload, &mut messages);
                let timestamp_ms = elapsed.as_millis() as u64;
                for message in &messages {
                    let Some(event) = parse_midi(message, timestamp_ms) else {
                        continue;
                    };
                    // Sound it HERE, before the event goes anywhere near the
                    // bridge: this is the whole point of the echo. The queue
                    // hand-off is lock-free-ish and the audio callback picks it
                    // up on its next block, so what the player hears is the
                    // device plus the output buffer — not the UI isolate's
                    // schedule.
                    if let Some(sound) = echo_event(midi_echo(), &event) {
                        super::audio::send(sound);
                    }
                    // Read the current sink each message: it may have been
                    // swapped by a re-subscription since this port was opened.
                    if let Some(sink) = SINK.lock().unwrap().as_ref() {
                        let _ = sink.add(event);
                    }
                }
            },
            (),
        )
        .map_err(|e| anyhow::anyhow!("could not connect to MIDI port: {e}"))?;

    CONNECTIONS.lock().unwrap().push(conn);
    super::platform_log::log_line(
        "cymbra-midi",
        &format!("connected (open connections: {})", open_connections()),
    );
    *CONNECTED_PORT.lock().unwrap() = Some(name.clone());
    super::platform_log::log_line("cymbra-midi", &format!("Connected: {name}"));
    Ok(())
}
