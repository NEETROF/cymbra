// The AnalyzerPort RPC protocol (Firefox variant). Firefox's CSP blocks WASM in a
// content script, so the engine lives in the event page and the content script /
// side panel reach it over runtime messaging. This module is the wire contract shared
// by the caller (messaging-port.ts) and the host (the event-page handler in
// background.ts). Everything crossing the wire is structured-clone-serialisable
// (plain objects, strings, numbers, null).

export const RPC_TYPE = "lingua-rpc";

export interface RpcRequest {
  type: typeof RPC_TYPE;
  method: string;
  args: unknown[];
}

export type RpcResponse = { ok: true; result: unknown } | { ok: false; error: string };

/** Send one RPC to the event-page engine and await its result (caller side). */
export async function sendRpc(method: string, args: unknown[]): Promise<unknown> {
  const request: RpcRequest = { type: RPC_TYPE, method, args };
  const response = (await chrome.runtime.sendMessage(request)) as RpcResponse | undefined;
  if (!response || !response.ok) throw new Error(response && !response.ok ? response.error : "lingua RPC failed");
  return response.result;
}
