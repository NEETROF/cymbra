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

//! Pure, host-testable microphone-capture logic — no device, threads, or FFI
//! (change: add-acoustic-piano-input).
//!
//! Split out of [`super::audio_input`] so it can be unit-tested (and counted
//! by `cargo llvm-cov`) on CI hosts that have no capture device. The
//! cpal-input/thread glue in `audio_input.rs` stays out of the coverage gate;
//! everything genuinely testable — the capture lifecycle, and later the route
//! classification, calibration state machine and note detection — lives here.
//!
//! These types are internal to the engine (the FFI surface in
//! `audio_input.rs` only exposes plain scalars), so they are `#[frb(ignore)]`d
//! to keep them out of the generated bridge.

use flutter_rust_bridge::frb;

use super::audio_input::{InputRouteKind, InputRouteVerdict};

/// Classifies a capture route from the **platform's stable port-type
/// identifier** — never from a display name (spec: Input Route
/// Classification). The tokens are the raw identifiers each platform layer
/// forwards verbatim:
///
/// - iOS `AVAudioSession.Port` raw values: `MicrophoneBuiltIn`,
///   `MicrophoneWired`, `LineIn`, `USBAudio`, `BluetoothHFP`.
/// - Android `AudioDeviceInfo` type names: `TYPE_BUILTIN_MIC`,
///   `TYPE_WIRED_HEADSET`, `TYPE_USB_DEVICE`, `TYPE_USB_HEADSET`,
///   `TYPE_USB_ACCESSORY`, `TYPE_BLUETOOTH_SCO`, `TYPE_BLE_HEADSET`.
///
/// Anything unrecognized — including the desktop hosts, where `cpal` exposes
/// no transport type — degrades to [`InputRouteKind::Other`] and stays
/// usable, per the unknown-kind scenario.
pub(crate) fn classify_input_route(token: &str) -> InputRouteKind {
    match token {
        "MicrophoneBuiltIn" | "TYPE_BUILTIN_MIC" => InputRouteKind::Builtin,
        "MicrophoneWired" | "LineIn" | "TYPE_WIRED_HEADSET" => InputRouteKind::Wired,
        "USBAudio" | "TYPE_USB_DEVICE" | "TYPE_USB_HEADSET" | "TYPE_USB_ACCESSORY" => {
            InputRouteKind::Usb
        }
        "BluetoothHFP" | "TYPE_BLUETOOTH_SCO" | "TYPE_BLE_HEADSET" => InputRouteKind::Bluetooth,
        _ => InputRouteKind::Other,
    }
}

/// The acquisition verdict for a route kind (spec: Bluetooth Input Refusal).
/// Bluetooth capture runs over the voice profile — 8–16 kHz and 100–300 ms of
/// jitter against a ±160 ms binding window — so it is refused outright, a
/// hard rule rather than the output side's compensable wireless warning.
/// USB-C wireless receivers enumerate as class-compliant USB and pass.
pub(crate) fn input_route_verdict(kind: InputRouteKind) -> InputRouteVerdict {
    match kind {
        InputRouteKind::Bluetooth => InputRouteVerdict::RefusedBluetooth,
        InputRouteKind::Builtin
        | InputRouteKind::Wired
        | InputRouteKind::Usb
        | InputRouteKind::Other => InputRouteVerdict::Accepted,
    }
}

/// The capture lifecycle as the engine reasons about it. The glue owns the
/// actual cpal stream; this state machine owns the *decisions* — whether a
/// start/stop request changes anything — so double-starts and stray stops are
/// idempotent by construction and testable on any host.
#[frb(ignore)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureLifecycle {
    /// No capture: the microphone is not open.
    Idle,
    /// Capture runs: frames flow from the device.
    Running,
}

/// What a lifecycle request asks the glue to do with the real device.
#[frb(ignore)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureTransition {
    /// Open the device and start the stream.
    Open,
    /// Close the stream and release the device.
    Close,
    /// Nothing: the request is already satisfied.
    None,
}

