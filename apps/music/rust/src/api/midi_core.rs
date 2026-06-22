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

//! Pure, host-testable MIDI logic — no hardware, threads, or FFI.
//!
//! Split out of [`super::midi`] so it can be unit-tested (and counted by
//! `cargo llvm-cov`) on CI hosts that have no MIDI devices. The thread/IO glue
//! in `midi.rs` stays out of the coverage gate; everything genuinely testable
//! lives here.

use super::midi::{MidiEvent, MidiEventKind};

/// True if the port is a virtual/loopback MIDI port (e.g. ALSA "Midi Through"),
/// which we avoid by default in auto mode.
pub(crate) fn is_virtual_port(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("through") || n.contains("rtpmidi") || n.contains("network")
}

/// Stable-sorts port names so real devices come first and virtual ports last.
pub(crate) fn sort_ports_virtual_last(names: &mut [String]) {
    // Stable sort: real devices (false) before virtual ones (true).
    names.sort_by_key(|n| is_virtual_port(n));
}

/// Parses a raw MIDI message into a [`MidiEvent`].
///
/// - `0x90` (NoteOn) with velocity > 0 → NoteOn
/// - `0x80` (NoteOff), or `0x90` with velocity 0 → NoteOff
/// - anything else (too short, CC, program change, …) → `None`
pub(crate) fn parse_midi(message: &[u8], timestamp_ms: u64) -> Option<MidiEvent> {
    if message.len() < 3 {
        return None;
    }
    let status = message[0] & 0xF0;
    let pitch = message[1];
    let velocity = message[2];

    match status {
        0x90 if velocity > 0 => Some(MidiEvent {
            kind: MidiEventKind::NoteOn,
            pitch,
            velocity,
            timestamp_ms,
        }),
        0x80 | 0x90 => Some(MidiEvent {
            kind: MidiEventKind::NoteOff,
            pitch,
            velocity: 0,
            timestamp_ms,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_is_parsed() {
        let ev = parse_midi(&[0x90, 60, 100], 42).expect("note on");
        assert_eq!(ev.kind, MidiEventKind::NoteOn);
        assert_eq!(ev.pitch, 60);
        assert_eq!(ev.velocity, 100);
        assert_eq!(ev.timestamp_ms, 42);
    }

    #[test]
    fn note_on_ignores_running_status_channel_bits() {
        // 0x95 = NoteOn on channel 6 → still a NoteOn (status high nibble 0x90).
        let ev = parse_midi(&[0x95, 64, 80], 0).expect("note on ch6");
        assert_eq!(ev.kind, MidiEventKind::NoteOn);
        assert_eq!(ev.pitch, 64);
    }

    #[test]
    fn explicit_note_off_is_parsed() {
        let ev = parse_midi(&[0x80, 60, 64], 7).expect("note off");
        assert_eq!(ev.kind, MidiEventKind::NoteOff);
        assert_eq!(ev.pitch, 60);
        // NoteOff velocity is normalized to 0.
        assert_eq!(ev.velocity, 0);
    }

    #[test]
    fn note_on_with_zero_velocity_is_note_off() {
        let ev = parse_midi(&[0x90, 60, 0], 0).expect("note off via vel 0");
        assert_eq!(ev.kind, MidiEventKind::NoteOff);
        assert_eq!(ev.velocity, 0);
    }

    #[test]
    fn too_short_message_is_ignored() {
        assert!(parse_midi(&[0x90, 60], 0).is_none());
        assert!(parse_midi(&[], 0).is_none());
    }

    #[test]
    fn control_change_is_ignored() {
        // 0xB0 = Control Change → not a note event.
        assert!(parse_midi(&[0xB0, 7, 127], 0).is_none());
    }

    #[test]
    fn virtual_ports_are_detected_case_insensitively() {
        assert!(is_virtual_port("Midi Through Port-0"));
        assert!(is_virtual_port("RtpMidi Session"));
        assert!(is_virtual_port("Network Session 1"));
        assert!(!is_virtual_port("Roland Digital Piano"));
        assert!(!is_virtual_port("USB MIDI Device"));
    }

    #[test]
    fn sort_puts_real_devices_first_and_is_stable() {
        let mut ports = vec![
            "Midi Through".to_string(),
            "Piano".to_string(),
            "Network Session".to_string(),
            "Keyboard".to_string(),
        ];
        sort_ports_virtual_last(&mut ports);
        assert_eq!(
            ports,
            vec!["Piano", "Keyboard", "Midi Through", "Network Session"]
        );
    }
}
