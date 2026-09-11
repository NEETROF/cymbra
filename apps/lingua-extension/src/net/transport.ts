import { Code, type Client, ConnectError, createClient, type Interceptor, type Transport } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { AuthService } from "@/gen/auth_pb";
import { DeckService } from "@/gen/deck_pb";
import { KnownWordsService } from "@/gen/known_words_pb";
import { StatsService } from "@/gen/stats_pb";

// gRPC-web bearer transport for the extension, cloned from the back office
// (apps/back-office/src/lib/transport.ts). Design D1 (add-lingua-connected-clients):
// the extension talks gRPC-web with an `Authorization: Bearer` access token, never the
// /web/auth cookie (SameSite=Strict + exact-origin CORS is hostile to extensions). The
// tokens themselves live in the Session (state/session.ts); this module only knows how
// to attach one, refresh once on expiry, and surface a terminal 401.

// Backend gRPC-web origin, injected by esbuild `define` (env.d.ts). Defaults to the
// local backend so a dogfooding build works without configuration.

const authInterceptor =
  (getToken: () => string | null): Interceptor =>
  (next) =>
  async (req) => {
    const token = getToken();
    if (token) req.header.set("Authorization", `Bearer ${token}`);
    return next(req);
  };

// A call that comes back UNAUTHENTICATED after a refresh has already been tried means
// the session is really gone. The handler (wired in the background) purges local tokens.
let onUnauthenticated: (() => void) | null = null;

export function setUnauthenticatedHandler(fn: (() => void) | null): void {
  onUnauthenticated = fn;
}

/** Notify the handler when `e` is an UNAUTHENTICATED Connect error. Exported for tests. */
export function notifyIfUnauthenticated(e: unknown): void {
  if (e instanceof ConnectError && e.code === Code.Unauthenticated) onUnauthenticated?.();
}

const sessionExpiryInterceptor: Interceptor = (next) => async (req) => {
  try {
    return await next(req);
  } catch (e) {
    notifyIfUnauthenticated(e);
    throw e;
  }
};

// Silent single-flight refresh: an UNAUTHENTICATED response usually just means the
// short-lived access token expired, so refresh it once and retry — concurrent 401s
// trigger only ONE refresh. Only if the refresh itself fails does the error reach the
// session-expiry handler. The refresh RPC is skipped to avoid recursion.
let tokenRefresher: (() => Promise<boolean>) | null = null;
let inflightRefresh: Promise<boolean> | null = null;

export function setTokenRefresher(fn: (() => Promise<boolean>) | null): void {
  tokenRefresher = fn;
}

/** Reset the single-flight state (tests). */
export function resetRefreshState(): void {
  inflightRefresh = null;
}

function refreshOnce(): Promise<boolean> {
  if (!tokenRefresher) return Promise.resolve(false);
  inflightRefresh ??= tokenRefresher().finally(() => {
    inflightRefresh = null;
  });
  return inflightRefresh;
}

export const refreshInterceptor: Interceptor = (next) => async (req) => {
  try {
    return await next(req);
  } catch (e) {
    // Never refresh-and-retry the refresh call itself (would recurse / deadlock).
    const isRefreshCall = req.method === AuthService.method.refresh;
    if (!isRefreshCall && e instanceof ConnectError && e.code === Code.Unauthenticated) {
      if (await refreshOnce()) return await next(req);
    }
    throw e;
  }
};

export function baseUrl(): string {
  return __GRPC_WEB_URL__;
}

export function createTransport(getToken: () => string | null): Transport {
  // Order = outermost→innermost: session-expiry wraps refresh (so it only fires once
  // refresh has given up), which wraps auth (so the retry re-attaches the new token).
  const interceptors: Interceptor[] = [sessionExpiryInterceptor, refreshInterceptor, authInterceptor(getToken)];
  return createGrpcWebTransport({ baseUrl: baseUrl(), interceptors });
}

export interface Clients {
  auth: Client<typeof AuthService>;
  knownWords: Client<typeof KnownWordsService>;
  deck: Client<typeof DeckService>;
  stats: Client<typeof StatsService>;
}

export function createClients(transport: Transport): Clients {
  return {
    auth: createClient(AuthService, transport),
    knownWords: createClient(KnownWordsService, transport),
    deck: createClient(DeckService, transport),
    stats: createClient(StatsService, transport),
  };
}