impl CaptureLifecycle {
    /// Handles a start request: opens only from [`CaptureLifecycle::Idle`].
    pub(crate) fn request_start(&mut self) -> CaptureTransition {
        match self {
            CaptureLifecycle::Idle => {
                *self = CaptureLifecycle::Running;
                CaptureTransition::Open
            }
            CaptureLifecycle::Running => CaptureTransition::None,
        }
    }

    /// Handles a stop request: closes only from [`CaptureLifecycle::Running`].
    pub(crate) fn request_stop(&mut self) -> CaptureTransition {
        match self {
            CaptureLifecycle::Running => {
                *self = CaptureLifecycle::Idle;
                CaptureTransition::Close
            }
            CaptureLifecycle::Idle => CaptureTransition::None,
        }
    }

    /// Whether frames are (supposed to be) flowing.
    pub(crate) fn is_running(&self) -> bool {
        matches!(self, CaptureLifecycle::Running)
    }
}

/// How long the detector observes ambient noise before asking for the click.
pub(crate) const CALIB_BASELINE_MS: u64 = 150;

/// How long after the click emission the detector listens before giving up.
pub(crate) const CALIB_LISTEN_TIMEOUT_MS: u64 = 2000;

/// Analysis hop: RMS is evaluated per block of this many mono samples, so the
/// detection granularity is `HOP / rate` (~2.7 ms at 48 kHz) — far below the
/// ~10 ms usefulness bar of the measurement.
const CALIB_HOP: usize = 128;

/// Where a calibration run currently stands (spec: Measured Input-Offset
/// Calibration — armed → click emitted → detected | timeout).
#[frb(ignore)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CalibPhase {
    /// Accumulating the ambient noise floor; the click must not sound yet.
    Baseline,
    /// Baseline done: the glue should emit the reference click now and call
    /// [`CalibrationDetector::click_emitted`].
    ReadyToClick,
    /// Click emitted; scanning for its onset until the deadline.
    Listening,
    /// An outcome is available via [`CalibrationDetector::take_outcome`].
    Done,
}

/// The terminal result of a calibration run.
#[frb(ignore)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CalibOutcome {
    /// The click was heard: the measured emission→capture round trip.
    Detected { latency_ms: f64 },
    /// Nothing crossed the threshold before the deadline (mic muted, too far,
    /// too noisy). The caller shows guidance; no value is stored.
    TimedOut,
}

/// Pure onset-of-click detector over mono PCM, counted in samples so tests
/// drive it with synthetic buffers and no clock. The glue owns real time: it
/// feeds capture frames as they arrive, emits the click when the phase asks
/// for it, and polls the outcome.
#[frb(ignore)]
pub(crate) struct CalibrationDetector {
    sample_rate: u32,
    /// Total mono samples fed so far — the detector's only clock.
    fed: u64,
    /// Partial hop being accumulated (sum of squares + count).
    hop_energy: f64,
    hop_len: usize,
    /// Highest hop RMS observed during the baseline phase.
    noise_floor: f64,
    phase: CalibPhase,
    /// Sample position at which the glue reported the click emission.
    click_at: u64,
    /// Sample position after which listening times out.
    deadline: u64,
    outcome: Option<CalibOutcome>,
}

