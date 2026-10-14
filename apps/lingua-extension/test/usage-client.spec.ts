import { describe, expect, it } from "vitest";
import { Code, ConnectError } from "@connectrpc/connect";
import { initApi } from "@/net/api.ts";
import { setTokenRefresher } from "@/net/transport.ts";
import { createUsageClient, outcomeOf, type UsageReport } from "@/usage/client.ts";

// The counts' transport (add-lingua-usage-counts D5, D8): its own gRPC-web transport with no
// interceptor, so a report carries no account even while a session holds a token, and each answer
// becomes accepted, invalid (dropped) or retry (kept).

const REPORT: UsageReport = {
  installWeek: 20_745,
  browser: "firefox",
  version: "1.10",
  steps: ["installed", "set_up"],
};

/** A gRPC-web frame: a flag byte, a big-endian length, the bytes. */
function frame(flag: number, bytes: Uint8Array): Uint8Array {
  const out = new Uint8Array(5 + bytes.length);
  out[0] = flag;
  new DataView(out.buffer).setUint32(1, bytes.length);
  out.set(bytes, 5);
  return out;
}

/** A successful unary answer: one empty message, then `grpc-status: 0` in the trailers. */
function ok(): Response {
  const trailer = frame(0x80, new TextEncoder().encode("grpc-status: 0\r\n"));
  const body = new Uint8Array([...frame(0, new Uint8Array()), ...trailer]);
  return new Response(body, { status: 200, headers: { "content-type": "application/grpc-web+proto" } });
}

/** A trailers-only refusal with a gRPC status. */
function refused(code: number): Response {
  return new Response(null, {
    status: 200,
    headers: { "content-type": "application/grpc-web+proto", "grpc-status": String(code), "grpc-message": "no" },
  });
}

interface Seen {
  url: string;
  headers: Headers;
  body: Uint8Array;
}

function fakeFetch(answer: () => Response | Promise<Response>) {
  const seen: Seen[] = [];
  const fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    seen.push({
      url: String(input),
      headers: new Headers(init?.headers),
      body: new Uint8Array((init?.body as Uint8Array | undefined) ?? []),
    });
    return answer();
  };
  return { fetch: fetch as typeof globalThis.fetch, seen };
}

describe("A report carries no identifier", () => {
  it("A signed-in reader: the request carries no Authorization header", async () => {
    // A session holds a token: every account call would carry it.
    initApi(() => "eyJ.signed.in.reader");
    setTokenRefresher(async () => true);
    const { fetch, seen } = fakeFetch(ok);
    const client = createUsageClient({ baseUrl: "https://api.example.test", fetch });
    expect(await client.report(REPORT)).toBe("accepted");
    expect(seen).toHaveLength(1);
    expect(seen[0].url).toBe("https://api.example.test/cymbra.lingua.v1.UsageCountsService/Report");
    expect(seen[0].headers.has("authorization")).toBe(false);
    expect(new TextDecoder().decode(seen[0].body)).not.toContain("eyJ");
    // The message: the week, the browser family, the version, the steps — "1.10" is in it.
    expect(new TextDecoder().decode(seen[0].body)).toContain("1.10");
    setTokenRefresher(null);
  });
});

describe("Reports ride the extension's own wake-ups: each answer", () => {
  it("accepted, counted or not", async () => {
    expect(await createUsageClient({ baseUrl: "https://x.test", fetch: fakeFetch(ok).fetch }).report(REPORT)).toBe(
      "accepted",
    );
  });

  it("refused as malformed: invalid, the steps are dropped", async () => {
    const { fetch } = fakeFetch(() => refused(3));
    expect(await createUsageClient({ baseUrl: "https://x.test", fetch }).report(REPORT)).toBe("invalid");
  });

  it("too many requests, a server down, the network gone: tried again later", async () => {
    for (const answer of [
      () => refused(8),
      () => refused(14),
      () => new Response("bad gateway", { status: 502 }),
      () => Promise.reject(new TypeError("Failed to fetch")),
    ]) {
      const { fetch } = fakeFetch(answer);
      expect(await createUsageClient({ baseUrl: "https://x.test", fetch }).report(REPORT)).toBe("retry");
    }
  });

  it("only a malformed refusal drops the steps", () => {
    expect(outcomeOf(new ConnectError("x", Code.InvalidArgument))).toBe("invalid");
    expect(outcomeOf(new ConnectError("x", Code.ResourceExhausted))).toBe("retry");
    expect(outcomeOf(new Error("offline"))).toBe("retry");
  });
});
