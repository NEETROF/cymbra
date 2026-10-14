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

//! Offline sound for the showcase video (change: add-site-music-showcase, D6).
//!
//! The capture harness renders the video's frames on a virtual clock and logs, on
//! that same clock, every command the app sends to its sound engine. This module
//! replays that log through the app's own mixer ([`Renderer`]) into a WAV file, so
//! the video's sound is the app's sound to the sample and lands on the frame that
//! shows it — no recording, no clock to line up afterwards.
//!
//! Not part of the FFI surface: it lives outside `api/`, so the bridge codegen
//! never sees it. Only the `showcase-render` tool calls it.
//!
//! The log is one command per line, `<ms> <verb> [args]`, `#` comments allowed:
//!
//! ```text
//! 0 font UprightPianoKW-20220221.sf2
//! 1250.0 on 64 96
//! 1583.3 off 64
//! 1600 drum-on 36 110
//! 1700 drum-off 36
//! 2000 all-off
//! 2000 click 1
//! 26000 end
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;

use anyhow::{Context, Result, anyhow, bail};
use rustysynth::SoundFont;

use crate::api::audio_core::AudioEvent;
use crate::api::renderer::{RenderCommand, Renderer};

/// One command of the log, without its time.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Install the SoundFont with this file name (the app swaps its whole synth).
    Font(String),
    NoteOn {
        pitch: u8,
        velocity: u8,
    },
    NoteOff {
        pitch: u8,
    },
    DrumOn {
        key: u8,
        velocity: u8,
    },
    DrumOff {
        key: u8,
    },
    AllOff,
    Click {
        accent: bool,
    },
    /// The end of the video: the render stops here.
    End,
}

/// A command stamped with its time on the video's clock, in milliseconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Timed {
    pub ms: f64,
    pub command: Command,
}

/// Parses the log. Commands come out in time order (stable for equal times, so a
/// release logged before a press at the same instant stays before it).
pub fn parse_log(text: &str) -> Result<Vec<Timed>> {
    let mut out = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let at = || format!("line {}: {raw:?}", index + 1);
        let mut parts = line.split_whitespace();
        let ms: f64 = parts.next().unwrap_or_default().parse().with_context(at)?;
        if !ms.is_finite() || ms < 0.0 {
            bail!("negative or invalid time, {}", at());
        }
        let verb = parts
            .next()
            .ok_or_else(|| anyhow!("no command, {}", at()))?;
        let args: Vec<&str> = parts.collect();
        let byte = |i: usize| -> Result<u8> {
            let value: u8 = args
                .get(i)
                .ok_or_else(|| anyhow!("missing argument {}, {}", i + 1, at()))?
                .parse()
                .with_context(at)?;
            if value > 127 {
                bail!("{value} is outside 0..=127, {}", at());
            }
            Ok(value)
        };
        let command = match verb {
            "font" => Command::Font(
                args.first()
                    .ok_or_else(|| anyhow!("missing file name, {}", at()))?
                    .to_string(),
            ),
            "on" => Command::NoteOn {
                pitch: byte(0)?,
                velocity: byte(1)?,
            },
            "off" => Command::NoteOff { pitch: byte(0)? },
            "drum-on" => Command::DrumOn {
                key: byte(0)?,
                velocity: byte(1)?,
            },
            "drum-off" => Command::DrumOff { key: byte(0)? },
            "all-off" => Command::AllOff,
            "click" => Command::Click {
                accent: byte(0)? != 0,
            },
            "end" => Command::End,
            other => bail!("unknown command {other:?}, {}", at()),
        };
        out.push(Timed { ms, command });
    }
    out.sort_by(|a, b| a.ms.total_cmp(&b.ms));
    Ok(out)
}

/// The length of the video the log describes: its `end` command, which every
/// log must carry — without it the sound would stop on the last release and run
/// shorter than the picture.
pub fn end_ms(log: &[Timed]) -> Result<f64> {
    log.iter()
        .find(|t| t.command == Command::End)
        .map(|t| t.ms)
        .ok_or_else(|| anyhow!("the log has no `end` command"))
}

/// The sample index a time falls on, at `sample_rate`.
pub fn sample_at(ms: f64, sample_rate: u32) -> usize {
    (ms * f64::from(sample_rate) / 1000.0).round() as usize
}