impl CalibrationDetector {
    pub(crate) fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            fed: 0,
            hop_energy: 0.0,
            hop_len: 0,
            noise_floor: 0.0,
            phase: CalibPhase::Baseline,
            click_at: 0,
            deadline: 0,
            outcome: None,
        }
    }

    pub(crate) fn phase(&self) -> CalibPhase {
        self.phase
    }

    /// The glue confirms the reference click was just handed to the output
    /// path. Stamps the emission on the sample clock and opens the listening
    /// window. Only meaningful in [`CalibPhase::ReadyToClick`].
    pub(crate) fn click_emitted(&mut self) {
        if self.phase == CalibPhase::ReadyToClick {
            self.click_at = self.fed;
            self.deadline = self.fed + self.ms_to_samples(CALIB_LISTEN_TIMEOUT_MS);
            self.phase = CalibPhase::Listening;
        }
    }

    /// Feeds mono samples; hop-by-hop RMS drives the phase machine.
    pub(crate) fn feed(&mut self, mono: &[f32]) {
        for &s in mono {
            self.fed += 1;
            self.hop_energy += f64::from(s) * f64::from(s);
            self.hop_len += 1;
            if self.hop_len == CALIB_HOP {
                let rms = (self.hop_energy / CALIB_HOP as f64).sqrt();
                self.hop_energy = 0.0;
                self.hop_len = 0;
                self.on_hop(rms);
            }
        }
    }

    /// One terminal outcome, consumed by the poller.
    pub(crate) fn take_outcome(&mut self) -> Option<CalibOutcome> {
        self.outcome.take()
    }

    fn on_hop(&mut self, rms: f64) {
        match self.phase {
            CalibPhase::Baseline => {
                self.noise_floor = self.noise_floor.max(rms);
                if self.fed >= self.ms_to_samples(CALIB_BASELINE_MS) {
                    self.phase = CalibPhase::ReadyToClick;
                }
            }
            CalibPhase::Listening => {
                // The click must clear the room by a wide margin; the absolute
                // floor keeps a dead-silent baseline from arming a hair
                // trigger on the first breath.
                let threshold = (self.noise_floor * 6.0).max(0.02);
                if rms > threshold {
                    let latency_samples = self.fed.saturating_sub(self.click_at);
                    self.outcome = Some(CalibOutcome::Detected {
                        latency_ms: latency_samples as f64 * 1000.0 / f64::from(self.sample_rate),
                    });
                    self.phase = CalibPhase::Done;
                } else if self.fed >= self.deadline {
                    self.outcome = Some(CalibOutcome::TimedOut);
                    self.phase = CalibPhase::Done;
                }
            }
            CalibPhase::ReadyToClick | CalibPhase::Done => {}
        }
    }

    fn ms_to_samples(&self, ms: u64) -> u64 {
        ms * u64::from(self.sample_rate) / 1000
    }
}

// ---------------------------------------------------------------------------
// Acoustic piano note detection (spec: Score-Informed Presence Detection).
// ---------------------------------------------------------------------------

/// Analysis hop for the onset follower.
const DETECT_HOP: usize = 128;

/// Fast/slow energy follower ratio that declares an attack transient.
const ONSET_RATIO: f64 = 3.0;

/// Absolute RMS floor under which no onset fires (breath, hiss).
const ONSET_ABS_FLOOR: f64 = 0.01;

/// Refractory period after an onset before another may fire.
const ONSET_REFRACTORY_MS: u64 = 60;

/// The **nominal** pitch-confirmation window (samples at 44.1/48 kHz land in
/// the same ballpark): what the Dart scoring layer adds to the measured input
/// offset as `kDetectionConfirmMs`. Low pitches confirm over a longer window
/// (see [`confirm_window`]) — a known POC approximation the offset does not
/// model per pitch.
pub(crate) const DETECTION_CONFIRM_NOMINAL_MS: u64 = 46;

/// How long after its onset a detected note's synthetic release is emitted.
/// Releases are undetectable under the damper pedal (spec: attack events
/// always, releases best-effort), but downstream key feedback needs *some*
/// off, so the detector schedules one on its sample clock.
const SYNTHETIC_OFF_MS: u64 = 300;

/// The longest evidence buffer an onset keeps (per-pitch windows cap here).
const MAX_EVIDENCE: usize = 8192;

