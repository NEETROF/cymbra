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

/// Resolves the device a capture should open (spec: Desktop Capture Device
/// Selection): an exact match on the requested name, or the system default —
/// never a failure — when nothing (or something absent) was requested. Pure,
/// so the fallback rule is host-testable.
pub(crate) fn resolve_input_device<'a>(
    requested: Option<&str>,
    available: &'a [String],
) -> Option<&'a str> {
    let name = requested?;
    available
        .iter()
        .find(|d| d.as_str() == name)
        .map(String::as_str)
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
    /// Loudest hop RMS seen while listening — diagnostic: what the mic
    /// actually heard, threshold or not.
    listen_peak: f64,
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
            listen_peak: 0.0,
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
                self.listen_peak = self.listen_peak.max(rms);
                let threshold = self.threshold();
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

    /// The level the click must clear. **Ratio-first**, absolute-last: with
    /// the voice-processing chain off (`.measurement` / UNPROCESSED) the raw
    /// capture level varies wildly across devices — an iPad at full volume
    /// measured its own accented click at 0.0012 RMS over a 0.0001 floor
    /// (2026-09-07), 13× the room but far under any workable absolute. So the
    /// room ratio decides, and the tiny absolute only keeps a dead-silent
    /// baseline (floor ≈ 0) from arming a hair trigger.
    fn threshold(&self) -> f64 {
        (self.noise_floor * 5.0).max(0.0005)
    }

    /// Diagnostics for a failed run: the room, what the mic heard while
    /// listening, and the bar it had to clear — enough to tell "capture is
    /// silent" from "the click never cleared the threshold".
    pub(crate) fn diagnostics(&self) -> (f64, f64, f64) {
        (self.noise_floor, self.listen_peak, self.threshold())
    }
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Acoustic piano note detection (spec: Score-Informed Presence Detection).
// ---------------------------------------------------------------------------

/// Analysis hop: per-pitch envelopes are re-evaluated once per hop.
const DETECT_HOP: usize = 128;

/// Per-pitch envelope window: short enough to localize an attack (~21 ms at
/// 48 kHz), long enough for a stable Goertzel reading.
const ENV_WINDOW: usize = 1024;

/// A pitch triggers when its own envelope rises this far above its own slow
/// average. Per-pitch by design: a broadband level gate cannot see a second
/// note struck while the first still rings (the total RMS barely moves), which
/// is exactly how consecutive notes went unread on a real piano.
const ENV_RISE_RATIO: f64 = 4.0;

/// Envelope floor below which a rise never triggers (near-digital silence;
/// unprocessed capture levels are tiny — see the calibration threshold note).
const ENV_ABS_FLOOR: f64 = 1e-9;

/// A pitch that emitted stays quiet for this long: its own ring keeps the
/// envelope high, and one keystroke must stay one event.
const PITCH_REFRACTORY_MS: u64 = 200;

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

/// The sliding sample buffer confirmations read from (per-pitch confirmation
/// windows cap here).
const MAX_EVIDENCE: usize = 8192;

/// A failed confirmation retries at this cadence until
/// [`CONFIRM_RETRY_BUDGET_MS`] runs out. One shot per trigger was
/// structurally chord-blind: the single evaluation lands right on the attack,
/// where the hammer transient and the OTHER co-struck notes inflate both the
/// broadband and the probes, so only the loudest chord member survived it —
/// and a decaying note can never re-trigger (its slow average rose while it
/// waited). On device, sol failed at sig=8.2e-3 inside the transient and
/// passed at 6.1e-3 once it had decayed: the timing decided, not the level.
/// The wrong-note vetoes hold across retries — a struck neighbor or lower
/// octave keeps its convicting bins hot through the whole budget, and a
/// click's window slides out entirely (the tonality gate then sees noise).
const CONFIRM_RETRY_STEP_MS: u64 = 43;

/// How long past the first evaluation a triggered pitch keeps retrying.
/// Bounded well under the Wait Mode reaction windows (450/1200 ms): a note
/// confirmed on the last retry still reads as an immediate answer.
const CONFIRM_RETRY_BUDGET_MS: u64 = 260;

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

