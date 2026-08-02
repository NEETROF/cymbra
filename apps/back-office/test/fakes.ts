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

export interface EditCall {
  scoreId: string;
  title?: string;
  composer?: string;
  arranger?: string;
  level?: string;
}

export interface FakeState {
  searchCalls: SearchCall[];
  evaluateCalls: { scoreId: string; status: string }[];
  editCalls: EditCall[];
  getCatalogScoreCalls: number;
  grantCalls: { userId: string; scope: string; role: string }[];
  revokeCalls: { userId: string; scope: string; role: string }[];
  listAccountsCalls: { query: string; limit: number; offset: number }[];
  setLocaleCalls: string[];
  hits: unknown[];
  total: number;
  grants: unknown[];
  accounts: unknown[];
  /** The account's stored language `getAccount` returns; `setLocale` writes it. */
  accountLocale?: string;
  tokens: { accessToken: string; refreshToken: string };
  // feature-flags panel
  setFlagCalls: { key: string; app: string; enabled: boolean; rolloutScope: string; confirm: boolean }[];
  setConfigCalls: { key: string; app: string; rolloutScope: string; confirm: boolean; value: unknown }[];
  clearCalls: { key: string; app: string; confirm: boolean }[];
  listDefinitionsCalls: { appFilter: string }[];
  listChangesCalls: { appFilter: string; key: string }[];
  flagDefs: unknown[];
  flagChanges: unknown[];
}

// Build a fake `Clients` recording calls, castable to the real (large) generated
// client types. Only the methods the console uses are implemented.
export function makeFakeClients(state: Partial<FakeState> = {}): { clients: Clients; state: FakeState } {
  const s: FakeState = {
    searchCalls: [],
    evaluateCalls: [],
    editCalls: [],
    getCatalogScoreCalls: 0,
    grantCalls: [],
    revokeCalls: [],
    listAccountsCalls: [],
    setLocaleCalls: [],
    hits: state.hits ?? [],
    total: state.total ?? 0,
    grants: state.grants ?? [],
    accounts: state.accounts ?? [],
    accountLocale: state.accountLocale,
    tokens: state.tokens ?? { accessToken: makeJwt({ roles: ["moderator"], sub: "u1" }), refreshToken: "r" },
    setFlagCalls: [],
    setConfigCalls: [],
    clearCalls: [],
    listDefinitionsCalls: [],
    listChangesCalls: [],
    flagDefs: state.flagDefs ?? [],
    flagChanges: state.flagChanges ?? [],
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
      updateCatalogScore: async (req: EditCall) => {
        s.editCalls.push(req);
        return {};
      },
      getCatalogScore: async (req: { catalogId: string }) => {
        s.getCatalogScoreCalls += 1;
        return s.hits[0] ?? { id: req.catalogId };
      },
      getCatalogScoreBytes: async () => ({ data: new Uint8Array([1, 2, 3]) }),
      listSoundFonts: async () => ({ soundfonts: [] }),
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
      getAccount: async () => ({ userId: "u1", locale: s.accountLocale }),
      setLocale: async (req: { locale: string }) => {
        s.setLocaleCalls.push(req.locale);
        if (req.locale) s.accountLocale = req.locale;
        return { userId: "u1", locale: s.accountLocale };
      },
    },
    flags: {
      listFlagDefinitions: async (req: { appFilter: string }) => {
        s.listDefinitionsCalls.push(req);
        return { definitions: s.flagDefs };
      },
      listFlagChanges: async (req: { appFilter?: string; key?: string }) => {
        s.listChangesCalls.push({ appFilter: req.appFilter ?? "", key: req.key ?? "" });
        return { changes: s.flagChanges };
      },
      setFlag: async (req: { key: string; app: string; enabled: boolean; rolloutScope: string; confirm: boolean }) => {
        s.setFlagCalls.push(req);
        return {};
      },
      setConfig: async (req: { key: string; app: string; rolloutScope: string; confirm: boolean; value: unknown }) => {
        s.setConfigCalls.push(req);
        return {};
      },
      clearOverride: async (req: { key: string; app: string; confirm: boolean }) => {
        s.clearCalls.push(req);
        return {};
      },
    },
  } as unknown as Clients;
  return { clients, state: s };
}