/// Encodes interleaved stereo `f32` samples as a 16-bit PCM WAV file.
pub fn encode_wav(left: &[f32], right: &[f32], sample_rate: u32) -> Vec<u8> {
    let frames = left.len().min(right.len());
    let data_len = (frames * 4) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&2u16.to_le_bytes()); // stereo
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 4).to_le_bytes()); // byte rate
    out.extend_from_slice(&4u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..frames {
        for s in [left[i], right[i]] {
            let v = (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

/// Replays `log` through the app's mixer and returns the stereo signal, from the
/// log's time 0 to its `end`. SoundFonts are looked up by file name in
/// `soundfonts`; the log's first `font` command must come before its first note.
pub fn render(log: &[Timed], soundfonts: &Path, sample_rate: u32) -> Result<(Vec<f32>, Vec<f32>)> {
    let total = sample_at(end_ms(log)?, sample_rate);
    let mut fonts: HashMap<String, Arc<SoundFont>> = HashMap::new();
    let mut load = |name: &str| -> Result<Arc<SoundFont>> {
        if let Some(font) = fonts.get(name) {
            return Ok(font.clone());
        }
        let path: PathBuf = soundfonts.join(name);
        let mut file =
            std::fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
        let font = Arc::new(
            SoundFont::new(&mut file).map_err(|e| anyhow!("parse {}: {e}", path.display()))?,
        );
        fonts.insert(name.to_string(), font.clone());
        Ok(font)
    };

    let mut renderer: Option<Renderer> = None;
    let (tx, rx) = mpsc::channel();
    let mut left = Vec::with_capacity(total);
    let mut right = Vec::with_capacity(total);

    for timed in log {
        let at = sample_at(timed.ms, sample_rate).min(total);
        if at > left.len() {
            match renderer.as_mut() {
                Some(r) => {
                    let (l, rr) = r.render(at - left.len());
                    left.extend_from_slice(l);
                    right.extend_from_slice(rr);
                }
                // Nothing can sound before the first font: silence.
                None => {
                    left.resize(at, 0.0);
                    right.resize(at, 0.0);
                }
            }
        }
        let event = match &timed.command {
            Command::End => break,
            Command::Font(name) => {
                let font = load(name)?;
                match renderer.as_mut() {
                    None => renderer = Some(Renderer::new(&font, sample_rate as i32)?),
                    Some(_) => {
                        tx.send(RenderCommand::ReplaceSynth(font, None))?;
                    }
                }
                None
            }
            Command::NoteOn { pitch, velocity } => Some(AudioEvent::note_on(*pitch, *velocity)),
            Command::NoteOff { pitch } => Some(AudioEvent::note_off(*pitch)),
            Command::DrumOn { key, velocity } => Some(AudioEvent::drum_on(*key, *velocity)),
            Command::DrumOff { key } => Some(AudioEvent::drum_off(*key)),
            Command::AllOff => Some(AudioEvent::AllOff),
            Command::Click { accent } => Some(AudioEvent::Click { accent: *accent }),
        };
        let Some(r) = renderer.as_mut() else {
            if event.is_some() {
                bail!("a note at {} ms comes before the first `font`", timed.ms);
            }
            continue;
        };
        if let Some(event) = event {
            tx.send(RenderCommand::Control(event))?;
        }
        r.drain(&rx);
    }
    if total > left.len() {
        let r = renderer
            .as_mut()
            .ok_or_else(|| anyhow!("the log never installs a font"))?;
        let (l, rr) = r.render(total - left.len());
        left.extend_from_slice(l);
        right.extend_from_slice(rr);
    }
    Ok((left, right))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn soundfonts() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/soundfonts")
    }

    #[test]
    fn parses_every_command_in_time_order() {
        let log = parse_log(
            "# header\n\
             0 font Piano.sf2\n\
             20 off 60   # trailing comment\n\
             10.5 on 60 96\n\
             30 drum-on 36 110\n\
             31 drum-off 36\n\
             40 all-off\n\
             41 click 1\n\
             42 click 0\n\
             50 end\n",
        )
        .unwrap();
        let commands: Vec<_> = log.iter().map(|t| (t.ms, t.command.clone())).collect();
        assert_eq!(
            commands,
            vec![
                (0.0, Command::Font("Piano.sf2".into())),
                (
                    10.5,
                    Command::NoteOn {
                        pitch: 60,
                        velocity: 96
                    }
                ),
                (20.0, Command::NoteOff { pitch: 60 }),
                (
                    30.0,
                    Command::DrumOn {
                        key: 36,
                        velocity: 110
                    }
                ),
                (31.0, Command::DrumOff { key: 36 }),
                (40.0, Command::AllOff),
                (41.0, Command::Click { accent: true }),
                (42.0, Command::Click { accent: false }),
                (50.0, Command::End),
            ]
        );
    }

    #[test]
    fn equal_times_keep_their_logged_order() {
        let log = parse_log("5 off 60\n5 on 60 90\n").unwrap();
        assert_eq!(log[0].command, Command::NoteOff { pitch: 60 });
        assert_eq!(
            log[1].command,
            Command::NoteOn {
                pitch: 60,
                velocity: 90
            }
        );
    }

    #[test]
    fn rejects_malformed_lines() {
        for bad in [
            "x on 60 90",
            "-1 on 60 90",
            "10",
            "10 on 60",
            "10 on 200 90",
            "10 hum",
            "10 font",
            "inf end",
        ] {
            assert!(parse_log(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn end_is_required() {
        assert!(end_ms(&parse_log("0 font a.sf2\n").unwrap()).is_err());
        assert_eq!(
            end_ms(&parse_log("0 font a.sf2\n9 end\n").unwrap()).unwrap(),
            9.0
        );
    }

    #[test]
    fn sample_index_follows_the_rate() {
        assert_eq!(sample_at(0.0, 48_000), 0);
        assert_eq!(sample_at(1000.0, 48_000), 48_000);
        assert_eq!(sample_at(33.333, 48_000), 1600);
    }

    #[test]
    fn wav_header_and_samples() {
        let wav = encode_wav(&[0.0, 1.0, -2.0], &[0.5, -1.0, 0.0], 48_000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 48_000);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 12);
        assert_eq!(wav.len(), 44 + 12);
        let sample = |i: usize| i16::from_le_bytes(wav[44 + 2 * i..46 + 2 * i].try_into().unwrap());
        assert_eq!(sample(0), 0);
        assert_eq!(sample(1), 16384);
        assert_eq!(sample(2), i16::MAX);
        assert_eq!(sample(4), -i16::MAX); // clamped
    }

    #[test]
    fn renders_the_app_piano_on_time() {
        let log = parse_log(
            "0 font UprightPianoKW-20220221.sf2\n\
             500 on 60 100\n\
             900 off 60\n\
             1200 end\n",
        )
        .unwrap();
        let rate = 48_000;
        let (left, right) = render(&log, &soundfonts(), rate).unwrap();
        assert_eq!(left.len(), sample_at(1200.0, rate));
        assert_eq!(right.len(), left.len());
        let peak = |from: usize, to: usize| left[from..to].iter().fold(0f32, |m, s| m.max(s.abs()));
        // Silent before the press, sounding right after it.
        assert!(peak(0, sample_at(495.0, rate)) < 1e-4);
        assert!(peak(sample_at(500.0, rate), sample_at(600.0, rate)) > 0.01);
    }

    #[test]
    fn a_note_before_any_font_is_an_error() {
        let log = parse_log("0 on 60 90\n10 end\n").unwrap();
        assert!(render(&log, &soundfonts(), 48_000).is_err());
    }

    #[test]
    fn a_missing_font_is_an_error() {
        let log = parse_log("0 font Nope.sf2\n10 end\n").unwrap();
        assert!(render(&log, &soundfonts(), 48_000).is_err());
    }

    #[test]
    fn a_font_swap_mid_log_keeps_rendering() {
        let log = parse_log(
            "0 font UprightPianoKW-20220221.sf2\n\
             100 on 60 100\n\
             200 font FluidR3Drums-bank128.sf2\n\
             300 drum-on 36 110\n\
             400 end\n",
        )
        .unwrap();
        let (left, _) = render(&log, &soundfonts(), 48_000).unwrap();
        assert_eq!(left.len(), sample_at(400.0, 48_000));
    }
}
