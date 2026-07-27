import { toJson, type DescMessage } from "@bufbuild/protobuf";
import { ConnectError, type Interceptor } from "@connectrpc/connect";

// Feed the "gRPC-Web Developer Tools" Chrome extension
// (github.com/SafetyCulture/grpc-web-devtools): it renders each call — method,
// request, response, and the real gRPC status — in a dedicated DevTools panel, so
// you don't have to read the grpc-status trailer off a "200 OK" by hand.
//
// The extension listens for window messages tagged `__GRPCWEB_DEVTOOLS__`. This is
// unary-only (the console makes no streaming calls), dev-only, and fully guarded so
// it can never break a real request (no extension installed → the postMessage is a
// harmless no-op).

function post(payload: Record<string, unknown>): void {
  try {
    window.postMessage({ type: "__GRPCWEB_DEVTOOLS__", ...payload }, "*");
  } catch {
    /* extension absent or window unavailable — ignore */
  }
}

function safeJson(schema: DescMessage, message: unknown): unknown {
  try {
    return toJson(schema, message as never);
  } catch {
    return undefined;
  }
}

export const grpcWebDevtoolsInterceptor: Interceptor = (next) => async (req) => {
  const method = `/${req.method.parent.typeName}/${req.method.name}`;
  // Always send objects (never undefined) — the extension drops entries whose
  // request/response is falsy, which would hide successful calls.
  const request = (req.stream ? {} : safeJson(req.method.input, req.message)) ?? {};
  try {
    const res = await next(req);
    const response = (res.stream ? {} : safeJson(req.method.output, res.message)) ?? {};
    post({ method, methodType: "unary", request, response });
    return res;
  } catch (e) {
    post({
      method,
      methodType: "unary",
      request,
      error:
        e instanceof ConnectError
          ? { code: e.code, message: e.rawMessage }
          : { message: String(e) },
    });
    throw e;
  }
};
