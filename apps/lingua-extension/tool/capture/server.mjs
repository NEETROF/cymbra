// The capture's local web server (add-site-lingua-showcase D3): content scripts do not run on
// `file://` without a setting, so the fixture pages are served from 127.0.0.1, on a port the system
// picks. It serves the committed pages and whatever the tool adds (the EPUBs it assembles), nothing
// else, and only to this machine.
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, normalize } from "node:path";

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".epub": "application/epub+zip",
};

/**
 * Serve `dir` and the in-memory `extra` files (path → bytes) on 127.0.0.1.
 * @param {{ dir: string, extra?: Map<string, Uint8Array> }} options
 * @returns {Promise<{ origin: string, close: () => Promise<void> }>}
 */
export async function serveFixtures({ dir, extra = new Map() }) {
  const server = createServer(async (request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://127.0.0.1").pathname);
    const type = TYPES[extname(path)] ?? "application/octet-stream";
    try {
      const body = extra.get(path) ?? (await readFile(safeJoin(dir, path)));
      response.writeHead(200, { "content-type": type, "cache-control": "no-store" });
      response.end(body);
    } catch {
      response.writeHead(404, { "content-type": "text/plain" });
      response.end("not found");
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = /** @type {import("node:net").AddressInfo} */ (server.address());
  return {
    origin: `http://127.0.0.1:${port}`,
    close: () => new Promise((resolve) => server.close(() => resolve())),
  };
}

/** `path` under `dir`, never above it. */
function safeJoin(dir, path) {
  const inside = normalize(path).replace(/^(\.\.[/\\])+/, "");
  return join(dir, inside);
}
