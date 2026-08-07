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

/// Drops a raw message that is byte-identical to the one just before it and
/// arrived less than [`DuplicateGuard::WINDOW_US`] later.
///
/// A physical keyboard cannot emit the same status/pitch/velocity triple twice
/// inside a couple of milliseconds — a second NoteOn for a pitch requires a
/// NoteOff in between, which differs in bytes. So this only ever fires on a
/// driver re-delivering a message it already gave us.
///
/// It exists because midir's Android (AMidi) backend used to do exactly that:
/// `AMidiOutputPort_receive` returns 0 when no message is pending and leaves
/// its out-params untouched, and the reader loop only checked for the negative
/// error code. After the very first key press the loop re-delivered that press
/// forever, at full speed, flooding the Flutter sink until the UI thread
/// starved and Android raised an ANR. We ship a patched midir (see the
/// `[patch.crates-io]` entry in the workspace manifest), but the guard stays as
/// a cheap backstop: any backend that re-delivers is capped at one event.
pub(crate) struct DuplicateGuard {
    last: Vec<u8>,
    last_us: u64,
}

impl DuplicateGuard {
    /// Messages closer together than this *and* byte-identical are duplicates.
    pub(crate) const WINDOW_US: u64 = 2_000;

    pub(crate) fn new() -> Self {
        Self {
            last: Vec::new(),
            last_us: 0,
        }
    }

    /// True if `message` should be forwarded to Flutter.
    ///
    /// The window slides on every rejection, so a sustained re-delivery loop is
    /// suppressed entirely instead of leaking one event every `WINDOW_US`.
    pub(crate) fn accept(&mut self, message: &[u8], now_us: u64) -> bool {
        let duplicate =
            self.last == message && now_us.saturating_sub(self.last_us) < Self::WINDOW_US;
        self.last_us = now_us;
        if duplicate {
            return false;
        }
        self.last.clear();
        self.last.extend_from_slice(message);
        true
    }
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
        assert!(matches!(ev.kind, MidiEventKind::NoteOn));
        assert_eq!(ev.pitch, 60);
        assert_eq!(ev.velocity, 100);
        assert_eq!(ev.timestamp_ms, 42);
    }

    #[test]
    fn note_on_ignores_running_status_channel_bits() {
        // 0x95 = NoteOn on channel 6 → still a NoteOn (status high nibble 0x90).
        let ev = parse_midi(&[0x95, 64, 80], 0).expect("note on ch6");
        assert!(matches!(ev.kind, MidiEventKind::NoteOn));
        assert_eq!(ev.pitch, 64);
    }

    #[test]
    fn explicit_note_off_is_parsed() {
        let ev = parse_midi(&[0x80, 60, 64], 7).expect("note off");
        assert!(matches!(ev.kind, MidiEventKind::NoteOff));
        assert_eq!(ev.pitch, 60);
        // NoteOff velocity is normalized to 0.
        assert_eq!(ev.velocity, 0);
    }

    #[test]
    fn note_on_with_zero_velocity_is_note_off() {
        let ev = parse_midi(&[0x90, 60, 0], 0).expect("note off via vel 0");
        assert!(matches!(ev.kind, MidiEventKind::NoteOff));
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
    fn guard_accepts_the_first_message() {
        let mut guard = DuplicateGuard::new();
        assert!(guard.accept(&[0x90, 60, 100], 0));
    }

    #[test]
    fn guard_drops_an_immediate_byte_identical_repeat() {
        let mut guard = DuplicateGuard::new();
        assert!(guard.accept(&[0x90, 60, 100], 1_000));
        assert!(!guard.accept(&[0x90, 60, 100], 1_000));
        assert!(!guard.accept(&[0x90, 60, 100], 1_001));
    }

    #[test]
    fn guard_keeps_suppressing_a_sustained_redelivery_loop() {
        // The AMidi defect re-delivers at full speed: every poll is within the
        // window of the previous one, so the window slides and nothing leaks
        // through after the first event.
        let mut guard = DuplicateGuard::new();
        assert!(guard.accept(&[0x90, 60, 100], 0));
        let leaked = (1..10_000)
            .filter(|us| guard.accept(&[0x90, 60, 100], *us))
            .count();
        assert_eq!(leaked, 0);
    }

    #[test]
    fn guard_accepts_an_identical_message_after_the_window() {
        let mut guard = DuplicateGuard::new();
        assert!(guard.accept(&[0x90, 60, 100], 0));
        assert!(guard.accept(&[0x90, 60, 100], DuplicateGuard::WINDOW_US));
    }

    #[test]
    fn guard_never_blocks_a_different_message() {
        let mut guard = DuplicateGuard::new();
        assert!(guard.accept(&[0x90, 60, 100], 0));
        // Same pitch, different velocity — a genuine distinct press.
        assert!(guard.accept(&[0x90, 60, 101], 1));
        // The matching NoteOff, immediately after.
        assert!(guard.accept(&[0x80, 60, 0], 2));
        // A chord: several pitches within the same window.
        assert!(guard.accept(&[0x90, 64, 100], 3));
        assert!(guard.accept(&[0x90, 67, 100], 4));
    }

    #[test]
    fn guard_lets_a_fast_trill_through() {
        // Alternating NoteOn/NoteOff on one pitch, 500 µs apart — far inside
        // the window, but never byte-identical back to back.
        let mut guard = DuplicateGuard::new();
        for i in 0..20u64 {
            let msg: [u8; 3] = if i % 2 == 0 {
                [0x90, 60, 100]
            } else {
                [0x80, 60, 0]
            };
            assert!(guard.accept(&msg, i * 500), "rejected message {i}");
        }
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