/// One detection emission, on the detector's sample clock.
#[frb(ignore)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DetectedNote {
    /// True for the attack, false for the scheduled synthetic release.
    pub(crate) on: bool,
    /// MIDI note number.
    pub(crate) pitch: u8,
    /// Velocity estimated from the attack energy (0 for a release).
    pub(crate) velocity: u8,
    /// Sample position of the **onset** this emission belongs to — never the
    /// confirmation instant (spec: Onset Timing Decoupled From Pitch
    /// Confirmation). Releases carry their scheduled position instead.
    pub(crate) at_sample: u64,
}

/// Streaming score-informed note detector over mono PCM.
///
/// Two decoupled stages (design D3): a broadband energy-transient **onset**
/// stage stamps *when* something was struck; a Goertzel **presence** stage
/// then confirms *which of the expected pitches* are sounding in the evidence
/// window that follows. Pitches outside the expected set are never reported —
/// downstream judgment only sees what arrives (spec: unreported extras carry
/// no penalty) — and a broadband click with no tonal match (the metronome)
/// confirms nothing by construction.
#[frb(ignore)]
pub(crate) struct NoteDetector {
    sample_rate: u32,
    /// Total mono samples fed — the detector's only clock.
    fed: u64,
    hop_energy: f64,
    hop_len: usize,
    /// Slow-moving RMS average (the "room + sustained notes" level).
    slow: f64,
    refractory_until: u64,
    /// The score's active window, set by the app; empty = detection idle.
    expected: Vec<u8>,
    /// The open onset, if any: its sample position, its evidence buffer, its
    /// peak amplitude, and which pitches already confirmed for it.
    onset: Option<OnsetWindow>,
    /// Scheduled synthetic releases `(due_sample, pitch)`.
    pending_offs: Vec<(u64, u8)>,
}

#[frb(ignore)]
struct OnsetWindow {
    at_sample: u64,
    buf: Vec<f32>,
    peak: f32,
    confirmed: Vec<u8>,
}

impl NoteDetector {
    pub(crate) fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            fed: 0,
            hop_energy: 0.0,
            hop_len: 0,
            slow: 0.0,
            refractory_until: 0,
            expected: Vec::new(),
            onset: None,
            pending_offs: Vec::new(),
        }
    }

    /// The rate the detector was built at (the capture stream's).
    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Replaces the expected set (the score's active window). Clearing it
    /// closes any open onset and idles the detector.
    pub(crate) fn set_expected(&mut self, pitches: Vec<u8>) {
        self.expected = pitches;
        if self.expected.is_empty() {
            self.onset = None;
        }
    }

    /// Feeds mono samples; returns every emission they completed.
    pub(crate) fn feed(&mut self, mono: &[f32]) -> Vec<DetectedNote> {
        let mut out = Vec::new();
        for &s in mono {
            self.fed += 1;

            // Evidence accumulation for the open onset.
            if let Some(w) = self.onset.as_mut() {
                if w.buf.len() < MAX_EVIDENCE {
                    w.buf.push(s);
                    w.peak = w.peak.max(s.abs());
                }
                self.try_confirm(&mut out);
                if self
                    .onset
                    .as_ref()
                    .is_some_and(|w| w.buf.len() >= MAX_EVIDENCE)
                {
                    self.onset = None;
                }
            }

            // Onset following.
            self.hop_energy += f64::from(s) * f64::from(s);
            self.hop_len += 1;
            if self.hop_len == DETECT_HOP {
                let rms = (self.hop_energy / DETECT_HOP as f64).sqrt();
                self.hop_energy = 0.0;
                self.hop_len = 0;
                self.on_hop(rms);
            }

            // Due synthetic releases.
            if !self.pending_offs.is_empty() {
                let fed = self.fed;
                self.pending_offs.retain(|&(due, pitch)| {
                    if due <= fed {
                        out.push(DetectedNote {
                            on: false,
                            pitch,
                            velocity: 0,
                            at_sample: due,
                        });
                        false
                    } else {
                        true
                    }
                });
            }
        }
        out
    }

    fn on_hop(&mut self, rms: f64) {
        // A fresh detector has no room level yet: adopt the first hop.
        if self.slow == 0.0 {
            self.slow = rms;
        }
        let attack = rms > (self.slow * ONSET_RATIO).max(ONSET_ABS_FLOOR);
        // EMA after the comparison, so the attack itself does not raise the
        // bar it is compared against.
        self.slow = self.slow * 0.95 + rms * 0.05;

        if attack && self.fed >= self.refractory_until && !self.expected.is_empty() {
            self.refractory_until = self.fed + self.ms_to_samples(ONSET_REFRACTORY_MS);
            // Include the just-analyzed hop in the evidence: the transient's
            // first samples carry energy the presence stage should see.
            self.onset = Some(OnsetWindow {
                at_sample: self.fed.saturating_sub(DETECT_HOP as u64),
                buf: Vec::with_capacity(MAX_EVIDENCE),
                peak: 0.0,
                confirmed: Vec::new(),
            });
        }
    }

    /// Confirms every expected pitch whose window is now full, at most once
    /// per onset.
    fn try_confirm(&mut self, out: &mut Vec<DetectedNote>) {
        let Some(w) = self.onset.as_mut() else { return };
        let rate = self.sample_rate;
        for &pitch in &self.expected {
            if w.confirmed.contains(&pitch) {
                continue;
            }
            let window = confirm_window(rate, pitch);
            if w.buf.len() < window {
                continue;
            }
            if pitch_present(&w.buf[..window], rate, pitch) {
                w.confirmed.push(pitch);
                let velocity = velocity_from_peak(w.peak);
                out.push(DetectedNote {
                    on: true,
                    pitch,
                    velocity,
                    at_sample: w.at_sample,
                });
                self.pending_offs
                    .push((w.at_sample + ms_to_samples(rate, SYNTHETIC_OFF_MS), pitch));
            }
        }
    }

    fn ms_to_samples(&self, ms: u64) -> u64 {
        ms_to_samples(self.sample_rate, ms)
    }
}

