import type { LinguaPort } from "./port.ts";
import type { StudiedLanguage } from "./types.ts";
import { type RpcRequest, RPC_TYPE, type RpcResponse } from "./rpc.ts";

// The event-page side of the AnalyzerPort RPC (Firefox). Dispatches one request
// against the hosted engine, hydrating it first (so a woken event page restores state
// from storage before serving). Kept transport-agnostic so it is unit-tested with a
// fake port; background.ts wires it to runtime messaging.

export function isRpcRequest(msg: unknown): msg is RpcRequest {
  return (
    !!msg &&
    typeof msg === "object" &&
    (msg as RpcRequest).type === RPC_TYPE &&
    typeof (msg as RpcRequest).method === "string" &&
    Array.isArray((msg as RpcRequest).args) &&
    ((msg as RpcRequest).language === undefined || typeof (msg as RpcRequest).language === "string")
  );
}

type PortMethods = Record<string, (...args: unknown[]) => Promise<unknown> | unknown>;

/** Dispatch one RPC against the engine `port`; `ensure` hydrates it before the call. */
export async function handleRpc(
  port: LinguaPort,
  ensure: () => Promise<void>,
  request: RpcRequest,
): Promise<RpcResponse> {
  try {
    await ensure();
    // A language-bound call is answered on the port's view of its language; a whole-reader
    // call on the port itself. Each side only has its own methods, so a call sent to the
    // wrong one is an unknown method, never a silent default.
    const target: object = request.language === undefined ? port : port.for(request.language as StudiedLanguage);
    const fn = (target as PortMethods)[request.method];
    if (typeof fn !== "function") return { ok: false, error: `unknown method: ${request.method}` };
    const result = await fn.apply(target, request.args);
    return { ok: true, result };
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}