/// Per-expected-pitch tracking state.
#[frb(ignore)]
struct PitchTracker {
    pitch: u8,
    /// Slow envelope average — the pitch's own recent level, followed
    /// quickly downward (a piano decays) and slowly upward.
    slow: f64,
    /// Slowly-decaying envelope peak. Two neighboring notes sounding together
    /// BEAT (the short window cannot resolve a semitone), swinging the bin's
    /// envelope several-fold at a few hertz — held peaks are what a beat
    /// maximum can never exceed, while a real re-strike does.
    peak: f64,
    /// Consecutive hops the rise condition has held (a noise fluke spikes a
    /// bin for one reading; a struck note stays up).
    rise_hops: u8,
    /// Recent per-hop envelope readings (newest last), for edge-locating the
    /// attack at emission time: the trigger only ARMS a pitch — a noise
    /// fluke arming it just before the real strike must not steal the
    /// timestamp.
    hist: Vec<f64>,
    /// Sample position of a pending rise, awaiting confirmation.
    triggered_at: Option<u64>,
    /// Earliest sample position the next confirmation attempt may run at —
    /// paces the retries so the Goertzel battery is not recomputed every hop.
    next_conf_at: u64,
    /// Sample position of the last emission (`0` = never).
    last_emit: u64,
    /// Re-arm hysteresis: false after an emission until the envelope dips
    /// below a fraction of its peak. A REPEATED note (mi-mi, the opening of
    /// every other melody) dips between strikes and re-arms; the steady ring
    /// of a still-held note never does. Replaces a peak-ratio bar that made
    /// every repeat fight its own resonance.
    armed: bool,
}

/// Streaming score-informed note detector over mono PCM.
///
/// Two decoupled stages (design D3), both **per expected pitch**: a
/// short-window envelope-rise stage stamps *when* that pitch was struck —
/// per-pitch because a broadband gate is blind to a note played over another
/// note's ring — and the far-probe presence stage then confirms the rise is
/// tonal over a longer window, which is what rejects the metronome click and
/// other broadband transients. Pitches outside the expected set are never
/// reported — downstream judgment only sees what arrives (spec: unreported
/// extras carry no penalty).
#[frb(ignore)]
pub(crate) struct NoteDetector {
    sample_rate: u32,
    /// Total mono samples fed — the detector's only clock.
    fed: u64,
    hop_buf: Vec<f32>,
    /// Sliding buffer of the most recent samples (≤ [`MAX_EVIDENCE`]).
    ring: Vec<f32>,
    trackers: Vec<PitchTracker>,
    /// Slow broadband RMS average, for the strike detector below.
    bb_slow: f64,
    /// Sample position of the last broadband energy jump — a hammer strike.
    /// Re-triggering an already-emitted pitch requires one nearby: a real
    /// repeat bumps the TOTAL level, a beat between two ringing neighbors
    /// only sloshes energy between bins (the sum of two sines has constant
    /// power) and must never re-emit.
    last_strike_at: u64,
    /// Scheduled synthetic releases `(due_sample, pitch)`.
    pending_offs: Vec<(u64, u8)>,
    /// POC diagnostics: one line per trigger/confirm/reject, drained by the
    /// glue into the platform log so an on-device run can be analyzed.
    pub(crate) debug_log: Vec<String>,
}

