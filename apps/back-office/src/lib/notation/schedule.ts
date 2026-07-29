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
  measure_start_ms: number[];
  song_end_ms: number;
  bpm: number;
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
  return v < lo ? lo : v > hi ? hi : v;
}
