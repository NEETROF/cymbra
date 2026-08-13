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

//! Android's output path: the engine renders, Kotlin's `AudioTrack` plays.
//!
//! Why not `cpal` here, when it serves every other platform? Because measured on
//! device its AAudio path does two things that make it unusable for this app:
//!
//!  - it cannot enumerate Android's outputs (its JNI request fails and it falls
//!    back to a single placeholder device named "Default Device"), so the user can
//!    never be offered the USB-audio piano they plugged in;
//!  - and its stream reports itself healthy while delivering almost nothing to
//!    that route.
//!
//! A bare `AudioTrack` fed from a writer thread keeps perfect time on the same
//! route (measured: +15 ms over 24 s, the producer held at a constant ~120 ms
//! ahead), enumerates every output, and can pin playback to a chosen one with
//! `setPreferredDevice`. So on Android the platform owns the stream and calls
//! *down* into the engine for samples, which is the inverse of the `cpal` model.
//!
//! `Renderer` is shared with the `cpal` path, so the synth, the voice
//! bookkeeping, the metronome and the preview clips behave identically here.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use jni::JNIEnv;
use jni::objects::{JClass, JShortArray};
use jni::sys::{jboolean, jint};
use rustysynth::SoundFont;

use super::platform_log;
use super::renderer::{RenderCommand, Renderer};

/// The mixer and its command queue, installed by `audio_init` and driven by the
/// Kotlin writer thread. `None` until the SoundFont has been parsed.
static OUTPUT: Mutex<Option<AndroidOutput>> = Mutex::new(None);

/// The sample rate the platform wants rendered, set by Kotlin **before** it
/// opens its `AudioTrack` ([`nativeSetRate`]). Rendering at the route's own
/// rate spares an AudioFlinger resample stage — the output thread runs at
/// whatever the HAL negotiated (48 kHz on some devices, 44.1 kHz on others),
/// and a track at any other rate is resampled to it.
static DESIRED_RATE: AtomicU32 = AtomicU32::new(44_100);

struct AndroidOutput {
    /// Kept so the renderer can be rebuilt when the platform changes rate.
    sound_font: Arc<SoundFont>,
    renderer: Renderer,
    queue: Receiver<RenderCommand>,
}

/// Frames pulled and the loudest sample seen, reported periodically.
///
/// The one fact that separates "the sound goes to the wrong place" from "there is
/// no sound to place": a pipeline that is running but rendering pure silence
/// looks identical from the outside to one that is correctly routed and mute.
static PULLED_FRAMES: AtomicU64 = AtomicU64::new(0);
static PEAK_MILLI: AtomicU64 = AtomicU64::new(0);
static LAST_REPORT_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Worst single-block render time in the current report window, in microseconds.
/// The one number that says whether the synth (debug builds are ~10× slower)
/// ever comes close to the ~23 ms real-time budget of a block — the difference
/// between "the pipeline crackles" and "the render is the crackle".
static MAX_RENDER_MICROS: AtomicU64 = AtomicU64::new(0);

/// Installs the mixer for the Kotlin side to pull from, rendering at the rate
/// the platform asked for. Replaces any previous one, so a re-init is safe.
pub(crate) fn install(
    sound_font: Arc<SoundFont>,
    queue: Receiver<RenderCommand>,
) -> Result<(), String> {
    let rate = DESIRED_RATE.load(Ordering::SeqCst);
    let renderer = Renderer::new(&sound_font, rate as i32).map_err(|e| e.to_string())?;
    super::audio::set_stream_rate(rate);
    *OUTPUT.lock().unwrap() = Some(AndroidOutput {
        sound_font,
        renderer,
        queue,
    });
    platform_log::log_line(
        "cymbra-audio",
        &format!("android output installed (AudioTrack path, {rate} Hz)"),
    );
    Ok(())
}