impl NoteDetector {
    pub(crate) fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            fed: 0,
            hop_buf: Vec::with_capacity(DETECT_HOP),
            ring: Vec::with_capacity(MAX_EVIDENCE + DETECT_HOP),
            trackers: Vec::new(),
            bb_slow: 0.0,
            last_strike_at: 0,
            pending_offs: Vec::new(),
            debug_log: Vec::new(),
        }
    }

    /// The rate the detector was built at (the capture stream's).
    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Replaces the expected set (the score's active window). Trackers of
    /// pitches that stay carry their state over — the window slides on every
    /// playhead move, and resetting a ringing pitch's average would re-arm it.
    pub(crate) fn set_expected(&mut self, pitches: Vec<u8>) {
        let mut next = Vec::with_capacity(pitches.len());
        for p in pitches {
            if next.iter().any(|t: &PitchTracker| t.pitch == p) {
                continue;
            }
            let old = self.trackers.iter().position(|t| t.pitch == p);
            next.push(match old {
                Some(i) => self.trackers.swap_remove(i),
                None => PitchTracker {
                    pitch: p,
                    slow: 0.0,
                    peak: 0.0,
                    rise_hops: 0,
                    hist: Vec::new(),
                    triggered_at: None,
                    next_conf_at: 0,
                    last_emit: 0,
                    armed: true,
                },
            });
        }
        self.trackers = next;
    }

    /// Feeds mono samples; returns every emission they completed.
    pub(crate) fn feed(&mut self, mono: &[f32]) -> Vec<DetectedNote> {
        let mut out = Vec::new();
        for &s in mono {
            self.fed += 1;
            self.hop_buf.push(s);
            if self.hop_buf.len() == DETECT_HOP {
                self.ring.extend_from_slice(&self.hop_buf);
                self.hop_buf.clear();
                let excess = self.ring.len().saturating_sub(MAX_EVIDENCE);
                if excess > 0 {
                    self.ring.drain(..excess);
                }
                self.on_hop(&mut out);
            }
        }
        out
    }

    fn on_hop(&mut self, out: &mut Vec<DetectedNote>) {
        let fed = self.fed;
        let rate = self.sample_rate;
        // Broadband strike follower (hammer transients raise the TOTAL level).
        if self.ring.len() >= DETECT_HOP {
            let r = rms(&self.ring[self.ring.len() - DETECT_HOP..]);
            if self.bb_slow > 0.0 && r > (self.bb_slow * 2.0).max(1e-5) {
                self.last_strike_at = fed;
            }
            self.bb_slow = if self.bb_slow == 0.0 {
                r.max(1e-9)
            } else if r < self.bb_slow {
                self.bb_slow * 0.90 + r * 0.10
            } else {
                self.bb_slow * 0.95 + r * 0.05
            };
        }
        if self.ring.len() >= ENV_WINDOW {
            let env_slice = &self.ring[self.ring.len() - ENV_WINDOW..];
            let expected: Vec<u8> = self.trackers.iter().map(|t| t.pitch).collect();
            let self_last_strike = self.last_strike_at;
            for t in &mut self.trackers {
                let env = goertzel(env_slice, rate, pitch_freq(t.pitch));
                t.hist.push(env);
                if t.hist.len() > 64 {
                    t.hist.remove(0);
                }

                if let Some(at) = t.triggered_at {
                    // Awaiting confirmation over the long window — doubled
                    // when an expected semitone NEIGHBOR exists: confirming
                    // mi against fa needs resolution a base window can't
                    // give while it still straddles pre-attack silence.
                    let has_neighbor = [-1i16, 1].iter().any(|&d| {
                        let np = i16::from(t.pitch) + d;
                        (0..=127).contains(&np) && expected.contains(&(np as u8))
                    });
                    let window = (confirm_window(rate, t.pitch) * if has_neighbor { 2 } else { 1 })
                        .min(MAX_EVIDENCE);
                    let elapsed = fed.saturating_sub(at);
                    if elapsed >= window as u64 && fed >= t.next_conf_at {
                        let have = self.ring.len().min(window);
                        let slice = &self.ring[self.ring.len() - have..];
                        // A held note still carries energy in the LAST
                        // quarter of the window; a click — wherever it sits
                        // in the slice — leaves it at the noise floor.
                        // Broadband RMS, so the beating that nulls a per-bin
                        // envelope mid-note (mi+fa beat at ~19 Hz) cannot
                        // fake a silence.
                        let q = (have / 4).max(1);
                        let head = rms(slice);
                        let tail = rms(&slice[have - q..]);
                        let still_sounding = tail * 4.0 >= head;
                        // An expected semitone NEIGHBOR that was actually
                        // played leaks heavily into this bin: only emit when
                        // this bin holds its own against the neighbor's. Over
                        // a DOUBLED window — at the confirmation length the
                        // two lobes still overlap and their phases interfere,
                        // which is exactly what made mi disappear under fa.
                        let disc_len = (2 * window).min(self.ring.len());
                        let disc = &self.ring[self.ring.len() - disc_len..];
                        let own = goertzel_hann(disc, rate, pitch_freq(t.pitch));
                        let f = pitch_freq(t.pitch);
                        let h2 = goertzel_hann(slice, rate, 2.0 * f);
                        let signal = goertzel_hann(slice, rate, f);
                        let beats_neighbors = [-1i16, 1].iter().all(|&d| {
                            let np = i16::from(t.pitch) + d;
                            if !(0..=127).contains(&np) {
                                return true;
                            }
                            let nb = goertzel_hann(disc, rate, pitch_freq(np as u8));
                            if expected.contains(&(np as u8)) {
                                // Both may genuinely sound (mi+fa): only veto
                                // when the neighbor dwarfs this bin.
                                own * 2.5 >= nb
                            } else {
                                // An UNEXPECTED semitone neighbor louder than
                                // the expected pitch is the wrong note being
                                // played (si accepted for do, on device): the
                                // expected bin then only holds leakage. And
                                // the harmonic contest (see above): the
                                // neighbor's octave outshining ours convicts
                                // the neighbor even when the fundamentals
                                // are ambiguous.
                                let nb_h2 = goertzel_hann(slice, rate, 2.0 * pitch_freq(np as u8));
                                own * 1.5 >= nb && h2 * 2.0 >= nb_h2
                            }
                        });
                        let present = pitch_present(slice, rate, t.pitch, &expected);
                        // Harmonic CONTEST, not an absolute harmonic gate: a
                        // real note's octave level varies wildly with the
                        // instrument and mic position (on-device, honest fa
                        // strikes measured 0.2–0.5% of the fundamental —
                        // below any workable absolute bar, overlapping the
                        // leak range). What never lies: the neighbors'
                        // octaves live far apart (fa5 = 698 Hz, fa♯5 =
                        // 740 Hz — no leakage up there), so if a neighbor's
                        // octave outshines this pitch's, the neighbor is
                        // what was struck.
                        let nb_lo =
                            goertzel_hann(disc, rate, pitch_freq(t.pitch.saturating_sub(1)));
                        let nb_hi =
                            goertzel_hann(disc, rate, pitch_freq(t.pitch.saturating_add(1)));
                        self.debug_log.push(format!(
                            "conf p={} sounding={still_sounding} neighbors={beats_neighbors} present={present} own={own:.2e} nb_lo={nb_lo:.2e} nb_hi={nb_hi:.2e} sig={signal:.2e} h2={h2:.2e} rms={head:.2e}",
                            t.pitch
                        ));
                        if still_sounding && beats_neighbors && present {
                            t.triggered_at = None;
                            t.last_emit = fed;
                            t.armed = false;
                            let peak = slice.iter().fold(0f32, |m, &x| m.max(x.abs()));
                            // Edge-located onset: the first hop whose env
                            // reached a quarter of the recent maximum IS the
                            // attack — wherever the arming trigger sat.
                            let hist_max = t.hist.iter().cloned().fold(0.0f64, f64::max);
                            let edge_idx = t
                                .hist
                                .iter()
                                .position(|&e| e >= hist_max * 0.25)
                                .unwrap_or(0);
                            let hops_ago = (t.hist.len() - 1 - edge_idx) as u64;
                            let edge = fed
                                .saturating_sub(hops_ago * DETECT_HOP as u64)
                                .saturating_sub(ENV_WINDOW as u64);
                            let onset = edge.max(at.saturating_sub(ENV_WINDOW as u64));
                            out.push(DetectedNote {
                                on: true,
                                pitch: t.pitch,
                                velocity: velocity_from_peak(peak),
                                at_sample: onset,
                            });
                            self.pending_offs
                                .push((onset + ms_to_samples(rate, SYNTHETIC_OFF_MS), t.pitch));
                        } else if elapsed
                            >= window as u64 + ms_to_samples(rate, CONFIRM_RETRY_BUDGET_MS)
                        {
                            t.triggered_at = None;
                        } else {
                            t.next_conf_at = fed + ms_to_samples(rate, CONFIRM_RETRY_STEP_MS);
                        }
                    }
                } else {
                    let refractory = t.last_emit != 0
                        && fed.saturating_sub(t.last_emit)
                            < ms_to_samples(rate, PITCH_REFRACTORY_MS);
                    if !t.armed && env < t.peak * 0.6 {
                        t.armed = true;
                    }
                    let bar = (t.slow * ENV_RISE_RATIO).max(ENV_ABS_FLOOR);
                    // A pitch that already emitted only re-triggers on the
                    // heels of a broadband strike (see `last_strike_at`).
                    let restrike_ok = t.last_emit == 0
                        || fed.saturating_sub(self_last_strike) < ms_to_samples(rate, 150);
                    let rise = t.armed && restrike_ok && env > bar && t.slow > 0.0;
                    if rise && !refractory {
                        t.rise_hops = t.rise_hops.saturating_add(1);
                    } else {
                        t.rise_hops = 0;
                    }
                    // Two consecutive hops: one exponential noise fluke can
                    // clear any ratio for a single reading, a struck note
                    // holds its bin up.
                    if t.rise_hops >= 2 {
                        t.rise_hops = 0;
                        // The rise shows once the window mostly covers the
                        // attack: stamp the onset a window back.
                        t.triggered_at =
                            Some(fed.saturating_sub(ENV_WINDOW as u64 + DETECT_HOP as u64));
                        t.next_conf_at = 0;
                        self.debug_log.push(format!(
                            "trig p={} env={env:.2e} slow={:.2e} peak={:.2e}",
                            t.pitch, t.slow, t.peak
                        ));
                    }
                }

                // Follow the level: quickly down (a piano decays and the next
                // rise is measured against the decayed level), slowly up (the
                // attack must not lift its own bar). Adopt the first reading.
                if t.slow == 0.0 {
                    // A fresh tracker is born whenever the gate reaches its
                    // pitch — often WHILE the room still rings from the notes
                    // just played (a rewind, a chord arriving). Adopting that
                    // first reading as "quiet" made the real strike forever
                    // unable to rise 4× above it (on device: after a rewind
                    // only the one pitch whose tracker had survived kept
                    // detecting). Seed a decade under instead: worst case it
                    // triggers on ring and the confirmation stage rejects.
                    t.slow = (env * 0.1).max(1e-12);
                } else if env < t.slow {
                    t.slow = t.slow * 0.80 + env * 0.20;
                } else {
                    t.slow = t.slow * 0.97 + env * 0.03;
                }
                // Peak-hold with a piano-ish decay (~-5 dB/s at 375 hops/s).
                t.peak = (t.peak * 0.997).max(env);
            }
        }

        // Due synthetic releases.
        if !self.pending_offs.is_empty() {
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
}

fn ms_to_samples(rate: u32, ms: u64) -> u64 {
    ms * u64::from(rate) / 1000
}

/// Whether a semitone position sits within one semitone of an expected
/// pitch's fundamental OR its octave partial — i.e., whether a control bin
/// there would measure the chord itself. Both the far probes and the
/// octave-below tattletale went blind to this: a probe for ré landed on
/// sol4 (sol's burning 2nd partial), and si's fifth-check bin at fa♯4 sat a
/// semitone under that same sol4 (Minuet in G, on device).
fn near_expected_partial(semitone_pitch: i16, expected: &[u8]) -> bool {
    expected.iter().any(|&e| {
        let e = i16::from(e);
        (semitone_pitch - e).abs() <= 1 || (semitone_pitch - (e + 12)).abs() <= 1
    })
}

/// MIDI note number → frequency in Hz (equal temperament, A4 = 440).
fn pitch_freq(pitch: u8) -> f64 {
    440.0 * 2f64.powf((f64::from(pitch) - 69.0) / 12.0)
}

/// The evidence window a pitch needs before its presence can be judged:
/// enough cycles for a selective Goertzel reading (~17 periods), floored at
/// the nominal window and capped at [`MAX_EVIDENCE`]. Low pitches therefore
/// confirm later — the documented POC approximation behind
/// [`DETECTION_CONFIRM_NOMINAL_MS`].
fn confirm_window(rate: u32, pitch: u8) -> usize {
    let periods = 17.0 * f64::from(rate) / pitch_freq(pitch);
    (periods as usize)
        .max(ms_to_samples(rate, DETECTION_CONFIRM_NOMINAL_MS) as usize)
        .min(MAX_EVIDENCE)
}

/// Broadband RMS of `buf`.
fn rms(buf: &[f32]) -> f64 {
    if buf.is_empty() {
        return 0.0;
    }
    let sum: f64 = buf.iter().map(|&x| f64::from(x) * f64::from(x)).sum();
    (sum / buf.len() as f64).sqrt()
}

/// Goertzel power of `freq` over a Hann-windowed `buf`. The window is what
/// makes the far-probe test discriminating: a rectangular window's −13 dB
/// side lobes let one played note leak into every nearby expected bin far
/// above the room floor (a synthetic C+E chord confirmed a never-played G);
/// Hann drops the leakage below the probes.
fn goertzel_hann(buf: &[f32], rate: u32, freq: f64) -> f64 {
    let w = 2.0 * std::f64::consts::PI * freq / f64::from(rate);
    let coeff = 2.0 * w.cos();
    let n = buf.len();
    let step = 2.0 * std::f64::consts::PI / (n.max(2) - 1) as f64;
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for (i, &x) in buf.iter().enumerate() {
        let hann = 0.5 * (1.0 - (step * i as f64).cos());
        let s0 = f64::from(x) * hann + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coeff * s1 * s2) / (n as f64)
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

/// Tonal-versus-broadband confirmation against FAR probes. Adjacent-semitone
/// controls looked discriminating but were a whack-a-mole on a real piano:
/// E's control sat on an expected F (the mi/fa bug), and after stepping
/// around expected pitches, G's control at F# still drowned in a *ringing*
/// F's leakage (the sol bug). Probes ±3..±6 semitones out — skipping anything
/// within a semitone of an expected pitch — measure the broadband floor a
/// click or knock would raise, and no sustained neighbor note can sit on
/// them. The trade, accepted for the POC: a wrong note one semitone off leaks
/// into the expected bin and may pass — score-informed presence is not a
/// wrong-note detector (the spec's "unreported extras carry no penalty"
/// covers the reverse direction).
fn pitch_present(buf: &[f32], rate: u32, pitch: u8, expected: &[u8]) -> bool {
    let f = pitch_freq(pitch);
    // A real piano's LOW fundamentals are weak — the energy lives in the
    // partials (played do3 went undetected on device while do2, whose 2nd
    // partial sits exactly on do3's bin, sailed through). Below ~A3 the
    // octave partial joins the evidence.
    let low_register = f < 220.0;
    let fundamental = goertzel_hann(buf, rate, f);
    let signal = if low_register {
        fundamental + goertzel_hann(buf, rate, 2.0 * f)
    } else {
        fundamental
    };
    if signal < 1e-10 {
        return false;
    }
    // Octave-below tattletale: the note an octave down owns every octave bin
    // this pitch has, so no octave-based evidence can tell them apart. Its
    // 3rd partial can: it sits at the FIFTH (1.5f — do2 puts sol3 at 196 Hz
    // where a genuine do3 has nothing). A hot fifth convicts the lower
    // octave — unless that fifth is itself expected (quint chords exist).
    let fifth_pitch = i16::from(pitch) + 7;
    if !near_expected_partial(fifth_pitch, expected) {
        let fifth = goertzel_hann(buf, rate, 1.5 * f);
        // POWER ratio: a lower-octave 3rd partial at half the amplitude of
        // its 2nd is a QUARTER of its power — the bar sits well under that,
        // and three orders of magnitude above the noise a genuine note
        // leaves at its fifth.
        if fifth > signal * 0.08 {
            return false;
        }
    }
    // Tonality gate, scale-free: a sine concentrates ~N/2 × its broadband
    // power into its bin, noise concentrates ~1×. Requiring N/64 leaves a
    // note carrying only a few percent of a chord's power detectable while
    // making a broadband-only window (noise, clicks) unconfirmable — the
    // guard that stops a noise fluke from becoming a phantom note whose
    // refractory then swallows the real strike.
    //
    // Chord-aware: the broadband the gate compares against is discounted by
    // the energy MEASURED at the other expected pitches' bins (fundamental
    // + octave). Against the whole buffer the gate is structurally
    // chord-blind — in sol-si-ré each member holds a fraction of the total,
    // and on device the missing member's sig/bar ratio sat frozen at
    // 0.4–0.55 through every retry (signal and rms decay together; no
    // amount of waiting fixes an energy SHARE). The discount is
    // self-limiting: a click or noise fluke puts nothing in the co-expected
    // bins, so a lone phantom faces the full gate, and the rms²/8 floor
    // keeps a hard bar (8× a noise window's concentration at N=4096) even
    // when the chord mates own the whole buffer.
    let broadband = rms(buf);
    let mut mates = 0.0;
    for &e in expected {
        if e == pitch {
            continue;
        }
        let fe = pitch_freq(e);
        mates += goertzel_hann(buf, rate, fe) + goertzel_hann(buf, rate, 2.0 * fe);
    }
    // A Hann-windowed sine of amplitude A reads A²·N/16 in its bin and
    // contributes A²/2 to rms²: bin → broadband power is 8/N.
    let residual =
        (broadband * broadband - 8.0 * mates / buf.len() as f64).max(broadband * broadband / 8.0);
    if signal < residual * (buf.len() as f64 / 64.0) {
        return false;
    }
    let semitone = 2f64.powf(1.0 / 12.0);
    let mut probes: Vec<f64> = Vec::with_capacity(8);
    // ±5 included: inside a chord, a pitch flanked by expected notes (si in
    // sol-si-ré) loses its ±3/±4 probes to the skip rule and was left with
    // two — and a two-value "median" indexed at len/2 is the MAX, turning
    // the test unfairly strict for exactly that pitch.
    for offset in [-6i16, -5, -4, -3, 3, 4, 5, 6] {
        let probe_pitch = i16::from(pitch) + offset;
        if !(0..=127).contains(&probe_pitch) {
            continue;
        }
        if near_expected_partial(probe_pitch, expected) {
            continue;
        }
        probes.push(goertzel_hann(
            buf,
            rate,
            f * semitone.powi(i32::from(offset)),
        ));
    }
    if probes.len() < 2 {
        // A cluster so dense no probe is clean: absolute evidence alone.
        return signal > 1e-8;
    }
    probes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // Lower median: with an even count this picks the smaller middle value,
    // so a thin probe set degrades lenient rather than strict.
    let median = probes[(probes.len() - 1) / 2].max(1e-12);
    // 2×, not the 6× a sine suggested: a REAL piano note carries hammer
    // noise and partial spread that lift the probes (on-device logs showed
    // honest notes rejected in series at 6×). Broadband transients stay
    // caught by the still-sounding check and the per-bin trigger.
    signal / median > 2.0
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

    /// Deterministic white-ish samples in ±amp (LCG): a REAL broadband
    /// fixture. The earlier ±amp alternation was a pure Nyquist tone — a
    /// pathological spectrum that made floors meaningless.
    fn white(n: usize, amp: f32, seed: u64) -> Vec<f32> {
        let mut state = seed;
        (0..n)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u = ((state >> 33) as f64 / f64::from(1u32 << 31)) - 1.0;
                (u as f32) * amp
            })
            .collect()
    }

    /// `ms` of quiet room noise (well under the absolute floor).
    fn noise(ms: u64) -> Vec<f32> {
        white((ms * u64::from(RATE) / 1000) as usize, 0.004, 7)
    }

    /// `ms` of a loud broadband click-like burst.
    fn burst(ms: u64) -> Vec<f32> {
        white((ms * u64::from(RATE) / 1000) as usize, 0.5, 99)
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
    /// The bed matters: noiseless sines give near-zero far probes, and any
    /// leakage then beats any ratio — a world no microphone lives in.
    fn tone(ms: u64, pitches: &[u8], amp: f32) -> Vec<f32> {
        let n = (ms * u64::from(RATE) / 1000) as usize;
        let bed = white(n, 0.004, 21);
        (0..n)
            .map(|i| {
                let t = i as f64 / f64::from(RATE);
                let s: f64 = pitches
                    .iter()
                    .map(|&p| {
                        let w = 2.0 * std::f64::consts::PI * pitch_freq(p) * t;
                        // Fundamental + a STRONG 2nd partial, like a struck
                        // string (real chords killed probes and tattletales
                        // that a 0.35 partial let pass): nothing real sounds
                        // like a bare sine.
                        (w.sin() + (2.0 * w).sin() * 0.6) * f64::from(amp)
                    })
                    .sum();
                (s as f32) + bed[i]
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
        // The rise shows once the envelope window covers the attack: the
        // stamp is accurate to ~ENV_WINDOW plus a few hops (~25 ms), which
        // the measured input offset absorbs.
        assert!(err < 2400, "onset error: {err} samples");
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

    /// Sample-wise sum of two signals (they must be the same length).
    fn mix(a: &[f32], b: &[f32]) -> Vec<f32> {
        a.iter().zip(b).map(|(x, y)| x + y).collect()
    }

    #[test]
    fn chord_member_masked_by_its_own_strike_confirms_on_retry() {
        let mut d = NoteDetector::new(RATE);
        // The on-device chord (Minuet in G): sol3-si3-ré4. si is the
        // vulnerable member — above the low-register boundary (no octave
        // help) and quieter than ré, its one confirmation shot landed inside
        // the hammer transient, where the broadband floor buries its share
        // of the chord, and a decaying note never re-triggers.
        d.set_expected(vec![55, 59, 62]);
        let mut events = d.feed(&noise(200));
        // The strike: unequal chord voicing under a loud broadband burst.
        let n = (80 * u64::from(RATE) / 1000) as usize;
        let attack = mix(
            &mix(&tone(80, &[62], 0.22), &tone(80, &[55, 59], 0.16)),
            &white(n, 0.4, 3),
        );
        events.extend(d.feed(&attack));
        // The hammer noise gone, the chord keeps ringing: the retries get a
        // clean window.
        events.extend(d.feed(&mix(&tone(600, &[62], 0.22), &tone(600, &[55, 59], 0.16))));

        let mut on: Vec<u8> = events.iter().filter(|e| e.on).map(|e| e.pitch).collect();
        on.sort_unstable();
        assert_eq!(on, vec![55, 59, 62], "events: {events:?}");
    }

    #[test]
    fn quiet_chord_member_holds_a_small_energy_share_and_still_confirms() {
        let mut d = NoteDetector::new(RATE);
        // The v24 on-device failure: si's fundamental held well under an
        // eighth of the chord's power, so the un-discounted tonality gate
        // rejected it at EVERY retry — sig and rms decay together, the
        // ratio sat frozen at 0.4–0.55 for the whole budget. No transient
        // needed: the share alone kills it.
        d.set_expected(vec![55, 59, 62]);
        let mut events = d.feed(&noise(200));
        events.extend(d.feed(&mix(
            &mix(&tone(700, &[62], 0.28), &tone(700, &[55], 0.16)),
            &tone(700, &[59], 0.09),
        )));

        let mut on: Vec<u8> = events.iter().filter(|e| e.on).map(|e| e.pitch).collect();
        on.sort_unstable();
        assert_eq!(on, vec![55, 59, 62], "events: {events:?}");
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
    fn adjacent_expected_semitones_both_detect() {
        // E4 + F4 expected AND both sounding: E's +1-semitone control sits ON
        // F, which vetoed every E before the controls learned to step around
        // expected pitches (the mi/fa bug from the on-device pass).
        const F4: u8 = 65;
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![E4, F4]);
        let events = play(&mut d, &[E4, F4], 400);

        let mut on_pitches: Vec<u8> = events.iter().filter(|e| e.on).map(|e| e.pitch).collect();
        on_pitches.sort_unstable();
        assert_eq!(on_pitches, vec![E4, F4], "events: {events:?}");
    }

    #[test]
    fn sol_detects_while_fa_rings() {
        // E-F-G all expected, F and G sounding together (F under sustain):
        // adjacent-semitone controls put G's control on F# where the ringing
        // F leaks — far probes must not care.
        const F4: u8 = 65;
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![E4, F4, G4]);
        let events = play(&mut d, &[F4, G4], 400);

        let mut on_pitches: Vec<u8> = events.iter().filter(|e| e.on).map(|e| e.pitch).collect();
        on_pitches.sort_unstable();
        assert_eq!(on_pitches, vec![F4, G4], "events: {events:?}");
    }

    /// A low piano note: weak fundamental, strong 2nd partial, present 3rd —
    /// how real strings below ~C3 measure at a microphone.
    fn low_tone(ms: u64, pitch: u8, amp: f32) -> Vec<f32> {
        let n = (ms * u64::from(RATE) / 1000) as usize;
        let bed = white(n, 0.004, 33);
        let f = pitch_freq(pitch);
        (0..n)
            .map(|i| {
                let t = i as f64 / f64::from(RATE);
                let w = 2.0 * std::f64::consts::PI * f * t;
                let s = w.sin() * 0.15 + (2.0 * w).sin() * 1.0 + (3.0 * w).sin() * 0.5;
                (s * f64::from(amp)) as f32 + bed[i]
            })
            .collect()
    }

    #[test]
    fn weak_fundamental_low_note_detects() {
        // do3 played for a do3 gate: the fundamental is weak but the octave
        // partial carries it (the on-device report: do3 undetectable).
        const C3: u8 = 48;
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![C3]);
        let mut events = d.feed(&noise(200));
        events.extend(d.feed(&low_tone(500, C3, 0.3)));
        let ons: Vec<_> = events.iter().filter(|e| e.on).collect();
        assert_eq!(ons.len(), 1, "events: {events:?}");
        assert_eq!(ons[0].pitch, C3);
    }

    #[test]
    fn octave_below_does_not_impersonate() {
        // do2 played for a do3 gate: do2's 2nd partial sits exactly on do3's
        // bin — but its 3rd partial lights the fifth (sol3), where a real
        // do3 has nothing. The tattletale must reject.
        const C3: u8 = 48;
        const C2: u8 = 36;
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![C3]);
        let mut events = d.feed(&noise(200));
        events.extend(d.feed(&low_tone(500, C2, 0.3)));
        assert!(events.iter().all(|e| !e.on), "events: {events:?}");
    }

    #[test]
    fn chord_middle_pitch_flanked_by_expected_detects() {
        // sol-si-ré (Minuet in G): si's ±3/±4 probes all land on expected
        // chordmates and are skipped — the thinned probe set must degrade
        // lenient, not strict (on device, si went undetected in the chord
        // while working alone).
        const G3: u8 = 55;
        const B3: u8 = 59;
        const D4: u8 = 62;
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![G3, B3, D4]);
        let events = play(&mut d, &[G3, B3, D4], 400);
        let mut on_pitches: Vec<u8> = events.iter().filter(|e| e.on).map(|e| e.pitch).collect();
        on_pitches.sort_unstable();
        assert_eq!(on_pitches, vec![G3, B3, D4], "events: {events:?}");
    }

    #[test]
    fn gate_arriving_on_a_ringing_pitch_still_detects_the_restrike() {
        // The rewind case: the gate returns to a pitch whose string still
        // rings from the previous pass. The tracker created at that moment
        // must not adopt the ring as its quiet baseline.
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![G4]); // some other gate first
        let mut events = d.feed(&noise(200));
        events.extend(d.feed(&tone(400, &[A4], 0.3))); // A4 rings, untracked
        d.set_expected(vec![A4]); // rewind: gate returns to A4 mid-ring
        events.extend(d.feed(&tone(200, &[A4], 0.3))); // ring continues
        events.extend(d.feed(&tone(400, &[A4], 0.55))); // the RE-STRIKE
        let ons: Vec<_> = events.iter().filter(|e| e.on).collect();
        assert!(
            ons.iter().any(|e| e.pitch == A4),
            "restrike went undetected: {events:?}"
        );
    }

    #[test]
    fn wrong_semitone_neighbor_does_not_pass() {
        // C4 expected, B3 played (si for do, the on-device report): B leaks
        // into C's bin enough to trigger and probe-pass — the unexpected-
        // neighbor comparison is what says "the louder bin is the played
        // one, and it is not the expected one".
        const B3: u8 = 59;
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![C4]);
        let events = play(&mut d, &[B3], 400);
        assert!(events.iter().all(|e| !e.on), "events: {events:?}");
    }

    #[test]
    fn repeated_same_note_emits_twice() {
        // mi-mi, the opening of half the repertoire: the first strike rings,
        // decays a little, and the SAME pitch is struck again. The re-arm
        // hysteresis plus the broadband-strike gate must let the repeat
        // through (a beat between ringing neighbors must not — see
        // adjacent_expected_semitones_both_detect).
        let mut d = NoteDetector::new(RATE);
        d.set_expected(vec![E4]);
        let mut events = play(&mut d, &[E4], 300);
        events.extend(d.feed(&tone(250, &[E4], 0.08)));
        events.extend(d.feed(&tone(300, &[E4], 0.35)));

        let ons: Vec<_> = events.iter().filter(|e| e.on).collect();
        assert_eq!(ons.len(), 2, "events: {events:?}");
        assert!(ons.iter().all(|e| e.pitch == E4));
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
    fn selection_resolves_exact_name_or_falls_back() {
        let available = vec![
            "Scarlett 2i2".to_string(),
            "MacBook Pro Microphone".to_string(),
        ];
        assert_eq!(
            resolve_input_device(Some("Scarlett 2i2"), &available),
            Some("Scarlett 2i2")
        );
        // Absent device → None = system default, never a failure.
        assert_eq!(
            resolve_input_device(Some("Unplugged Mic"), &available),
            None
        );
        // No request → system default.
        assert_eq!(resolve_input_device(None, &available), None);
        // Display names must match exactly, never fuzzily.
        assert_eq!(resolve_input_device(Some("Scarlett"), &available), None);
    }

    #[test]
    fn low_pitch_uses_a_longer_confirmation_window() {
        // A2 (110 Hz) needs ~17 periods; A5 (880 Hz) sits on the floor.
        assert!(confirm_window(RATE, 45) > confirm_window(RATE, 81));
        assert!(confirm_window(RATE, 45) <= MAX_EVIDENCE);
    }
}
