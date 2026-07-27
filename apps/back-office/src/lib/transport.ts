import {
  Code,
  ConnectError,
  createClient,
  type Client,
  type Interceptor,
  type Transport,
} from "@connectrpc/connect";
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

// gRPC-web always returns HTTP 200; the real status is the grpc-status trailer, so
// the Network tab hides errors. In dev, log one clear line per failed call
// (method + decoded code + message) to the Console so failures are obvious.
const devLogInterceptor: Interceptor = (next) => async (req) => {
  try {
    return await next(req);
  } catch (e) {
    if (e instanceof ConnectError) {
      console.error(
        `gRPC ${req.method.name} → ${Code[e.code]} (${e.code}): ${e.rawMessage}`,
      );
    }
    throw e;
  }
};

export function createTransport(getToken: () => string | null): Transport {
  const interceptors: Interceptor[] = [authInterceptor(getToken)];
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
