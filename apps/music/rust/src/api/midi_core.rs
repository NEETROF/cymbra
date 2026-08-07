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

/// Splits a raw MIDI byte stream into complete channel-voice messages.
///
/// Most midir backends hand us exactly one complete message per callback
/// (CoreMIDI walks the packet and calls back per message, ALSA and WinMM are
/// message-oriented). **Android is not**: its AMidi backend forwards whatever
/// the device wrote, and Android MIDI is a byte-stream API — a single USB bulk
/// transfer carries up to 16 MIDI events, which `UsbMidiDevice` decodes and
/// pushes in one go. Reading only the first three bytes of that buffer silently
/// dropped every event behind it, so a NoteOff sharing a transfer with another
/// event never arrived and its key stayed lit on the virtual piano.
///
/// So we treat every backend's payload as a stream. Feeding it a single
/// complete message (every non-Android backend) is just the degenerate case.
/// The parser handles what a real MIDI stream can contain:
///
/// - several messages back to back in one buffer;
/// - running status (the status byte omitted on repeats);
/// - System Real-Time bytes (`0xF8`–`0xFF`), which may appear *inside* another
///   message and must not disturb it;
/// - System Common (`0xF1`–`0xF7`), which cancels running status;
/// - SysEx, skipped;
/// - a message split across two buffers.
pub(crate) struct MidiStreamParser {
    /// Current running status, `None` after a System Common message.
    status: Option<u8>,
    data: [u8; 2],
    /// Data bytes collected so far for the message in progress.
    have: usize,
    in_sysex: bool,
}

impl MidiStreamParser {
    pub(crate) fn new() -> Self {
        Self {
            status: None,
            data: [0; 2],
            have: 0,
            in_sysex: false,
        }
    }

    /// Feeds `bytes`, appending every message completed by them to `out` as a
    /// normalized 3-byte `[status, data1, data2]` (single-data-byte messages
    /// such as Program Change are padded with 0 — [`parse_midi`] ignores them).
    pub(crate) fn push(&mut self, bytes: &[u8], out: &mut Vec<[u8; 3]>) {
        for &b in bytes {
            // System Real-Time: valid anywhere, even between the data bytes of
            // another message. Never touches the message in progress.
            if b >= 0xF8 {
                continue;
            }

            if self.in_sysex {
                if b & 0x80 == 0 {
                    continue; // payload
                }
                // 0xF7 ends it cleanly; any other status aborts it.
                self.in_sysex = false;
                if b == 0xF7 {
                    continue;
                }
            }

            if b & 0x80 != 0 {
                self.have = 0;
                match b {
                    0xF0 => {
                        self.in_sysex = true;
                        self.status = None;
                    }
                    // System Common: cancels running status. Its own data bytes
                    // are then dropped below as orphans, which is what we want.
                    0xF1..=0xF7 => self.status = None,
                    _ => self.status = Some(b),
                }
                continue;
            }

            // Data byte: only meaningful under a live (running) status.
            let Some(status) = self.status else {
                continue;
            };
            self.data[self.have] = b;
            self.have += 1;
            // Program Change (0xC_) and Channel Pressure (0xD_) carry one data
            // byte; every other channel message carries two.
            let needed = if (0xC0..0xE0).contains(&status) { 1 } else { 2 };
            if self.have == needed {
                out.push([
                    status,
                    self.data[0],
                    if needed == 2 { self.data[1] } else { 0 },
                ]);
                // Running status persists: the next data byte starts a new
                // message with the same status.
                self.have = 0;
            }
        }
    }
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

    /// Collects everything `bytes` completes, in order.
    fn parse_stream(parser: &mut MidiStreamParser, bytes: &[u8]) -> Vec<[u8; 3]> {
        let mut out = Vec::new();
        parser.push(bytes, &mut out);
        out
    }

