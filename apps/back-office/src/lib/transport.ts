import { Code, ConnectError, createClient, type Client, type Interceptor, type Transport } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { grpcWebDevtoolsInterceptor } from "./grpc-devtools";
import { AuthService } from "@/gen/auth_pb";
import { ScoreService } from "@/gen/score_pb";
import { UserService } from "@/gen/user_pb";

// The backend speaks gRPC-web (tonic-web); Connect's gRPC-web transport talks to
// it directly. The auth interceptor attaches the bearer access token the same way
// the native interceptor expects (`Authorization: Bearer <token>`).
const authInterceptor =
  (getToken: () => string | null): Interceptor =>
  (next) =>
  async (req) => {
    const token = getToken();
    if (token) req.header.set("Authorization", `Bearer ${token}`);
    return next(req);
  };

/// Default to the backend's local gRPC addr so `yarn dev` works without a `.env`.
/// Connect's `createMethodUrl` calls `baseUrl.toString()`, so an undefined baseUrl
/// crashes with a cryptic "reading 'toString'" before any request is sent — never
/// pass undefined.
const DEFAULT_GRPC_WEB_URL = "http://localhost:50051";

export function baseUrl(): string {
  const url = import.meta.env.VITE_GRPC_WEB_URL;
  if (!url) {
    console.warn(
      `VITE_GRPC_WEB_URL is not set — defaulting to ${DEFAULT_GRPC_WEB_URL}. ` +
        "Set it in apps/back-office/.env (see .env.example) for other environments.",
    );
    return DEFAULT_GRPC_WEB_URL;
  }
  return url;
}

// Global session-expiry handling: any call that comes back UNAUTHENTICATED means
// the access token is missing/expired/rejected. A registered handler (wired in
// main.ts) signs the user out and redirects to sign-in. It fires for EVERY such
// error; the handler itself decides what to do (e.g. ignore when no session
// exists, so a failed sign-in attempt with bad credentials is NOT a redirect).
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

// Silent token refresh: an UNAUTHENTICATED response usually just means the short-
// lived access token expired, so refresh it once and retry the call with the new
// token — the user stays signed in as long as the refresh token is valid. Refreshes
// are single-flighted so concurrent 401s trigger only ONE refresh. Only if the
// refresh itself fails does the error reach the session-expiry handler (real sign-
// out). The refresh RPC is skipped to avoid recursion. Wired in main.ts.
let tokenRefresher: (() => Promise<boolean>) | null = null;
let inflightRefresh: Promise<boolean> | null = null;

export function setTokenRefresher(fn: (() => Promise<boolean>) | null): void {
  tokenRefresher = fn;
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
      // Retry once; the auth interceptor re-attaches the freshly refreshed token.
      if (await refreshOnce()) return await next(req);
    }
    throw e;
  }
};

// gRPC-web always returns HTTP 200; the real status is the grpc-status trailer, so
// the Network tab hides errors. In dev, log one clear line per failed call
// (method + decoded code + message) to the Console so failures are obvious.
const devLogInterceptor: Interceptor = (next) => async (req) => {
  try {
    return await next(req);
  } catch (e) {
    if (e instanceof ConnectError) {
      console.error(`gRPC ${req.method.name} → ${Code[e.code]} (${e.code}): ${e.rawMessage}`);
    }
    throw e;
  }
};

export function createTransport(getToken: () => string | null): Transport {
  // Order = outermost→innermost. session-expiry wraps refresh (so it only fires
  // once refresh has given up), which wraps auth (so a retry re-attaches the new
  // token).
  const interceptors: Interceptor[] = [sessionExpiryInterceptor, refreshInterceptor, authInterceptor(getToken)];
  if (import.meta.env.DEV) {
    // Console one-liner + the gRPC-Web Developer Tools panel (if the extension is
    // installed). Both dev-only.
    interceptors.push(devLogInterceptor, grpcWebDevtoolsInterceptor);
  }
  return createGrpcWebTransport({ baseUrl: baseUrl(), interceptors });
}

export interface Clients {
  auth: Client<typeof AuthService>;
  score: Client<typeof ScoreService>;
  user: Client<typeof UserService>;
}

export function createClients(transport: Transport): Clients {
  return {
    auth: createClient(AuthService, transport),
    score: createClient(ScoreService, transport),
    user: createClient(UserService, transport),
  };
}
