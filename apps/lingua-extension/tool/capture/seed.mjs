// The committed seeds' dates, moved to the run's day (add-site-lingua-showcase D4). A backup holds
// the moments it was recorded at — when each card was met, answered and is next due, when each word
// was last read and its level declared — so a seed restored a week after its recording would find
// every card due, and the review would open on another card than the one the capture was made for.
// Before restoring it, the tool moves every one of those moments by the whole days between the
// recording and the run: the reader restored stands at the run as it stood at the recording. The
// product's format is otherwise untouched. Pure.

const DAY_MS = 86_400_000;
const DAY_S = 86_400;

/** The UTC day the seed was recorded on: its levels were declared first, on the install's welcome page. */
export function recordedDay(backup) {
  const declared = Object.values(backup?.knowledge?.declared_level_at ?? {}).filter((v) => typeof v === "number");
  if (declared.length === 0) throw new Error("the seed declares no level: re-record it (yarn capture --record-seed)");
  return Math.floor(Math.min(...declared) / DAY_MS);
}

/** `value` moved by `by` when it is a number, else as it is. */
const moved = (value, by) => (typeof value === "number" ? value + by : value);

/** Every value of a `{ language: { word: value } }` table, moved by `by`. */
function movedTable(table, by) {
  if (!table) return table;
  return Object.fromEntries(
    Object.entries(table).map(([language, words]) => [
      language,
      Object.fromEntries(Object.entries(words).map(([word, value]) => [word, moved(value, by)])),
    ]),
  );
}

/**
 * `backup` with every moment moved by `days` whole days: in milliseconds `knowledge.declared_level_at`
 * and `knowledge.updated`; in seconds each card's `provenance.captured_at`, `updated_at`,
 * `review.due` and `review.last_review`, and each word's `exposure` `last_seen`; in days its
 * `last_day`. A copy; `backup` is not changed.
 */
export function shiftBackup(backup, days) {
  const out = structuredClone(backup);
  if (days === 0) return out;
  const ms = days * DAY_MS;
  const s = days * DAY_S;
  const knowledge = out.knowledge ?? {};
  if (knowledge.declared_level_at) {
    knowledge.declared_level_at = Object.fromEntries(
      Object.entries(knowledge.declared_level_at).map(([language, at]) => [language, moved(at, ms)]),
    );
  }
  knowledge.updated = movedTable(knowledge.updated, ms);
  for (const words of Object.values(out.exposure?.counters ?? {})) {
    for (const counter of Object.values(words)) {
      counter.last_seen = moved(counter.last_seen, s);
      counter.last_day = moved(counter.last_day, days);
    }
  }
  for (const cards of Object.values(out.deck?.cards ?? {})) {
    for (const card of Object.values(cards)) {
      card.updated_at = moved(card.updated_at, s);
      if (card.provenance) card.provenance.captured_at = moved(card.provenance.captured_at, s);
      if (card.review) {
        card.review.due = moved(card.review.due, s);
        card.review.last_review = moved(card.review.last_review, s);
      }
    }
  }
  return out;
}

/** `backup` as of `nowMs`: moved by the whole days between its recording and the run. */
export function seedForRun(backup, nowMs) {
  return shiftBackup(backup, Math.floor(nowMs / DAY_MS) - recordedDay(backup));
}
