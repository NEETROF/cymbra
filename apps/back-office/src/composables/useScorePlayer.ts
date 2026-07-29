// Drives audio playback + the playhead clock for the score preview, reusing the app's
// engine: `schedule(bytes)` (wasm) for note/measure times, and `render(bytes, sf2)`
// (wasm rustysynth) for the PCM, played through Web Audio. Play/Pause only — no other
// interaction. Everything degrades to a state (never throws): no AudioContext, a
// failed SoundFont fetch, or a render error surface as `audio = error`.

import { computed, onScopeDispose, ref, watch, type Ref } from "vue";
import { type Async, failure, idle, loading, success } from "@/lib/async";
import type { PlaybackSchedule } from "@/lib/notation/schedule";
import { loadNotationWasm } from "@/lib/notation/wasm";
import { loadAudioWasm } from "@/lib/audio/synth";
import { loadSoundFont } from "@/lib/audio/soundfont";
import { useAuthStore } from "@/stores/auth";

interface ScorePlayer {
  /** Schedule for the playhead (measure times, sounding notes) — Async. */
  schedule: Ref<Async<PlaybackSchedule>>;
  /** Audio preparation state (loading while the SoundFont/render runs; error if it fails). */
  audio: Ref<Async<void>>;
  playing: Ref<boolean>;
  elapsedMs: Ref<number>;
  canPlay: Ref<boolean>;
  toggle: () => void;
  stop: () => void;
  /** Start playback from an absolute time (ms) — e.g. a clicked measure's start. */
  playFrom: (ms: number) => void;
}

type Ctx = typeof AudioContext;

export function useScorePlayer(bytes: Ref<Uint8Array | null | undefined>): ScorePlayer {
  const schedule = ref<Async<PlaybackSchedule>>(idle);
  const audio = ref<Async<void>>(idle);
  const playing = ref(false);
  const elapsedMs = ref(0);
  const canPlay = computed(() => schedule.value.status === "success" && bytes.value != null);

  let ctx: AudioContext | null = null;
  let buffer: AudioBuffer | null = null; // rendered PCM, cached per bytes
  let source: AudioBufferSourceNode | null = null;
  let raf = 0;
  let startCtxTime = 0; // ctx.currentTime when the current source started
  let offsetSec = 0; // where playback resumes from (pause point)
  let playDurationMs = 0;

  // Derive the schedule (for the playhead) whenever the bytes change, and reset audio.
  watch(
    bytes,
    async (value) => {
      stop();
      buffer = null;
      if (value == null) {
        schedule.value = idle;
        return;
      }
      schedule.value = loading;
      const started = value;
      try {
        const wasm = await loadNotationWasm();
        if (bytes.value === started) schedule.value = success(wasm.schedule(started));
      } catch {
        if (bytes.value === started) schedule.value = failure("schedule_failed");
      }
    },
    { immediate: true },
  );

  function audioContextCtor(): Ctx | null {
    const w = globalThis as unknown as { AudioContext?: Ctx; webkitAudioContext?: Ctx };
    return w.AudioContext ?? w.webkitAudioContext ?? null;
  }

  function tick(): void {
    if (!playing.value || !ctx) return;
    elapsedMs.value = (offsetSec + (ctx.currentTime - startCtxTime)) * 1000;
    if (playDurationMs > 0 && elapsedMs.value >= playDurationMs) {
      stop();
      return;
    }
    raf = requestAnimationFrame(tick);
  }

  function startSource(): void {
    if (!ctx || !buffer) return;
    source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(ctx.destination);
    startCtxTime = ctx.currentTime;
    source.start(0, offsetSec);
    playing.value = true;
    raf = requestAnimationFrame(tick);
  }

  /** Onset of the first sounding note (notes are onset-sorted), so a fresh Play skips
   *  the leading empty measures many scores have. 0 when unknown. */
  function firstNoteMs(): number {
    const s = schedule.value;
    return s.status === "success" && s.data.notes.length > 0 ? s.data.notes[0].onset_ms : 0;
  }

  /** Ensure the AudioContext + the rendered buffer exist. Returns false (and sets the
   *  `audio` error/loading state) when audio is unavailable or the row changed mid-load. */
  async function ensureAudio(value: Uint8Array): Promise<boolean> {
    const Ctor = audioContextCtor();
    if (!Ctor) {
      audio.value = failure("audio_failed");
      return false;
    }
    try {
      ctx ??= new Ctor();
      await ctx.resume();
      if (!buffer) {
        audio.value = loading;
        // Fetch the SoundFont from the backend delivery route with the caller's access
        // token (resolved here — reached only with a real AudioContext, so unit tests
        // that stub audio never need a Pinia instance).
        const token = useAuthStore().accessToken;
        const [wasm, sf2] = await Promise.all([loadAudioWasm(), loadSoundFont(token)]);
        if (bytes.value !== value) return false; // row changed mid-load
        const pcm = wasm.render(value, sf2, ctx.sampleRate);
        const frames = Math.floor(pcm.length / 2);
        buffer = ctx.createBuffer(2, Math.max(frames, 1), ctx.sampleRate);
        const left = buffer.getChannelData(0);
        const right = buffer.getChannelData(1);
        for (let i = 0; i < frames; i++) {
          left[i] = pcm[i * 2];
          right[i] = pcm[i * 2 + 1];
        }
        playDurationMs = buffer.duration * 1000;
        audio.value = success(undefined);
      }
      return true;
    } catch {
      audio.value = failure("audio_failed");
      playing.value = false;
      return false;
    }
  }

  async function play(): Promise<void> {
    const value = bytes.value;
    if (value == null || playing.value) return;
    if (!(await ensureAudio(value))) return;
    // Fresh start (not resuming from a pause) begins at the first note.
    if (offsetSec === 0) offsetSec = firstNoteMs() / 1000;
    startSource();
  }

  /** Seek to `ms` and play from there (a clicked measure's start). */
  async function playFrom(ms: number): Promise<void> {
    const value = bytes.value;
    if (value == null) return;
    if (!(await ensureAudio(value))) return;
    stopSource();
    cancelAnimationFrame(raf);
    const maxSec = buffer ? buffer.duration : Number.POSITIVE_INFINITY;
    offsetSec = Math.min(Math.max(0, ms) / 1000, maxSec);
    elapsedMs.value = offsetSec * 1000; // jump the cursor immediately
    startSource();
  }

  function pause(): void {
    if (!playing.value || !ctx) return;
    offsetSec += ctx.currentTime - startCtxTime;
    stopSource();
    playing.value = false;
    cancelAnimationFrame(raf);
  }

  function stopSource(): void {
    if (source) {
      source.onended = null;
      try {
        source.stop();
      } catch {
        // already stopped
      }
      source.disconnect();
      source = null;
    }
  }

  /** Stop and rewind to the start. */
  function stop(): void {
    stopSource();
    cancelAnimationFrame(raf);
    playing.value = false;
    offsetSec = 0;
    elapsedMs.value = 0;
  }

  function toggle(): void {
    if (playing.value) pause();
    else void play();
  }

  onScopeDispose(() => {
    stop();
    void ctx?.close();
  });

  return {
    schedule,
    audio,
    playing,
    elapsedMs,
    canPlay,
    toggle,
    stop,
    playFrom: (ms) => void playFrom(ms),
  };
}