fn ms_to_samples(rate: u32, ms: u64) -> u64 {
    ms * u64::from(rate) / 1000
}

/// MIDI note number → frequency in Hz (equal temperament, A4 = 440).
fn pitch_freq(pitch: u8) -> f64 {
    440.0 * 2f64.powf((f64::from(pitch) - 69.0) / 12.0)
}

/// The evidence window a pitch needs before its presence can be judged:
/// enough cycles that the Goertzel main lobe separates it from the
/// half-semitone controls (~17 periods), floored at the nominal window and
/// capped at [`MAX_EVIDENCE`]. Low pitches therefore confirm later — the
/// documented POC approximation behind [`DETECTION_CONFIRM_NOMINAL_MS`].
fn confirm_window(rate: u32, pitch: u8) -> usize {
    let periods = 17.0 * f64::from(rate) / pitch_freq(pitch);
    (periods as usize)
        .max(ms_to_samples(rate, DETECTION_CONFIRM_NOMINAL_MS) as usize)
        .min(MAX_EVIDENCE)
}

/// Goertzel power of `freq` over `buf`.
fn goertzel(buf: &[f32], rate: u32, freq: f64) -> f64 {
    let w = 2.0 * std::f64::consts::PI * freq / f64::from(rate);
    let coeff = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &x in buf {
        let s0 = f64::from(x) + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coeff * s1 * s2) / (buf.len() as f64)
}

/// Whether `pitch` is tonally present in `buf`: the energy at its fundamental
/// must clearly beat semitone **controls** on both sides. Ratio-normalized,
/// so detuning within the main lobe passes, a broadband transient (a
/// metronome click) fails on the controls, and the absolute level does not
/// matter beyond a floor. The fundamental alone decides: folding harmonics in
/// let a neighboring pitch's fundamental leak through the harmonic bin (B4 at
/// 494 Hz reading as C4's 523 Hz second harmonic).
fn pitch_present(buf: &[f32], rate: u32, pitch: u8) -> bool {
    let f = pitch_freq(pitch);
    let signal = goertzel(buf, rate, f);
    if signal < 1e-7 {
        return false;
    }
    let semitone = 2f64.powf(1.0 / 12.0);
    let control = goertzel(buf, rate, f * semitone)
        .max(goertzel(buf, rate, f / semitone))
        .max(1e-9);
    signal / control > 4.0
}

