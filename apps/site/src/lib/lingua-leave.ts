// The Lingua leaving page's client (change: add-lingua-usage-counts, D12; spec
// `site-lingua-page`). Chrome and Firefox open the page when Lingua is removed while its
// anonymous counts are on, with `?b=<browser>&v=<major.minor>` and nothing else. The page
// sends one answer, chosen among six fixed ones, to `POST /web/lingua/leave` — no account,
// no cookie, no free text — and only when the reader taps it.

import type { Lang } from "./i18n";
import { config } from "./config";

/** The six answers, in the order the page shows them; the values are the wire's. */
export const LEAVE_REASONS = ["not_using", "highlighting", "missing_language", "slow", "unclear", "other"] as const;
export type LeaveReason = (typeof LEAVE_REASONS)[number];

/** The browser families the extension registers the page with, plus `unknown`. */
export type LeaveBrowser = "chromium" | "firefox" | "safari" | "unknown";

/** What one answer sends — exactly the route's body. */
export interface LeaveAnswer {
  reason: LeaveReason;
  browser: LeaveBrowser;
  /** The extension's `major.minor`, or `unknown`. */
  version: string;
}

/** Browser family and version, as the page's address carries them. */
export type LeaveContext = Pick<LeaveAnswer, "browser" | "version">;

const BROWSERS: readonly string[] = ["chromium", "firefox", "safari"];
const VERSION = /^\d{1,3}\.\d{1,3}$/;

/**
 * Reads `b` and `v` from the page's query. Anything the extension would not send — a
 * missing parameter, another browser, a version that is not `major.minor` — reads as
 * `unknown`, so a page opened by hand still sends a valid answer, and nothing else in the
 * address is ever sent.
 */
export function leaveContextFromQuery(search: string): LeaveContext {
  const params = new URLSearchParams(search.startsWith("?") ? search.slice(1) : search);
  const b = params.get("b") ?? "";
  const v = params.get("v") ?? "";
  return {
    browser: BROWSERS.includes(b) ? (b as LeaveBrowser) : "unknown",
    version: VERSION.test(v) ? v : "unknown",
  };
}

/** The help page in each site language, offered with the thanks. */
export const SUPPORT_HREF: Record<Lang, string> = {
  fr: "/support",
  en: "/en/support",
  es: "/es/soporte",
};

export interface LinguaLeaveClient {
  /** Sends one answer. Resolves on a 2xx, rejects otherwise; the page thanks either way. */
  leave(answer: LeaveAnswer): Promise<void>;
}

/**
 * The real client against the backend's HTTP origin. A plain JSON POST with no credentials:
 * no `Authorization` header and no cookie, so the answer cannot be tied to an account.
 */
export function createLinguaLeaveClient(apiUrl: string): LinguaLeaveClient {
  return {
    async leave(answer) {
      const resp = await fetch(`${apiUrl}/web/lingua/leave`, {
        method: "POST",
        credentials: "omit",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ reason: answer.reason, browser: answer.browser, version: answer.version }),
      });
      if (!resp.ok) throw new Error(`leave: ${resp.status}`);
    },
  };
}

let client: LinguaLeaveClient | null = null;

/** The page's client (created on first use, from the build's `PUBLIC_API_URL`). */
export function useLinguaLeaveClient(): LinguaLeaveClient {
  client ??= createLinguaLeaveClient(config.apiUrl);
  return client;
}

/** Test seam: inject a fake (`null` goes back to the real client). */
export function setLinguaLeaveClientForTest(fake: LinguaLeaveClient | null): void {
  client = fake;
}