/// Sets the render rate. Called by Kotlin before opening its `AudioTrack`; if
/// the mixer is already installed at another rate, it is rebuilt (any sounding
/// voices, click or preview clip are dropped — this only happens while the
/// output itself is being reopened, so nothing audible survives anyway).
///
/// # Safety
/// Called from the JVM only.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cymbra_music_EngineOutput_nativeSetRate(
    _env: JNIEnv,
    _class: JClass,
    rate: jint,
) {
    let rate = if rate > 0 { rate as u32 } else { 44_100 };
    DESIRED_RATE.store(rate, Ordering::SeqCst);
    let Ok(mut guard) = OUTPUT.lock() else { return };
    let Some(output) = guard.as_mut() else { return };
    if output.renderer.sample_rate() == rate as i32 {
        return;
    }
    match Renderer::new(&output.sound_font, rate as i32) {
        Ok(renderer) => {
            output.renderer = renderer;
            super::audio::set_stream_rate(rate);
            platform_log::log_line("cymbra-audio", &format!("render rate → {rate} Hz"));
        }
        Err(e) => platform_log::log_line(
            "cymbra-audio",
            &format!("rate change to {rate} Hz failed, keeping current: {e}"),
        ),
    }
}

/// Whether the engine is ready to be pulled from — i.e. the SoundFont is parsed.
///
/// # Safety
/// Called from the JVM only.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cymbra_music_EngineOutput_nativeIsReady(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    u8::from(OUTPUT.lock().map(|o| o.is_some()).unwrap_or(false))
}

/// Renders `frames` interleaved stereo frames of 16-bit PCM into `out`.
///
/// Returns the number of frames written, or 0 when the engine is not ready — the
/// caller then writes silence rather than dropping the stream. Called from
/// Kotlin's writer thread, never from a real-time callback, so taking the mixer's
/// lock here is safe.
///
/// # Safety
/// Called from the JVM only; `out` must hold at least `frames * 2` shorts.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cymbra_music_EngineOutput_nativeRender(
    env: JNIEnv,
    _class: JClass,
    out: JShortArray,
    frames: jint,
) -> jint {
    let frames = frames.max(0) as usize;
    if frames == 0 {
        return 0;
    }
    let Ok(mut guard) = OUTPUT.lock() else {
        return 0;
    };
    let Some(output) = guard.as_mut() else {
        return 0;
    };

    let render_started = std::time::Instant::now();
    output.renderer.drain(&output.queue);
    let (left, right) = output.renderer.render(frames);
    let render_micros = render_started.elapsed().as_micros() as u64;
    MAX_RENDER_MICROS.fetch_max(render_micros, Ordering::Relaxed);

    // Interleave into 16-bit PCM, clipped rather than wrapped: a wrapped sample is
    // a loud click, a clipped one is merely loud.
    let mut interleaved = vec![0i16; frames * 2];
    for i in 0..frames {
        interleaved[i * 2] = to_pcm16(left[i]);
        interleaved[i * 2 + 1] = to_pcm16(right[i]);
    }
    // Peak of this block, in thousandths of full scale.
    let peak = interleaved
        .iter()
        .map(|s| (s.unsigned_abs() as u64) * 1000 / (i16::MAX as u64))
        .max()
        .unwrap_or(0);
    PEAK_MILLI.fetch_max(peak, Ordering::Relaxed);
    let pulled = PULLED_FRAMES.fetch_add(frames as u64, Ordering::Relaxed) + frames as u64;
    // Feed the engine's drift monitor: this pull path replaces the cpal callback
    // on Android, so it must carry the same "are we keeping time" probe — the one
    // signal that exposed the AAudio mis-clocking this module exists to escape.
    super::audio::note_frames_rendered(frames as u64);

    // Roughly every two seconds at 44.1 kHz.
    if pulled - LAST_REPORT_FRAMES.load(Ordering::Relaxed) >= 88_200 {
        LAST_REPORT_FRAMES.store(pulled, Ordering::Relaxed);
        let peak = PEAK_MILLI.swap(0, Ordering::Relaxed);
        let max_render = MAX_RENDER_MICROS.swap(0, Ordering::Relaxed);
        platform_log::log_line(
            "cymbra-audio",
            &format!(
                "pulled {pulled} frames — peak {peak}/1000, worst render {max_render}µs \
                 (budget ~23000µs/block)"
            ),
        );
    }

    match env.set_short_array_region(&out, 0, &interleaved) {
        Ok(()) => frames as jint,
        Err(_) => 0,
    }
}

/// Converts a rendered sample to 16-bit PCM, clamped to the representable range.
fn to_pcm16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}