    #[test]
    fn stream_yields_a_single_complete_message() {
        // The degenerate case: every non-Android backend hands us exactly this.
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0x90, 60, 100]),
            vec![[0x90, 60, 100]]
        );
    }

    #[test]
    fn stream_splits_a_coalesced_buffer() {
        // The Android bug: a NoteOff riding in the same USB transfer as another
        // event used to be dropped, leaving the key lit on the virtual piano.
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0x90, 60, 100, 0x80, 60, 64, 0x90, 64, 90]),
            vec![[0x90, 60, 100], [0x80, 60, 64], [0x90, 64, 90]]
        );
    }

    #[test]
    fn stream_expands_running_status() {
        // One status byte, three note events — the last two turn the first two
        // off (NoteOn with velocity 0).
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0x90, 60, 100, 64, 100, 60, 0, 64, 0]),
            vec![
                [0x90, 60, 100],
                [0x90, 64, 100],
                [0x90, 60, 0],
                [0x90, 64, 0]
            ]
        );
    }

    #[test]
    fn stream_carries_a_message_across_buffers() {
        let mut parser = MidiStreamParser::new();
        assert!(parse_stream(&mut parser, &[0x90, 60]).is_empty());
        assert_eq!(parse_stream(&mut parser, &[100]), vec![[0x90, 60, 100]]);
        // Running status survives the split too.
        assert_eq!(parse_stream(&mut parser, &[60, 0]), vec![[0x90, 60, 0]]);
    }

    #[test]
    fn stream_ignores_real_time_bytes_even_mid_message() {
        // 0xF8 (clock) and 0xFE (active sensing) are legal between the data
        // bytes of another message and must leave it intact.
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0x90, 0xF8, 60, 0xFE, 100, 0xF8]),
            vec![[0x90, 60, 100]]
        );
    }

    #[test]
    fn stream_skips_sysex_and_resumes_after_it() {
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(
                &mut parser,
                &[0xF0, 0x7E, 0x00, 0x06, 0x01, 0xF7, 0x90, 60, 100],
            ),
            vec![[0x90, 60, 100]]
        );
    }

    #[test]
    fn stream_recovers_from_an_unterminated_sysex() {
        // A status byte other than 0xF7 aborts the SysEx rather than swallowing
        // the rest of the stream.
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0xF0, 0x7E, 0x00, 0x90, 60, 100]),
            vec![[0x90, 60, 100]]
        );
    }

    #[test]
    fn stream_cancels_running_status_on_system_common() {
        // 0xF3 (song select) + its data byte must not be read as note data, and
        // the bytes after it are orphans until a real status byte shows up.
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0x90, 60, 100, 0xF3, 0x05, 62, 100]),
            vec![[0x90, 60, 100]]
        );
        assert_eq!(
            parse_stream(&mut parser, &[0x80, 60, 64]),
            vec![[0x80, 60, 64]]
        );
    }

    #[test]
    fn stream_drops_data_bytes_with_no_status() {
        // Starting mid-stream (connected while the device was already sending).
        let mut parser = MidiStreamParser::new();
        assert!(parse_stream(&mut parser, &[60, 100, 64]).is_empty());
    }

    #[test]
    fn stream_pads_single_data_byte_messages() {
        // Program Change carries one data byte; padding keeps the 3-byte shape
        // `parse_midi` expects, which then ignores it as a non-note message.
        let mut parser = MidiStreamParser::new();
        let msgs = parse_stream(&mut parser, &[0xC0, 0x05, 0x90, 60, 100]);
        assert_eq!(msgs, vec![[0xC0, 0x05, 0], [0x90, 60, 100]]);
        assert!(parse_midi(&msgs[0], 0).is_none());
    }

    #[test]
    fn stream_handles_pitch_bend_between_notes() {
        let mut parser = MidiStreamParser::new();
        assert_eq!(
            parse_stream(&mut parser, &[0x90, 60, 100, 0xE0, 0x00, 0x40, 0x80, 60, 0]),
            vec![[0x90, 60, 100], [0xE0, 0x00, 0x40], [0x80, 60, 0]]
        );
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
