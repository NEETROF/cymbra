import type { Clients } from "@/lib/transport";

// A minimal JWT (unsigned; decodeClaims only reads the payload) carrying roles.
export function makeJwt(payload: Record<string, unknown>): string {
  const b64url = (o: unknown) => btoa(JSON.stringify(o)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  return `${b64url({ alg: "EdDSA" })}.${b64url(payload)}.sig`;
}

export interface SearchCall {
  query: string;
  instrument?: string;
  moderationStatus?: string;
  reviewQueue?: boolean;
  allStatuses?: boolean;
  source?: string;
  hasPreview?: boolean;
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
  evaluateCalls: { scoreId: string; status: string; reason?: string }[];
  editCalls: EditCall[];
  getCatalogScoreCalls: number;
  grantCalls: { userId: string; scope: string; role: string }[];
  revokeCalls: { userId: string; scope: string; role: string }[];
  listAccountsCalls: { query: string; limit: number; offset: number; ids?: string[] }[];
  reliabilityCalls: string[];
  /** Admin session revocations requested, by target user id. */
  revokeAccountSessionsCalls: string[];
  reliability?: unknown;
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
  /** Make `listFlagDefinitions` reject, so a caller's error branch is exercised. */
  failFlags?: boolean;
  /** Make `setFlag`/`setConfig` reject with this error (e.g. a ConnectError), so
   *  the store's refusal mapping is exercised (change: add-flag-campaign-integrity). */
  failFlagWrite?: unknown;
  // plans console (change: add-premium-subscription)
  lookupCalls: { userId: string; handle: string }[];
  /** What `lookupAccountPlan` returns (a LookupAccountPlanResponse-shaped object). */
  lookup?: unknown;
  campaigns: unknown[];
  members: unknown[];
  grantPremiumCalls: { userId: string; handle: string; endsAt?: string; confirmOpenEnded: boolean; reason: string }[];
  revokeEntitlementCalls: { entitlementId: string; reason: string }[];
  enrolCalls: { userId: string; handle: string; campaignKey: string; reason: string }[];
  revokeMembershipCalls: { userId: string; handle: string; campaignKey: string; reason: string }[];
  createCampaignCalls: { key: string; name: string; kind: string; durationDays?: number }[];
  closeEnrollmentCalls: string[];
  closeCampaignCalls: string[];
  reopenCampaignCalls: string[];
  reopenEnrollmentCalls: string[];
  /** How many memberships the fake says a reopening would restore. */
  reactivatable: number;
  mintCalls: { campaignKey: string; count: number; issuedToHint: string }[];
  /** Codes `mintCodes` returns (deterministic). */
  mintedCodes: string[];
  revokeCodesCalls: { campaignKey: string; codeIds: string[] }[];
  listMembersCalls: string[];
  idsByPlanCalls: { plan: string; betaCampaignKey: string }[];
  /** What `listAccountIdsByPlan` resolves to. */
  idsByPlan: string[];
  plansForAccountsCalls: string[][];
  /** Badges `getPlansForAccounts` returns. */
  badges: unknown[];
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
    reliabilityCalls: [],
    revokeAccountSessionsCalls: [],
    reliability: state.reliability,
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
    failFlags: state.failFlags,
    failFlagWrite: state.failFlagWrite,
    lookupCalls: [],
    lookup: state.lookup,
    campaigns: state.campaigns ?? [],
    members: state.members ?? [],
    grantPremiumCalls: [],
    revokeEntitlementCalls: [],
    enrolCalls: [],
    revokeMembershipCalls: [],
    createCampaignCalls: [],
    closeEnrollmentCalls: [],
    closeCampaignCalls: [],
    reopenCampaignCalls: [],
    reopenEnrollmentCalls: [],
    reactivatable: state.reactivatable ?? 0,
    mintCalls: [],
    mintedCodes: state.mintedCodes ?? ["CODE-1", "CODE-2"],
    revokeCodesCalls: [],
    listMembersCalls: [],
    idsByPlanCalls: [],
    idsByPlan: state.idsByPlan ?? [],
    plansForAccountsCalls: [],
    badges: state.badges ?? [],
  };
  const clients = {
    auth: {
      signInLocal: async () => s.tokens,
      signInOidc: async () => s.tokens,
      refresh: async () => s.tokens,
      revokeAccountSessions: async (req: { userId: string }) => {
        s.revokeAccountSessionsCalls.push(req.userId);
        return {};
      },
    },
    score: {
      searchCatalog: async (req: SearchCall) => {
        s.searchCalls.push(req);
        return { hits: s.hits, total: s.total, nextOffset: s.hits.length };
      },
      setModerationStatus: async (req: { scoreId: string; status: string; reason?: string }) => {
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
      getCuratorReliability: async (req: { userId: string }) => {
        s.reliabilityCalls.push(req.userId);
        return (
          s.reliability ?? {
            totalRatings: 0n,
            coverageContribution: 0n,
            alignmentRate: 0,
            settledCount: 0n,
            alignedCount: 0n,
          }
        );
      },
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
      listAccounts: async (req: { query: string; limit: number; offset: number; ids?: string[] }) => {
        s.listAccountsCalls.push(req);
        // `ids` (pre-resolved by the plan service, or a single account for the detail
        // page) narrows the directory like the server does.
        const scoped =
          req.ids && req.ids.length > 0
            ? s.accounts.filter((a) => req.ids!.includes((a as { userId: string }).userId))
            : s.accounts;
        return { accounts: scoped, total: scoped.length };
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
        if (s.failFlags) throw new Error("flag store unavailable");
        return { definitions: s.flagDefs };
      },
      listFlagChanges: async (req: { appFilter?: string; key?: string }) => {
        s.listChangesCalls.push({ appFilter: req.appFilter ?? "", key: req.key ?? "" });
        return { changes: s.flagChanges };
      },
      setFlag: async (req: { key: string; app: string; enabled: boolean; rolloutScope: string; confirm: boolean }) => {
        s.setFlagCalls.push(req);
        if (s.failFlagWrite) throw s.failFlagWrite;
        return {};
      },
      setConfig: async (req: { key: string; app: string; rolloutScope: string; confirm: boolean; value: unknown }) => {
        s.setConfigCalls.push(req);
        if (s.failFlagWrite) throw s.failFlagWrite;
        return {};
      },
      clearOverride: async (req: { key: string; app: string; confirm: boolean }) => {
        s.clearCalls.push(req);
        return {};
      },
    },
    plans: {
      lookupAccountPlan: async (req: { userId: string; handle: string }) => {
        s.lookupCalls.push(req);
        return s.lookup ?? { userId: "u-x", snapshot: { plan: "free", betas: [] }, rows: [], memberships: [] };
      },
      listCampaigns: async () => ({ campaigns: s.campaigns }),
      listMembers: async (req: { campaignKey: string }) => {
        s.listMembersCalls.push(req.campaignKey);
        return { members: s.members };
      },
      grantPremium: async (req: FakeState["grantPremiumCalls"][number]) => {
        s.grantPremiumCalls.push(req);
        return { row: { id: "e-new" } };
      },
      revokeEntitlement: async (req: { entitlementId: string; reason: string }) => {
        s.revokeEntitlementCalls.push(req);
        return {};
      },
      enrolHandle: async (req: FakeState["enrolCalls"][number]) => {
        s.enrolCalls.push(req);
        return { membership: {} };
      },
      revokeMembership: async (req: FakeState["revokeMembershipCalls"][number]) => {
        s.revokeMembershipCalls.push(req);
        return {};
      },
      createCampaign: async (req: FakeState["createCampaignCalls"][number]) => {
        s.createCampaignCalls.push(req);
        return { campaign: { ...req, id: "c-new" } };
      },
      closeEnrollment: async (req: { campaignKey: string }) => {
        s.closeEnrollmentCalls.push(req.campaignKey);
        return {};
      },
      closeCampaign: async (req: { campaignKey: string }) => {
        s.closeCampaignCalls.push(req.campaignKey);
        return {};
      },
      reopenCampaign: async (req: { campaignKey: string }) => {
        s.reopenCampaignCalls.push(req.campaignKey);
        return { reactivated: s.reactivatable };
      },
      reopenEnrollment: async (req: { campaignKey: string }) => {
        s.reopenEnrollmentCalls.push(req.campaignKey);
        return {};
      },
      previewReopenCampaign: async () => ({
        reactivated: s.reactivatable,
      }),
      mintCodes: async (req: { campaignKey: string; count: number; issuedToHint: string }) => {
        s.mintCalls.push(req);
        return { codes: s.mintedCodes.slice(0, req.count) };
      },
      revokeCodes: async (req: { campaignKey: string; codeIds: string[] }) => {
        s.revokeCodesCalls.push(req);
        return { revoked: 3 };
      },
      listAccountIdsByPlan: async (req: { plan: string; betaCampaignKey: string }) => {
        s.idsByPlanCalls.push(req);
        return { userIds: s.idsByPlan };
      },
      getPlansForAccounts: async (req: { userIds: string[] }) => {
        s.plansForAccountsCalls.push(req.userIds);
        return { badges: s.badges };
      },
    },
  } as unknown as Clients;
  return { clients, state: s };
}