/// Attack peak → MIDI velocity. A rough monotone map — audio velocity is an
/// estimate by design (proposal: velocity fidelity is a non-goal).
fn velocity_from_peak(peak: f32) -> u8 {
    let v = 30.0 + f64::from(peak) * 130.0;
    v.clamp(30.0, 112.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_from_idle_opens_and_runs() {
        let mut l = CaptureLifecycle::Idle;
        assert_eq!(l.request_start(), CaptureTransition::Open);
        assert!(l.is_running());
    }

    #[test]
    fn double_start_is_idempotent() {
        let mut l = CaptureLifecycle::Idle;
        l.request_start();
        assert_eq!(l.request_start(), CaptureTransition::None);
        assert!(l.is_running());
    }

    #[test]
    fn stop_from_running_closes_and_idles() {
        let mut l = CaptureLifecycle::Idle;
        l.request_start();
        assert_eq!(l.request_stop(), CaptureTransition::Close);
        assert!(!l.is_running());
    }

    #[test]
    fn stray_stop_is_idempotent() {
        let mut l = CaptureLifecycle::Idle;
        assert_eq!(l.request_stop(), CaptureTransition::None);
        assert!(!l.is_running());
    }

    #[test]
    fn ios_tokens_classify() {
        assert_eq!(
            classify_input_route("MicrophoneBuiltIn"),
            InputRouteKind::Builtin
        );
        assert_eq!(
            classify_input_route("MicrophoneWired"),
            InputRouteKind::Wired
        );
        assert_eq!(classify_input_route("USBAudio"), InputRouteKind::Usb);
        assert_eq!(
            classify_input_route("BluetoothHFP"),
            InputRouteKind::Bluetooth
        );
    }

    #[test]
    fn android_tokens_classify() {
        assert_eq!(
            classify_input_route("TYPE_BUILTIN_MIC"),
            InputRouteKind::Builtin
        );
        assert_eq!(
            classify_input_route("TYPE_WIRED_HEADSET"),
            InputRouteKind::Wired
        );
        assert_eq!(
            classify_input_route("TYPE_USB_HEADSET"),
            InputRouteKind::Usb
        );
        assert_eq!(
            classify_input_route("TYPE_BLUETOOTH_SCO"),
            InputRouteKind::Bluetooth
        );
        assert_eq!(
            classify_input_route("TYPE_BLE_HEADSET"),
            InputRouteKind::Bluetooth
        );
    }

    #[test]
    fn unknown_token_degrades_to_other() {
        assert_eq!(classify_input_route("CarPlayThing"), InputRouteKind::Other);
        assert_eq!(classify_input_route(""), InputRouteKind::Other);
        // Display names must never classify, even ones containing hints.
        assert_eq!(
            classify_input_route("Bob's Bluetooth Mic"),
            InputRouteKind::Other
        );
    }

    #[test]
    fn bluetooth_is_refused_everything_else_accepted() {
        assert_eq!(
            input_route_verdict(InputRouteKind::Bluetooth),
            InputRouteVerdict::RefusedBluetooth
        );
        for kind in [
            InputRouteKind::Builtin,
            InputRouteKind::Wired,
            InputRouteKind::Usb,
            InputRouteKind::Other,
        ] {
            assert_eq!(input_route_verdict(kind), InputRouteVerdict::Accepted);
        }
    }

    const RATE: u32 = 48_000;

    /// `ms` of quiet room noise (well under the absolute floor).
    fn noise(ms: u64) -> Vec<f32> {
        let n = (ms * u64::from(RATE) / 1000) as usize;
        (0..n)
            .map(|i| if i % 2 == 0 { 0.004 } else { -0.004 })
            .collect()
    }

    /// `ms` of a loud click-like burst.
    fn burst(ms: u64) -> Vec<f32> {
        let n = (ms * u64::from(RATE) / 1000) as usize;
        (0..n)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect()
    }

    #[test]
    fn calibration_measures_the_round_trip() {
        let mut d = CalibrationDetector::new(RATE);
        d.feed(&noise(CALIB_BASELINE_MS + 10));
        assert_eq!(d.phase(), CalibPhase::ReadyToClick);

        d.click_emitted();
        d.feed(&noise(30));
        d.feed(&burst(10));
        let Some(CalibOutcome::Detected { latency_ms }) = d.take_outcome() else {
            panic!("expected a detection");
        };
        // Onset lands at the first hop inside the burst: ~30 ms plus at most
        // a couple of hops (~2.7 ms each) of quantization.
        assert!(
            (28.0..40.0).contains(&latency_ms),
            "latency_ms = {latency_ms}"
        );
    }

    #[test]
    fn calibration_times_out_in_silence() {
        let mut d = CalibrationDetector::new(RATE);
        d.feed(&noise(CALIB_BASELINE_MS + 10));
        d.click_emitted();
        d.feed(&noise(CALIB_LISTEN_TIMEOUT_MS + 100));
        assert_eq!(d.take_outcome(), Some(CalibOutcome::TimedOut));
    }

    #[test]
    fn noisy_room_raises_the_threshold_not_a_false_positive() {
        let mut d = CalibrationDetector::new(RATE);
        // Baseline noise at 0.01 RMS-ish → threshold 6× above it.
        let loudish: Vec<f32> = (0..(u64::from(RATE) * (CALIB_BASELINE_MS + 10) / 1000) as usize)
            .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
            .collect();
        d.feed(&loudish);
        d.click_emitted();
        // The same ambient level must not read as the click.
        let more: Vec<f32> = (0..(u64::from(RATE) / 10) as usize)
            .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
            .collect();
        d.feed(&more);
        assert_eq!(d.phase(), CalibPhase::Listening);
        assert!(d.take_outcome().is_none());
    }

    #[test]
    fn outcome_is_consumed_once() {
        let mut d = CalibrationDetector::new(RATE);
        d.feed(&noise(CALIB_BASELINE_MS + 10));
        d.click_emitted();
        d.feed(&burst(10));
        assert!(d.take_outcome().is_some());
        assert!(d.take_outcome().is_none());
        assert_eq!(d.phase(), CalibPhase::Done);
    }

    // -- Note detection ----------------------------------------------------

    /// `ms` of the sum of sines at the given MIDI pitches.
    fn tone(ms: u64, pitches: &[u8], amp: f32) -> Vec<f32> {
        let n = (ms * u64::from(RATE) / 1000) as usize;
        (0..n)
            .map(|i| {
                let t = i as f64 / f64::from(RATE);
                pitches
                    .iter()
                    .map(|&p| {
                        (2.0 * std::f64::consts::PI * pitch_freq(p) * t).sin() * f64::from(amp)
                    })
                    .sum::<f64>() as f32
            })
            .collect()
    }

    /// Quiet room, then the tone: the level step is the onset.
    fn play(d: &mut NoteDetector, pitches: &[u8], ms: u64) -> Vec<DetectedNote> {
        let mut out = d.feed(&noise(200));
        out.extend(d.feed(&tone(ms, pitches, 0.3)));
        out
    }

    const A4: u8 = 69;
    const C4: u8 = 60;
    const E4: u8 = 64;
    const G4: u8 = 67;

    #[test]
    fn expected_note_is_detected_at_its_onset() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![A4]);
        let quiet_samples = 200 * u64::from(RATE) / 1000;
        let events = play(&mut d, &[A4], 300);

        let ons: Vec<_> = events.iter().filter(|e| e.on).collect();
        assert_eq!(ons.len(), 1, "events: {events:?}");
        assert_eq!(ons[0].pitch, A4);
        assert!(ons[0].velocity >= 30);
        // The timestamp is the onset's, not the confirmation's: within a few
        // hops of the level step.
        let err = ons[0].at_sample.abs_diff(quiet_samples);
        assert!(err < 6 * DETECT_HOP as u64, "onset error: {err} samples");
    }

    #[test]
    fn chord_confirms_each_expected_pitch_and_nothing_else() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![C4, E4, G4]);
        let events = play(&mut d, &[C4, E4], 400);

        let mut on_pitches: Vec<u8> = events.iter().filter(|e| e.on).map(|e| e.pitch).collect();
        on_pitches.sort_unstable();
        assert_eq!(on_pitches, vec![C4, E4], "events: {events:?}");
    }

    #[test]
    fn unexpected_pitch_emits_nothing() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![C4]);
        // B4 sounds; C4 is expected: no tonal match, no event.
        let events = play(&mut d, &[71], 300);
        assert!(events.iter().all(|e| !e.on), "events: {events:?}");
    }

    #[test]
    fn empty_expected_set_idles_the_detector() {
        let mut d = NoteDetector::new(RATE);
        let events = play(&mut d, &[A4], 300);
        assert!(events.is_empty());
    }

    #[test]
    fn damper_sustained_repeat_is_a_second_onset() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![A4]);
        let mut events = play(&mut d, &[A4], 300);
        // The note decays under the pedal, then is restruck louder: the level
        // step over the decayed sustain is a fresh onset of the same pitch.
        events.extend(d.feed(&tone(400, &[A4], 0.06)));
        events.extend(d.feed(&tone(300, &[A4], 0.5)));

        let ons: Vec<_> = events.iter().filter(|e| e.on).collect();
        assert_eq!(ons.len(), 2, "events: {events:?}");
        assert!(ons.iter().all(|e| e.pitch == A4));
    }

    #[test]
    fn metronome_click_alone_confirms_nothing() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![C4, E4]);
        let mut events = d.feed(&noise(200));
        // A short broadband burst — the click — fires the onset stage but has
        // no tonal match, so the presence stage rejects it by construction.
        events.extend(d.feed(&burst(5)));
        events.extend(d.feed(&noise(300)));
        assert!(events.iter().all(|e| !e.on), "events: {events:?}");
    }

    #[test]
    fn metronome_click_does_not_mask_a_real_note() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![A4]);
        let mut events = d.feed(&noise(200));
        // Click and note strike together: the tonal evidence still wins.
        let mut mix = tone(300, &[A4], 0.3);
        for (i, s) in burst(5).into_iter().enumerate() {
            mix[i] = (mix[i] + s).clamp(-1.0, 1.0);
        }
        events.extend(d.feed(&mix));
        let ons: Vec<_> = events.iter().filter(|e| e.on).collect();
        assert_eq!(ons.len(), 1, "events: {events:?}");
        assert_eq!(ons[0].pitch, A4);
    }

    #[test]
    fn synthetic_release_follows_the_attack() {
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![A4]);
        let events = play(&mut d, &[A4], 600);

        let on = events.iter().find(|e| e.on).expect("an attack");
        let off = events.iter().find(|e| !e.on).expect("a release");
        assert_eq!(off.pitch, A4);
        assert_eq!(
            off.at_sample - on.at_sample,
            SYNTHETIC_OFF_MS * u64::from(RATE) / 1000
        );
    }

    #[test]
    fn low_pitch_uses_a_longer_confirmation_window() {
        // A2 (110 Hz) needs ~17 periods; A5 (880 Hz) sits on the floor.
        assert!(confirm_window(RATE, 45) > confirm_window(RATE, 81));
        assert!(confirm_window(RATE, 45) <= MAX_EVIDENCE);
    }
}
