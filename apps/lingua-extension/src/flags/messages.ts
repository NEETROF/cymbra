import type { RuntimeSend } from "../sync/messages.ts";
import { FLAGS_KEY, type FlagValues, flagOn } from "./store.ts";

// Surfaces read the flags through the background, which owns the session (change:
// add-lingua-feature-betas). A read never touches the network; it may ask the background to
// refresh in the background, throttled.

export type FlagsMessage = { type: "flags:get" };

export function isFlagsMessage(m: unknown): m is FlagsMessage {
  return (m as { type?: unknown } | null)?.type === "flags:get";
}

const runtimeSend: RuntimeSend = (message) => chrome.runtime.sendMessage(message);

/** The signed-in account's flags; empty — every flag off — when the background is unreachable. */
export async function readFlags(send: RuntimeSend = runtimeSend): Promise<FlagValues> {
  try {
    const reply = (await send({ type: "flags:get" } satisfies FlagsMessage)) as { values?: FlagValues } | undefined;
    return reply?.values ?? {};
  } catch {
    return {};
  }
}

/** One boolean flag: off unless the server said `true` for this account. */
export async function readFlag(key: string, send: RuntimeSend = runtimeSend): Promise<boolean> {
  return flagOn(await readFlags(send), key);
}

/** Call `onChange` whenever the background stores a new snapshot (or clears it on sign-out). */
export function onFlagsChanged(onChange: () => void): () => void {
  const listener = (changes: Record<string, unknown>, area: string): void => {
    if (area === "local" && FLAGS_KEY in changes) onChange();
  };
  chrome.storage.onChanged.addListener(listener);
  return () => chrome.storage.onChanged.removeListener(listener);
}
