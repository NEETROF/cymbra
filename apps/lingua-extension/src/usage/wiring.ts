import { INTERFACE_LANGUAGE_KEY } from "../i18n/language.ts";
import { isSyncMessage } from "../sync/messages.ts";
import { touchesDataCollection } from "./consent.ts";
import type { UsageHost } from "./host.ts";
import { isUsageMessage } from "./messages.ts";

// Where the anonymous usage counts listen in the background (change: add-lingua-usage-counts, D1,
// D5, D6, D12): the wake-ups the background already has, and nothing else — no alarm, no timer.
// Kept out of background.ts so the wiring itself is tested (task 4.3): which message is a use,
// which only a wake, and which events follow the consent and the interface language.

/** The part of the background's host the wiring drives. */
export type UsageListener = Pick<
  UsageHost,
  "installed" | "analysed" | "used" | "wake" | "consentChanged" | "registerLeavingPage" | "handle"
>;

interface Event<Callback> {
  addListener(callback: Callback): void;
}

type MessageCallback = (
  message: unknown,
  sender: { tab?: { id?: number } },
  sendResponse: (response?: unknown) => void,
) => boolean | undefined;

/** The browser events the counts listen to: `chrome.*` in the background, fakes in tests. */
export interface UsageEvents {
  onMessage: Event<MessageCallback>;
  onInstalled: Event<(details: { reason: string }) => void>;
  onStartup: Event<() => void>;
  onStorageChanged: Event<(changes: Record<string, unknown>, area: string) => void>;
  /** Firefox only: its data-collection consent, changed in `about:addons` or from the switch. */
  onPermissionsAdded?: Event<(permissions: unknown) => void>;
  onPermissionsRemoved?: Event<(permissions: unknown) => void>;
}

/**
 * A page analysed in a studied language: a `stats` message from a tab with a percentage. A page
 * in no studied language (`pct` null) and a reader switched off on the tab are not uses. Only the
 * percentage's presence is read — never the language the message also carries.
 */
export function isAnalysedPage(message: unknown, sender: { tab?: { id?: number } }): boolean {
  const m = message as { type?: unknown; pct?: unknown; disabled?: unknown } | null;
  return m?.type === "stats" && m.disabled !== true && typeof m.pct === "number" && sender.tab?.id != null;
}

/** Listen to the events the counts ride on; the leaving page is set at once, from every start. */
export function attachUsage(usage: UsageListener, events: UsageEvents): void {
  void usage.registerLeavingPage();
  // The leaving page is in the interface language (D12).
  events.onStorageChanged.addListener((changes, area) => {
    if (area === "local" && INTERFACE_LANGUAGE_KEY in changes) void usage.registerLeavingPage();
  });
  events.onMessage.addListener((message, sender, sendResponse) => {
    if (isUsageMessage(message)) {
      void usage.handle(message).then(sendResponse, () => sendResponse(undefined));
      return true; // async response
    }
    if (isAnalysedPage(message, sender)) void usage.analysed();
    // A surface the reader opened is a use; a page that loaded is only a wake that sends what
    // may leave. The welcome tab sends no such request: its opening is not a use. The sync's own
    // listener answers the message.
    if (isSyncMessage(message) && !message.force) void (message.reason === "surface" ? usage.used() : usage.wake());
    return undefined;
  });
  // A new installation is day 0; an update — from a release that did not count — starts nothing.
  events.onInstalled.addListener((details) => void usage.installed(details.reason));
  // A browser start: try what may leave.
  events.onStartup.addListener(() => void usage.wake());
  // Firefox's consent given or withdrawn elsewhere: follow it (D6).
  const consentMoved = (permissions: unknown): void => {
    if (touchesDataCollection(permissions)) void usage.consentChanged();
  };
  events.onPermissionsAdded?.addListener(consentMoved);
  events.onPermissionsRemoved?.addListener(consentMoved);
}
