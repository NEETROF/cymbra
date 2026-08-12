import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import type { FlagChange, FlagDefinition, FlagValue } from "@/gen/flags_pb";

// The declared value types (mirrors the backend registry / proto `value_type`).
export type FlagKind = "bool" | "int" | "number" | "string" | "json";

/** A declared key row for the panel — decoupled from the wire types. */
export interface FlagRow {
  key: string;
  app: string;
  valueType: FlagKind;
  defaultDisplay: string;
  effectiveDisplay: string;
  effectiveBool: boolean;
  hasOverride: boolean;
  rolloutScope: string;
  sensitive: boolean;
  doc: string;
  /// Whether the signed-in admin may change this key (backend-computed per caller).
  editable: boolean;
  /// Last editor's account id + when (empty when the key is on its code default).
  updatedBy: string;
  updatedAt: string;
}

/** One change-audit row. */
export interface AuditRow {
  key: string;
  app: string;
  oldValue: string;
  newValue: string;
  actor: string;
  at: string;
}

/** Render a wire `FlagValue` as a display string. */
export function readValue(v: FlagValue | undefined): string {
  const k = v?.kind;
  switch (k?.case) {
    case "boolValue":
      return String(k.value);
    case "intValue":
      return k.value.toString();
    case "numberValue":
      return String(k.value);
    case "stringValue":
      return k.value;
    case "jsonValue":
      return k.value;
    default:
      return "";
  }
}

/** Read a wire `FlagValue` as a boolean (false when absent / not a bool). */
export function readBool(v: FlagValue | undefined): boolean {
  return v?.kind?.case === "boolValue" ? v.kind.value : false;
}

/** Build a wire `FlagValue.kind` from user input for a config edit. Throws on a
 *  malformed number/int/JSON so the caller's `run(...)` records it as an error. */
export function buildValueKind(kind: FlagKind, input: string): FlagValue["kind"] {
  switch (kind) {
    case "bool":
      return { case: "boolValue", value: input === "true" };
    case "int": {
      const n = Number(input);
      if (!Number.isInteger(n)) throw new Error(`"${input}" is not an integer`);
      return { case: "intValue", value: BigInt(input) };
    }
    case "number": {
      const n = Number(input);
      if (Number.isNaN(n)) throw new Error(`"${input}" is not a number`);
      return { case: "numberValue", value: n };
    }
    case "string":
      return { case: "stringValue", value: input };
    case "json": {
      const parsed: unknown = JSON.parse(input);
      if (typeof parsed !== "object" || parsed === null) {
        throw new Error("JSON value must be an object or array");
      }
      return { case: "jsonValue", value: JSON.stringify(parsed) };
    }
  }
}

/** Map a wire definition onto the panel row shape (shared with the
 *  notifications panel, which is a filtered view of the same registry). */
export function toRow(d: FlagDefinition): FlagRow {
  return {
    key: d.key,
    app: d.app,
    valueType: d.valueType as FlagKind,
    defaultDisplay: readValue(d.defaultValue),
    effectiveDisplay: readValue(d.effectiveValue),
    effectiveBool: readBool(d.effectiveValue),
    hasOverride: d.hasOverride,
    rolloutScope: d.rolloutScope,
    sensitive: d.sensitive,
    doc: d.doc,
    editable: d.editable,
    updatedBy: d.updatedBy,
    updatedAt: d.updatedAt,
  };
}

function toAuditRow(c: FlagChange): AuditRow {
  return { key: c.key, app: c.app, oldValue: c.oldValue, newValue: c.newValue, actor: c.actor, at: c.at };
}

// All API access goes through the `api()` client seam; components only ever call
// these store actions. Async state is one `Async<T>` union per resource.
export const useFlagsStore = defineStore("flags", () => {
  const definitions = ref<Async<FlagRow[]>>(idle);
  const audit = ref<Async<AuditRow[]>>(idle); // global audit (searchable)
  const keyAudit = ref<Async<AuditRow[]>>(idle); // per-key audit (drawer)
  const op = ref<Async<void>>(idle);
  const appFilter = ref("");
  // uuid → display name, resolved once from the admin directory (the flags schema
  // is isolated and only stores actor uuids). Best-effort: falls back to the uuid.
  const directory = ref<Record<string, string>>({});

  async function loadDirectory() {
    try {
      const resp = await api().user.listAccounts({ query: "", limit: 200, offset: 0 });
      const map: Record<string, string> = {};
      for (const a of resp.accounts) map[a.userId] = a.displayName || a.handle || a.userId;
      directory.value = map;
    } catch {
      // best-effort; the UI falls back to the uuid.
    }
  }

  /// The display name for an actor uuid, or a shortened uuid when unknown.
  function nameFor(uuid: string): string {
    if (!uuid) return "";
    return directory.value[uuid] ?? (uuid.length > 12 ? `${uuid.slice(0, 8)}…` : uuid);
  }

  async function load(app: string = appFilter.value) {
    appFilter.value = app;
    await run(definitions, async () => {
      const resp = await api().flags.listFlagDefinitions({ appFilter: app });
      return resp.definitions.map(toRow);
    });
  }

  // Global, searchable audit (by app and/or key).
  async function loadAudit(app = "", key = "") {
    await run(audit, async () => {
      const resp = await api().flags.listFlagChanges({ appFilter: app, key, limit: 100 });
      return resp.changes.map(toAuditRow);
    });
  }

  // Per-parameter audit for the edit drawer.
  async function loadKeyAudit(app: string, key: string) {
    await run(keyAudit, async () => {
      const resp = await api().flags.listFlagChanges({ appFilter: app, key, limit: 50 });
      return resp.changes.map(toAuditRow);
    });
  }

  async function refresh() {
    await load();
  }

  async function setFlag(key: string, app: string, enabled: boolean, rolloutScope: string, confirm: boolean) {
    const outcome = await run(op, async () => {
      await api().flags.setFlag({ key, app, enabled, rolloutScope, confirm });
    });
    if (outcome.status === "success") await refresh();
    return outcome;
  }

  async function setConfig(
    key: string,
    app: string,
    kind: FlagKind,
    input: string,
    rolloutScope: string,
    confirm: boolean,
  ) {
    const outcome = await run(op, async () => {
      const value: { kind: FlagValue["kind"] } = { kind: buildValueKind(kind, input) };
      await api().flags.setConfig({ key, app, value, rolloutScope, confirm });
    });
    if (outcome.status === "success") await refresh();
    return outcome;
  }

  async function clearOverride(key: string, app: string, confirm: boolean) {
    const outcome = await run(op, async () => {
      await api().flags.clearOverride({ key, app, confirm });
    });
    if (outcome.status === "success") await refresh();
    return outcome;
  }

  return {
    definitions,
    audit,
    keyAudit,
    op,
    appFilter,
    directory,
    nameFor,
    load,
    loadAudit,
    loadKeyAudit,
    loadDirectory,
    refresh,
    setFlag,
    setConfig,
    clearOverride,
  };
});
