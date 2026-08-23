// TS mirror of the playback schedule the wasm `schedule(bytes)` entry returns
// (`cymbra-musicxml-core::PlaybackSchedule`), plus the pure playhead maths shared by
// the app: which measure the playhead is in, and which notes are sounding. Keys are
// snake_case to match serde.

export interface TimedNote {
  midi: number;
  onset_ms: number;
  duration_ms: number;
  staff: number;
  measure_index: number;
  note_index: number;
}

export interface PlaybackSchedule {
  notes: TimedNote[];
  /** Start of each PLAYED slot of the playback order (the written measure
   *  table one-to-one when the piece has no repeats). */
  measure_start_ms: number[];
  /** The written measure each played slot performs, aligned with
   *  `measure_start_ms`. Absent on schedules from a pre-repeat wasm build. */
  written_measure?: number[];
  song_end_ms: number;
  bpm: number;
}

/** The written measure performed at played slot `slot` (identity without the
 *  mapping — pieces with no repeats, or an older wasm build). */
export function writtenMeasureAt(schedule: PlaybackSchedule, slot: number): number {
  const map = schedule.written_measure;
  return map && slot >= 0 && slot < map.length ? map[slot] : slot;
}

/** Playback start (ms) of a WRITTEN measure: its first played slot (a repeated
 *  bar seeks to its first pass), or null when it is never played. */
export function startOfWrittenMeasure(schedule: PlaybackSchedule, written: number): number | null {
  const map = schedule.written_measure;
  if (!map || map.length === 0) return schedule.measure_start_ms[written] ?? null;
  for (let i = 0; i < map.length; i++) {
    if (map[i] === written) return schedule.measure_start_ms[i] ?? null;
  }
  return null;
}

/** The measure containing `elapsedMs` and the fraction through it, or null when the
 *  playhead is before the first measure / past the end. Mirrors the app's
 *  `PlayerData.measureAt`. */
export function measureAt(schedule: PlaybackSchedule, elapsedMs: number): { index: number; fraction: number } | null {
  const starts = schedule.measure_start_ms;
  if (starts.length === 0 || elapsedMs < starts[0]) return null;
  for (let i = 0; i < starts.length; i++) {
    const start = starts[i];
    const end = i + 1 < starts.length ? starts[i + 1] : schedule.song_end_ms;
    if (elapsedMs >= start && elapsedMs < end) {
      const span = end - start;
      const fraction = span > 0 ? clamp((elapsedMs - start) / span, 0, 1) : 0;
      return { index: i, fraction };
    }
  }
  return null;
}

/** `data-note` ids (`"<measureIndex>:<noteIndex>"`) of the notes sounding at
 *  `elapsedMs`, for highlighting their heads. */
export function playingNoteIds(schedule: PlaybackSchedule, elapsedMs: number): Set<string> {
  const ids = new Set<string>();
  for (const n of schedule.notes) {
    if (n.onset_ms <= elapsedMs && elapsedMs < n.onset_ms + n.duration_ms) {
      ids.add(`${n.measure_index}:${n.note_index}`);
    }
  }
  return ids;
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.min(Math.max(v, lo), hi);
}
