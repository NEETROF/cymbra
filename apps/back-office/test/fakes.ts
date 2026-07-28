import type { Clients } from "@/lib/transport";

// A minimal JWT (unsigned; decodeClaims only reads the payload) carrying roles.
export function makeJwt(payload: Record<string, unknown>): string {
  const b64url = (o: unknown) => btoa(JSON.stringify(o)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  return `${b64url({ alg: "EdDSA" })}.${b64url(payload)}.sig`;
}

export interface SearchCall {
  query: string;
  moderationStatus?: string;
  reviewQueue?: boolean;
  sort: { field: string; descending: boolean }[];
  limit: number;
  offset: number;
}

export interface FakeState {
  searchCalls: SearchCall[];
  evaluateCalls: { scoreId: string; status: string }[];
  grantCalls: { userId: string; scope: string; role: string }[];
  revokeCalls: { userId: string; scope: string; role: string }[];
  listAccountsCalls: { query: string; limit: number; offset: number }[];
  hits: unknown[];
  total: number;
  grants: unknown[];
  accounts: unknown[];
  tokens: { accessToken: string; refreshToken: string };
}

// Build a fake `Clients` recording calls, castable to the real (large) generated
// client types. Only the methods the console uses are implemented.
export function makeFakeClients(state: Partial<FakeState> = {}): { clients: Clients; state: FakeState } {
  const s: FakeState = {
    searchCalls: [],
    evaluateCalls: [],
    grantCalls: [],
    revokeCalls: [],
    listAccountsCalls: [],
    hits: state.hits ?? [],
    total: state.total ?? 0,
    grants: state.grants ?? [],
    accounts: state.accounts ?? [],
    tokens: state.tokens ?? { accessToken: makeJwt({ roles: ["moderator"], sub: "u1" }), refreshToken: "r" },
  };
  const clients = {
    auth: {
      signInLocal: async () => s.tokens,
      signInOidc: async () => s.tokens,
      refresh: async () => s.tokens,
    },
    score: {
      searchCatalog: async (req: SearchCall) => {
        s.searchCalls.push(req);
        return { hits: s.hits, total: s.total, nextOffset: s.hits.length };
      },
      setModerationStatus: async (req: { scoreId: string; status: string }) => {
        s.evaluateCalls.push(req);
        return {};
      },
      getCatalogScore: async (req: { catalogId: string }) => s.hits[0] ?? { id: req.catalogId },
      getCatalogScoreBytes: async () => ({ data: new Uint8Array([1, 2, 3]) }),
    },
    user: {
      grantRole: async (req: { userId: string; scope: string; role: string }) => {
        s.grantCalls.push(req);
        return {};
      },
      revokeRole: async (req: { userId: string; scope: string; role: string }) => {
        s.revokeCalls.push(req);
        return {};
      },
      listRoleGrants: async () => ({ grants: s.grants }),
      listAccounts: async (req: { query: string; limit: number; offset: number }) => {
        s.listAccountsCalls.push(req);
        return { accounts: s.accounts, total: s.accounts.length };
      },
    },
  } as unknown as Clients;
  return { clients, state: s };
}
