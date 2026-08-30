import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { flushPromises, mount } from "@vue/test-utils";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import { i18n } from "@/i18n";
import { buildValueKind, readBool, readValue, useFlagsStore } from "@/stores/flags";
import FlagsView from "@/views/FlagsView.vue";
import { makeFakeClients, type FakeState } from "./fakes";

// FlagValue wire-shape helpers (Connect-ES oneof: { case, value }).
const boolVal = (v: boolean) => ({ kind: { case: "boolValue", value: v } });
const intVal = (v: number) => ({ kind: { case: "intValue", value: BigInt(v) } });

function def(p: Record<string, unknown> = {}) {
  return {
    key: "k",
    app: "music",
    valueType: "bool",
    defaultValue: boolVal(false),
    effectiveValue: boolVal(false),
    hasOverride: false,
    rolloutScope: "global",
    sensitive: false,
    doc: "",
    editable: true,
    updatedBy: "",
    updatedAt: "",
    ...p,
  };
}

const DEFS = [
  def({ key: "rating.enabled", valueType: "bool", effectiveValue: boolVal(false) }),
  def({ key: "rating.review.min_votes", valueType: "int", defaultValue: intVal(5), effectiveValue: intVal(5) }),
  def({
    key: "account.min_public_sharing_age",
    app: "all",
    valueType: "int",
    defaultValue: intVal(16),
    effectiveValue: intVal(16),
    sensitive: true,
  }),
];

// The write refusals of change add-flag-campaign-integrity, exactly as the backend
// sends them: gRPC code + a stable message prefix.
const unknownCampaignErr = () =>
  new ConnectError("unknown_campaign: rollout scope `beta:ghost` names no campaign", Code.FailedPrecondition);
const unverifiableErr = () =>
  new ConnectError(
    "campaign_unverifiable: cannot verify rollout scope `beta:ghost` right now; retry later",
    Code.Aborted,
  );

// Campaigns as `ListCampaigns` returns them (closed ones included).
const CAMPAIGNS = [
  { id: "c1", key: "midi-drums", name: "MIDI drums", kind: "feature", closedAt: "2026-08-01T00:00:00Z" },
  { id: "c2", key: "spring-beta", name: "Spring beta", kind: "feature" },
];

describe("flags value helpers", () => {
  it("reads typed wire values as display strings / bools", () => {
    expect(readValue(boolVal(true) as never)).toBe("true");
    expect(readValue(intVal(20) as never)).toBe("20");
    expect(readBool(boolVal(true) as never)).toBe(true);
    expect(readBool(intVal(1) as never)).toBe(false);
  });

  it("builds typed wire values and validates input", () => {
    expect(buildValueKind("bool", "true")).toEqual({ case: "boolValue", value: true });
    expect(buildValueKind("int", "9")).toEqual({ case: "intValue", value: 9n });
    expect(buildValueKind("number", "2.5")).toEqual({ case: "numberValue", value: 2.5 });
    expect(buildValueKind("json", '{"a":1}')).toEqual({ case: "jsonValue", value: '{"a":1}' });
    expect(() => buildValueKind("int", "nope")).toThrow();
    expect(() => buildValueKind("number", "x")).toThrow();
    expect(() => buildValueKind("json", "3")).toThrow();
  });
});

