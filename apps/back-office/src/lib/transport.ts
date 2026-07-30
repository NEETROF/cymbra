import { toJson, type DescMessage } from "@bufbuild/protobuf";
import { Code, ConnectError, createClient, type Client, type Interceptor, type Transport } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { grpcWebDevtoolsInterceptor } from "./grpc-devtools";
import { AuthService } from "@/gen/auth_pb";
import { ScoreService } from "@/gen/score_pb";
import { UserService } from "@/gen/user_pb";
import { FlagService } from "@/gen/flags_pb";

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

// Full decoded trace: for EVERY call, log method + decoded request/response JSON
// (or the gRPC status on failure) as a collapsed console group. Independent of any
// Chrome extension — this is the reliable way to read gRPC-web traffic, since the
// Network tab only shows the length-prefixed protobuf wire bytes.
function traceJson(schema: DescMessage, message: unknown): unknown {
  try {
    return toJson(schema, message as never);
  } catch {
    return "<undecodable>";
  }
}

// Opt-in flag that turns the trace on in a PRODUCTION build (it is always on in
// dev). Stays OFF for every user of bo.cymbra.app unless an operator explicitly runs
// `localStorage.setItem("cymbra:grpc-trace","1")` in their own browser and reloads.
// This lets us read decoded gRPC-web traffic on the live site WITHOUT loosening prod
// CORS/cookies (the reason we don't just point a local dev build at prod). Read once
// at transport creation (app startup), so toggling requires a reload.
const TRACE_FLAG = "cymbra:grpc-trace";

function grpcTraceEnabled(): boolean {
  if (import.meta.env.DEV) return true;
  try {
    return localStorage.getItem(TRACE_FLAG) === "1";
  } catch {
    return false; // localStorage blocked (private mode / hardened browser) — stay off
  }
}

const grpcTraceInterceptor: Interceptor = (next) => async (req) => {
  const label = `gRPC ${req.method.parent.typeName}/${req.method.name}`;
  const request = req.stream ? "<stream>" : traceJson(req.method.input, req.message);
  try {
    const res = await next(req);
    const response = res.stream ? "<stream>" : traceJson(req.method.output, res.message);
    console.groupCollapsed(`%c✓ ${label}`, "color:#3c7");
    console.log("request ", request);
    console.log("response", response);
    console.groupEnd();
    return res;
  } catch (e) {
    console.groupCollapsed(`%c✗ ${label}`, "color:#e55");
    console.log("request", request);
    if (e instanceof ConnectError) console.log(`status  ${Code[e.code]} (${e.code}): ${e.rawMessage}`);
    else console.log("error  ", e);
    console.groupEnd();
    throw e;
  }
};

export function createTransport(getToken: () => string | null): Transport {
  // Order = outermost→innermost. session-expiry wraps refresh (so it only fires
  // once refresh has given up), which wraps auth (so a retry re-attaches the new
  // token).
  const interceptors: Interceptor[] = [sessionExpiryInterceptor, refreshInterceptor, authInterceptor(getToken)];
  // Decoded request/response trace in the Console — always in dev, opt-in in prod
  // (see grpcTraceEnabled). Announce it in a prod build so the operator knows it's on.
  if (grpcTraceEnabled()) {
    interceptors.push(grpcTraceInterceptor);
    if (!import.meta.env.DEV) {
      console.info(`[cymbra] gRPC trace ON (localStorage "${TRACE_FLAG}"). Remove the key + reload to disable.`);
    }
  }
  // The gRPC-Web Developer Tools panel bridge (if the extension is installed) is
  // dev-tooling only — never shipped to the prod bundle.
  if (import.meta.env.DEV) interceptors.push(grpcWebDevtoolsInterceptor);
  return createGrpcWebTransport({ baseUrl: baseUrl(), interceptors });
}

export interface Clients {
  auth: Client<typeof AuthService>;
  score: Client<typeof ScoreService>;
  user: Client<typeof UserService>;
  flags: Client<typeof FlagService>;
}

export function createClients(transport: Transport): Clients {
  return {
    auth: createClient(AuthService, transport),
    score: createClient(ScoreService, transport),
    user: createClient(UserService, transport),
    flags: createClient(FlagService, transport),
  };
}
