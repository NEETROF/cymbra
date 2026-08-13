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

//! The mixer: everything that turns queued commands into interleaved samples.
//!
//! It used to live inside the `cpal` callback closure, which meant only `cpal`
//! could ever drive it. Android needs a second driver — the platform's own
//! `AudioTrack`, because `cpal`'s AAudio path neither enumerates the device's
//! outputs nor delivers usable audio to a USB-audio instrument there — so the
//! synth, the voice bookkeeping, the metronome click and the preview clip now sit
//! behind one type that either side can call.
//!
//! Not pure (it owns a `Synthesizer`), so it stays out of the coverage gate like
//! the rest of the engine glue.

use std::sync::mpsc::Receiver;

use anyhow::{Result, anyhow};
use flutter_rust_bridge::frb;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::sync::Arc;

use super::audio_core::{AudioEvent, ClickVoice, ClipVoice, PIANO_CHANNEL, VoiceTracker};
use super::platform_log;

/// A command handed to the mixer. Mirrors what the FFI entry points push.
#[frb(ignore)]
pub(crate) enum RenderCommand {
    /// A note/all-off/click control event.
    Control(AudioEvent),
    /// Replace the active synthesizer, silencing every voice across the swap.
    ReplaceSynth(Arc<SoundFont>),
    /// Start (or replace) a looping preview clip mixed on top of the synth.
    PlayClip { pcm: Vec<i16>, sample_rate: u32 },
    /// Stop any preview clip currently playing.
    StopClip,
}

/// Synth + voice bookkeeping + click + preview clip, rendering at one rate.
#[frb(ignore)]
pub(crate) struct Renderer {
    synth: Synthesizer,
    tracker: VoiceTracker,
    /// The sounding metronome click, if any: a short one-shot mixed on top of the
    /// synth, replaced outright by a new click.
    click: Option<ClickVoice>,
    /// The looping preview clip, if any — mixed like the click but until stopped.
    clip: Option<ClipVoice>,
    sample_rate: i32,
    /// Scratch buffers, so the steady path never allocates.
    left: Vec<f32>,
    right: Vec<f32>,
}

impl Renderer {
    pub(crate) fn new(sound_font: &Arc<SoundFont>, sample_rate: i32) -> Result<Renderer> {
        let settings = SynthesizerSettings::new(sample_rate);
        let synth =
            Synthesizer::new(sound_font, &settings).map_err(|e| anyhow!("synth init: {e}"))?;
        Ok(Renderer {
            synth,
            tracker: VoiceTracker::new(),
            click: None,
            clip: None,
            sample_rate,
            left: Vec::new(),
            right: Vec::new(),
        })
    }

    /// The rate this renderer produces samples at.
    #[cfg(target_os = "android")]
    pub(crate) fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Applies every queued command in FIFO order, so notes keep their ordering
    /// relative to a SoundFont swap or a clip start.
    pub(crate) fn drain(&mut self, queue: &Receiver<RenderCommand>) {
        while let Ok(command) = queue.try_recv() {
            self.apply(command);
        }
    }

    fn apply(&mut self, command: RenderCommand) {
        match command {
            RenderCommand::Control(event) => match event {
                AudioEvent::NoteOn { pitch, velocity } => {
                    self.tracker.apply(event);
                    self.synth
                        .note_on(PIANO_CHANNEL, pitch as i32, velocity as i32);
                }
                AudioEvent::NoteOff { .. } => {
                    for pitch in self.tracker.apply(event) {
                        self.synth.note_off(PIANO_CHANNEL, pitch as i32);
                    }
                }
                AudioEvent::AllOff => {
                    self.tracker.apply(event);
                    self.synth.note_off_all(true);
                }
                AudioEvent::Click { accent } => {
                    self.click = Some(ClickVoice::new(accent, self.sample_rate as f32));
                }
            },
            // Silence every voice across the swap (the tracker mirror and the
            // outgoing synth), then install the new instrument. A failed build
            // keeps the current one so audio never drops out.
            RenderCommand::ReplaceSynth(sound_font) => {
                self.tracker.clear_for_swap();
                self.synth.note_off_all(true);
                let settings = SynthesizerSettings::new(self.sample_rate);
                match Synthesizer::new(&sound_font, &settings) {
                    Ok(next) => self.synth = next,
                    Err(e) => platform_log::log_line(
                        "cymbra-audio",
                        &format!("swap synth build failed, keeping current: {e}"),
                    ),
                }
            }
            RenderCommand::PlayClip { pcm, sample_rate } => {
                self.clip = Some(ClipVoice::new(pcm, sample_rate, self.sample_rate as f32));
            }
            RenderCommand::StopClip => self.clip = None,
        }
    }

    /// Renders `frames` frames and returns the stereo pair. The click and the clip
    /// are mixed on top of the synth, both as the same mono signal in each
    /// channel; a finished click is dropped so the steady path does nothing.
    pub(crate) fn render(&mut self, frames: usize) -> (&[f32], &[f32]) {
        if self.left.len() < frames {
            self.left.resize(frames, 0.0);
            self.right.resize(frames, 0.0);
        }
        let l = &mut self.left[..frames];
        let r = &mut self.right[..frames];
        self.synth.render(l, r);

        if let Some(voice) = self.click.as_mut() {
            for i in 0..frames {
                let s = voice.next_sample();
                l[i] += s;
                r[i] += s;
            }
            if !voice.is_active() {
                self.click = None;
            }
        }
        if let Some(voice) = self.clip.as_mut() {
            for i in 0..frames {
                let s = voice.next_sample();
                l[i] += s;
                r[i] += s;
            }
        }
        (&self.left[..frames], &self.right[..frames])
    }
}