describe("flags store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("loads and maps declared definitions", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useFlagsStore();
    await store.load("");
    expect(state.listDefinitionsCalls).toEqual([{ appFilter: "" }]);
    expect(store.definitions.status).toBe("success");
    if (store.definitions.status === "success") {
      const rows = store.definitions.data;
      expect(rows).toHaveLength(3);
      expect(rows[0]).toMatchObject({ key: "rating.enabled", valueType: "bool", effectiveBool: false });
      expect(rows[2]).toMatchObject({ key: "account.min_public_sharing_age", sensitive: true, effectiveDisplay: "16" });
    }
  });

  it("toggles a flag then re-lists", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useFlagsStore();
    await store.load("music");
    await store.setFlag("rating.enabled", "music", true, "global", false);
    expect(state.setFlagCalls).toEqual([
      { key: "rating.enabled", app: "music", enabled: true, rolloutScope: "global", confirm: false },
    ]);
    expect(store.op.status).toBe("success");
    // re-lists after the write (initial load + reload)
    expect(state.listDefinitionsCalls).toHaveLength(2);
  });

  it("edits a typed config value", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useFlagsStore();
    await store.setConfig("rating.review.min_votes", "music", "int", "9", "global", false);
    expect(state.setConfigCalls[0]).toMatchObject({
      key: "rating.review.min_votes",
      app: "music",
      confirm: false,
      value: { kind: { case: "intValue", value: 9n } },
    });
    expect(store.op.status).toBe("success");
  });

  it("records a malformed config edit as an error, sending nothing", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useFlagsStore();
    const outcome = await store.setConfig("rating.review.min_votes", "music", "int", "abc", "global", false);
    expect(outcome.status).toBe("error");
    expect(store.op.status).toBe("error");
    expect(state.setConfigCalls).toHaveLength(0);
  });

  it("clears an override", async () => {
    const { clients, state } = makeFakeClients({ flagDefs: DEFS });
    setClientsForTest(clients);
    const store = useFlagsStore();
    await store.clearOverride("rating.enabled", "music", false);
    expect(state.clearCalls).toEqual([{ key: "rating.enabled", app: "music", confirm: false }]);
  });

  // The campaign-integrity refusals (change: add-flag-campaign-integrity) land as
  // their OWN union cases, so the view can tell "fix the scope" from "retry later".
  it("maps a scope naming no campaign onto its own refusal case", async () => {
    const { clients } = makeFakeClients({ flagDefs: DEFS, failFlagWrite: unknownCampaignErr() });
    setClientsForTest(clients);
    const store = useFlagsStore();
    const outcome = await store.setFlag("rating.enabled", "music", true, "beta:ghost", false);
    expect(outcome).toEqual({ status: "error", error: { kind: "unknownCampaign" } });
    expect(store.op).toEqual(outcome);
  });

  it("maps an unverifiable campaign check onto its own refusal case", async () => {
    const { clients } = makeFakeClients({ flagDefs: DEFS, failFlagWrite: unverifiableErr() });
    setClientsForTest(clients);
    const store = useFlagsStore();
    const outcome = await store.setConfig("rating.review.min_votes", "music", "int", "9", "beta:ghost", false);
    expect(outcome).toEqual({ status: "error", error: { kind: "campaignUnverifiable" } });
  });

  it("keeps an unrelated FAILED_PRECONDITION a generic error, never a campaign refusal", async () => {
    const { clients } = makeFakeClients({
      flagDefs: DEFS,
      failFlagWrite: new ConnectError("some other precondition", Code.FailedPrecondition),
    });
    setClientsForTest(clients);
    const store = useFlagsStore();
    const outcome = await store.setFlag("rating.enabled", "music", true, "global", false);
    expect(outcome.status).toBe("error");
    if (outcome.status === "error") expect(outcome.error.kind).toBe("other");
  });
});

