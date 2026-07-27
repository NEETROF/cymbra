import { createClients, createTransport, type Clients } from "./transport";

// A lazily-initialised singleton of the gRPC-web clients. `initApi` wires the real
// transport (bound to a token getter) at startup; tests inject fakes via
// `setClientsForTest`, so components/stores depend only on `api()`.
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
