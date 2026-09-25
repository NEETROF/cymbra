// A library that vanishes on a train is the failure the reader exists to prevent (design D5):
// the browser may evict an origin's storage under pressure unless the origin asked for it to
// be kept. So the library asks where the browser offers the question, and says so when the
// answer is no — the books still open, but the reader should know they may not stay. It asks
// once: a browser that prompts (Firefox may) is not asked again at every opening, and the
// refusal is remembered rather than re-asked.

/** What the browser said: kept, refused, or no such question on this browser. */
export type Persistence = "granted" | "refused" | "unavailable";

export type StorageManagerLike = Pick<StorageManager, "persisted" | "persist">;

/** Whether the question was already asked on this device. */
export interface AskedOnce {
  asked(): Promise<boolean>;
  markAsked(): Promise<void>;
}

/** Ask the browser to keep the extension's storage, once. Never throws. */
export async function requestPersistence(
  memo: AskedOnce,
  storage: StorageManagerLike | undefined = globalThis.navigator?.storage,
): Promise<Persistence> {
  if (!storage || typeof storage.persist !== "function" || typeof storage.persisted !== "function") {
    return "unavailable";
  }
  try {
    if (await storage.persisted()) return "granted";
    if (await memo.asked()) return "refused";
    await memo.markAsked();
    return (await storage.persist()) ? "granted" : "refused";
  } catch {
    return "unavailable";
  }
}