describe("FlagsView", () => {
  beforeEach(() => setActivePinia(createPinia()));

  async function mountView(defs: unknown[] = DEFS, accounts: unknown[] = [], extra: Partial<FakeState> = {}) {
    const { clients, state } = makeFakeClients({ flagDefs: defs, accounts, ...extra });
    setClientsForTest(clients);
    const pinia = createPinia();
    setActivePinia(pinia);
    const w = mount(FlagsView, { global: { plugins: [i18n, pinia] } });
    await flushPromises();
    return { w, state };
  }

  const openEdit = async (w: Awaited<ReturnType<typeof mountView>>["w"]) => {
    await w
      .findAll("tbody tr button")
      .find((b) => b.text() === "Edit")!
      .trigger("click");
    await flushPromises();
  };
  const clickSave = async (w: Awaited<ReturnType<typeof mountView>>["w"]) => {
    await w
      .findAll(".drawer button")
      .find((b) => b.text() === "Save")!
      .trigger("click");
    await flushPromises();
  };

  it("renders a row per declared key with read-only values (no inline edit)", async () => {
    const { w } = await mountView();
    expect(w.findAll("tbody tr").length).toBe(3);
    // no toggle in the row itself anymore
    expect(w.find(".value .toggle").exists()).toBe(false);
    expect(w.find(".drawer").exists()).toBe(false);
  });

  it("toggles a bool flag through the edit drawer", async () => {
    const { w, state } = await mountView([def({ key: "rating.enabled", valueType: "bool" })]);
    await openEdit(w);
    expect(w.find(".drawer").exists()).toBe(true);
    await w.get(".drawer .toggle").trigger("click"); // false -> true
    await clickSave(w);
    expect(state.setFlagCalls[0]).toMatchObject({ key: "rating.enabled", enabled: true, confirm: false });
  });

  it("disables the actions and shows a lock for a non-editable key", async () => {
    const { w, state } = await mountView([def({ key: "x.locked", valueType: "bool", editable: false })]);
    const editBtn = w.findAll("tbody tr button").find((b) => b.text() === "Edit")!;
    expect(editBtn.attributes("disabled")).toBeDefined();
    expect(w.text()).toContain("🔒");
    await editBtn.trigger("click");
    await flushPromises();
    expect(w.find(".drawer").exists()).toBe(false); // guarded open
    expect(state.setFlagCalls).toHaveLength(0);
  });

  it("edits a JSON value field-by-field in the drawer", async () => {
    const jd = def({
      key: "j.cfg",
      valueType: "json",
      defaultValue: { kind: { case: "jsonValue", value: '{"a":1}' } },
      effectiveValue: { kind: { case: "jsonValue", value: '{"a":1}' } },
    });
    const { w, state } = await mountView([jd]);
    await openEdit(w);
    const valInput = w.find(".drawer .jrow .jv");
    expect(valInput.exists()).toBe(true);
    await valInput.setValue("2");
    await clickSave(w);
    expect(state.setConfigCalls[0]).toMatchObject({
      key: "j.cfg",
      value: { kind: { case: "jsonValue", value: '{"a":2}' } },
    });
  });

  it("requires confirmation for a sensitive key, passing confirm=true when accepted", async () => {
    const { w, state } = await mountView([
      def({
        key: "account.min_public_sharing_age",
        app: "all",
        valueType: "int",
        effectiveValue: intVal(16),
        sensitive: true,
      }),
    ]);
    await openEdit(w);
    await w.get(".drawer .scalar").setValue("18");

    // The question is asked in an in-app dialog (never `window.confirm`, which
    // blocks the renderer and is unreachable from e2e / automation).
    await clickSave(w);
    // Target the component, not `dialog`/`.overlay`: the editor drawer is an
    // overlaid <dialog> as well, so a looser selector would drive the drawer.
    const confirmDialog = () => w.findComponent(ConfirmDialog);
    expect(confirmDialog().exists()).toBe(true);
    expect(confirmDialog().text()).toContain("account.min_public_sharing_age");
    expect(state.setConfigCalls).toHaveLength(0); // nothing written yet

    const answer = async (label: string) => {
      await confirmDialog()
        .findAll("button")
        .find((b) => b.text() === label)!
        .trigger("click");
      await flushPromises();
    };
    await answer("Cancel");
    expect(state.setConfigCalls).toHaveLength(0);

    await clickSave(w);
    await answer("Confirm");
    expect(state.setConfigCalls[0]).toMatchObject({
      key: "account.min_public_sharing_age",
      confirm: true,
      value: { kind: { case: "intValue", value: 18n } },
    });
  });

  it("resolves the last-editor uuid to a name from the directory", async () => {
    const uuid = "00000000-0000-0000-0000-0000000000aa";
    const { w } = await mountView(
      [
        def({
          key: "rating.enabled",
          valueType: "bool",
          hasOverride: true,
          updatedBy: uuid,
          updatedAt: "2026-07-31T00:00:00Z",
        }),
      ],
      [{ userId: uuid, handle: "ada", displayName: "Ada Lovelace" }],
    );
    expect(w.text()).toContain("Ada Lovelace");
    // the uuid is only in the title attribute, not shown as text
    expect(w.find(".editor .uid").text()).toBe("Ada Lovelace");
  });

  it("opens the global audit drawer and forwards the key search filter", async () => {
    const { w, state } = await mountView();
    await w
      .findAll("button")
      .find((b) => b.text() === "All changes")!
      .trigger("click");
    await flushPromises();
    expect(w.find(".drawer").exists()).toBe(true);
    await w.get(".drawer input").setValue("rating.enabled");
    await w
      .findAll(".drawer button")
      .find((b) => b.text() === "Search")!
      .trigger("click");
    await flushPromises();
    expect(state.listChangesCalls.at(-1)).toEqual({ appFilter: "", key: "rating.enabled" });
  });

  // --- stale stored scopes (change: add-flag-campaign-integrity) ---

  it("shows a closed campaign's stored scope suffixed, selectable and not as a defect", async () => {
    const { w } = await mountView([def({ key: "drums.enabled", rolloutScope: "beta:midi-drums" })], [], {
      campaigns: CAMPAIGNS,
    });
    await openEdit(w);
    const options = w.get(".drawer select").findAll("option");
    const texts = options.map((o) => o.text());
    // offered list = fixed scopes + OPEN campaigns only; the closed one appears
    // solely as the stored option, quietly suffixed.
    expect(texts).toContain("beta: Spring beta");
    expect(texts).toContain("beta: MIDI drums (closed)");
    expect(texts).not.toContain("beta: MIDI drums");
    const stale = options.find((o) => o.attributes("value") === "beta:midi-drums")!;
    expect(stale.classes()).not.toContain("dangling");
    expect(w.find(".drawer .scope-warn").exists()).toBe(false);
  });

  it("marks a scope naming no campaign as a defect, naming the cause", async () => {
    const { w } = await mountView([def({ key: "drums.enabled", rolloutScope: "beta:ghost" })], [], {
      campaigns: CAMPAIGNS,
    });
    await openEdit(w);
    const options = w.get(".drawer select").findAll("option");
    // still selectable (never silently rewritten), but distinct and explained.
    const stale = options.find((o) => o.attributes("value") === "beta:ghost")!;
    expect(stale.text()).toBe("beta:ghost — names no existing campaign");
    expect(stale.classes()).toContain("dangling");
    expect(w.get(".drawer .scope-warn").text()).toContain("names no existing campaign");
    // the selector stays a closed list: fixed scopes + open campaigns + the stored value.
    expect(options).toHaveLength(5);
  });

  // --- refused writes (change: add-flag-campaign-integrity) ---

  it("surfaces a scope-naming-no-campaign refusal as 'fix the scope', never the raw status", async () => {
    const { w } = await mountView([def({ key: "drums.enabled", valueType: "bool" })], [], {
      failFlagWrite: unknownCampaignErr(),
    });
    await openEdit(w);
    await clickSave(w);
    expect(w.get(".drawer .error").text()).toBe(
      "The selected beta scope names no existing campaign — fix the scope, then save again.",
    );
    expect(w.text()).not.toContain("unknown_campaign");
    expect(w.text()).not.toContain("failed_precondition");
  });

  it("surfaces an unverifiable check as 'retry later', never claiming the campaign is missing", async () => {
    const { w } = await mountView([def({ key: "drums.enabled", valueType: "bool" })], [], {
      failFlagWrite: unverifiableErr(),
    });
    await openEdit(w);
    await clickSave(w);
    const msg = w.get(".drawer .error").text();
    expect(msg).toBe(
      "The campaign directory can’t be reached, so the scope couldn’t be checked. Nothing was saved — try again later.",
    );
    expect(msg).not.toContain("no existing campaign");
    expect(w.text()).not.toContain("campaign_unverifiable");
    expect(w.text()).not.toContain("aborted");
  });
});
