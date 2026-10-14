import { Code, ConnectError, createClient } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { BrowserFamily, UsageCountsService, UsageStep as WireStep } from "@/gen/usage_counts_pb";
import { baseUrl } from "../net/transport.ts";
import type { BrowserFamily as Browser, ReportOutcome, UsageStep } from "./counts.ts";

// The anonymous usage counts' transport (change: add-lingua-usage-counts, D5, D8). A gRPC-web
// transport of its own, built with NO interceptor: no bearer, no refresh, no session-expiry hook —
// so no code path can attach the account to a report, even while a session holds a token. The
// service it calls is mounted with no authentication on the server.

/** One report: the week of day 0, the browser family, the version (major.minor) and the steps. */
export interface UsageReport {
  installWeek: number;
  browser: Browser;
  version: string;
  steps: readonly UsageStep[];
}

/** The one call the counts make. */
export interface UsageClient {
  report(report: UsageReport): Promise<ReportOutcome>;
}

const WIRE_STEPS: Record<UsageStep, WireStep> = {
  installed: WireStep.INSTALLED,
  set_up: WireStep.SET_UP,
  first_analysis: WireStep.FIRST_ANALYSIS,
  returned_day_1: WireStep.RETURNED_DAY_1,
  returned_days_7_13: WireStep.RETURNED_DAYS_7_13,
  returned_days_30_59: WireStep.RETURNED_DAYS_30_59,
};

const WIRE_BROWSERS: Record<Browser, BrowserFamily> = {
  chromium: BrowserFamily.CHROMIUM,
  firefox: BrowserFamily.FIREFOX,
  safari: BrowserFamily.SAFARI,
};

/**
 * How a failure is taken (D5): refused as malformed, the steps are dropped so a bug cannot loop;
 * anything else — too many requests, the server down, the network gone — is tried again later.
 * The server acknowledges everything it can parse, counted or not.
 */
export function outcomeOf(e: unknown): ReportOutcome {
  return e instanceof ConnectError && e.code === Code.InvalidArgument ? "invalid" : "retry";
}

export interface UsageClientOptions {
  /** The backend's gRPC-web origin; the build's by default. */
  baseUrl?: string;
  /** The fetch the transport uses (tests). */
  fetch?: typeof globalThis.fetch;
}

/** The counts' client: its own transport, without any interceptor. */
export function createUsageClient(options: UsageClientOptions = {}): UsageClient {
  const transport = createGrpcWebTransport({
    baseUrl: options.baseUrl ?? baseUrl(),
    fetch: options.fetch,
  });
  const client = createClient(UsageCountsService, transport);
  return {
    async report(r) {
      try {
        await client.report({
          installWeek: r.installWeek,
          browser: WIRE_BROWSERS[r.browser],
          version: r.version,
          steps: r.steps.map((s) => WIRE_STEPS[s]),
        });
        return "accepted";
      } catch (e) {
        return outcomeOf(e);
      }
    },
  };
}
