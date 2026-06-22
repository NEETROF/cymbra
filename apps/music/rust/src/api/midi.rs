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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use flutter_rust_bridge::frb;
use midir::{Ignore, MidiInput, MidiInputConnection};

use super::midi_core::{is_virtual_port, parse_midi, sort_ports_virtual_last};
use crate::frb_generated::StreamSink;

/// MIDI event type forwarded to Flutter.
pub enum MidiEventKind {
    NoteOn,
    NoteOff,
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
// Prevents launching multiple watcher threads.
static WATCHER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Lists the names of available MIDI input ports (UI selection).
/// Virtual ports ("Midi Through", rtpmidi…) are placed last.
#[frb(sync)]
pub fn list_midi_ports() -> Result<Vec<String>> {
    let mut names = current_port_names();
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
    *SELECTED_PORT.lock().unwrap() = name;
    // Release the current connection: the watcher thread will reconnect
    // to the desired port on the next pass (~700 ms).
    CONNECTIONS.lock().unwrap().clear();
    *CONNECTED_PORT.lock().unwrap() = None;
}

/// Starts MIDI watching and streams NoteOn/NoteOff into `sink`.
///
/// The thread connects to the first available port, reconnects on hot-plug,
/// and releases the connection on unplug.
pub fn midi_event_stream(sink: StreamSink<MidiEvent>) -> Result<()> {
    // A single watcher thread for the entire process lifetime.
    if WATCHER_RUNNING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let sink = Arc::new(sink);
    let start = Instant::now();

    eprintln!("[cymbra-midi] watcher started");
    thread::spawn(move || {
        loop {
            let ports = current_port_names();

            // Log only when the port list changes.
            {
                let mut last = LAST_LOGGED_PORTS.lock().unwrap();
                if *last != ports {
                    eprintln!("[cymbra-midi] detected ports = {ports:?}");
                    *last = ports.clone();
                }
            }

            let connected = CONNECTED_PORT.lock().unwrap().clone();

            match connected {
                // Connected: check that the port is still there.
                Some(name) if !ports.contains(&name) => {
                    CONNECTIONS.lock().unwrap().clear();
                    *CONNECTED_PORT.lock().unwrap() = None;
                    eprintln!("[cymbra-midi] Unplugged: {name}");
                }
                // Not connected: try to connect to the first port.
                None => {
                    if let Err(e) = try_connect(&sink, start) {
                        eprintln!("[cymbra-midi] Connection failed: {e}");
                    }
                }
                _ => {}
            }

            thread::sleep(Duration::from_millis(700));
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
fn try_connect(sink: &Arc<StreamSink<MidiEvent>>, start: Instant) -> Result<()> {
    let mut midi_in = MidiInput::new("cymbra-input")?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    if ports.is_empty() {
        return Ok(()); // no port: we'll retry on the next pass
    }

    let desired = SELECTED_PORT.lock().unwrap().clone();
    let port = match &desired {
        // Explicitly chosen port: we find it by name.
        Some(name) => ports
            .iter()
            .find(|p| midi_in.port_name(p).as_deref().ok() == Some(name.as_str())),
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

    let sink = Arc::clone(sink);
    let conn = midi_in
        .connect(
            port,
            "cymbra-read",
            move |_timestamp_us, message, _| {
                if let Some(event) = parse_midi(message, start.elapsed().as_millis() as u64) {
                    let _ = sink.add(event);
                }
            },
            (),
        )
        .map_err(|e| anyhow::anyhow!("could not connect to MIDI port: {e}"))?;

    CONNECTIONS.lock().unwrap().push(conn);
    *CONNECTED_PORT.lock().unwrap() = Some(name.clone());
    eprintln!("[cymbra-midi] Connected: {name}");
    Ok(())
}
