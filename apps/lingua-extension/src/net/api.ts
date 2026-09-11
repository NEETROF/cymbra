import { type Clients, createClients, createTransport } from "./transport.ts";

// Lazily-initialised singleton of the gRPC-web clients, behind a seam so callers depend
// only on `api()`. `initApi` wires the real transport (bound to a token getter) in the
// background; tests inject fakes via `setClientsForTest`. Mirrors the back office's
// apps/back-office/src/lib/api.ts.
let clients: Clients | null = null;

export function initApi(getToken: () => string | null): void {
  clients = createClients(createTransport(getToken));
}

export function setClientsForTest(fake: Clients): void {
  clients = fake;
}

export function api(): Clients {
  if (!clients) throw new Error("api() used before initApi()/setClientsForTest()");
  return clients;
}
