import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { type FlagRow, toRow } from "./flags";

// The notifications panel (change: add-push-notifications, task 5.1; foreground
// column: add-foreground-notifications).
//
// Push send configuration IS feature-flag configuration — the global kill-switch,
// each category's enable, its local schedule hour, and its foreground
// presentation are declared keys under one prefix, already hot-reloadable and
// audited. So this store is a *view* over the same registry rather than a second
// control plane: it loads the declared keys, keeps only the `notifications.`
// ones, and groups the per-category triplet into a row the panel can render.
//
// Because the set is discovered from the registry, a feature that lands a new
// notification type appears here on its own — nothing in this file changes.

/** Prefix every push-notification key shares (mirrors the backend registry). */
export const NOTIFICATIONS_PREFIX = "notifications.";
/** Prefix of the per-category keys: `notifications.category.<id>.<suffix>`. */
const CATEGORY_PREFIX = `${NOTIFICATIONS_PREFIX}category.`;
/** The global kill-switch key. */
export const KILL_SWITCH_KEY = `${NOTIFICATIONS_PREFIX}enabled`;

/** One notification category's controls, grouped for display. */
export interface CategoryRow {
  /** The category id, e.g. `practice_streak`. */
  category: string;
  /** `notifications.category.<id>.enabled`, absent if the feature declared none. */
  enabled: FlagRow | null;
  /** `notifications.category.<id>.hour` — the LOCAL hour the send fires. */
  hour: FlagRow | null;
  /** `notifications.category.<id>.foreground` — whether a message arriving with
   *  the app open is surfaced in-app (change: add-foreground-notifications). */
  foreground: FlagRow | null;
}

/** Parse `notifications.category.<id>.<suffix>` → `[id, suffix]`, else `null`. */
export function parseCategoryKey(key: string): [string, string] | null {
  if (!key.startsWith(CATEGORY_PREFIX)) return null;
  const rest = key.slice(CATEGORY_PREFIX.length);
  const dot = rest.lastIndexOf(".");
  if (dot <= 0) return null;
  return [rest.slice(0, dot), rest.slice(dot + 1)];
}

/** Group the flat key list into one row per category, ordered by id. */
export function groupCategories(rows: FlagRow[]): CategoryRow[] {
  const byCategory = new Map<string, CategoryRow>();
  for (const row of rows) {
    const parsed = parseCategoryKey(row.key);
    if (!parsed) continue;
    const [category, suffix] = parsed;
    const entry = byCategory.get(category) ?? {
      category,
      enabled: null,
      hour: null,
      foreground: null,
    };
    if (suffix === "enabled") entry.enabled = row;
    else if (suffix === "hour") entry.hour = row;
    else if (suffix === "foreground") entry.foreground = row;
    byCategory.set(category, entry);
  }
  return [...byCategory.values()].sort((a, b) => a.category.localeCompare(b.category));
}

export const useNotificationsStore = defineStore("notifications", () => {
  const definitions = ref<Async<FlagRow[]>>(idle);
  const op = ref<Async<void>>(idle);

  /** The global kill-switch row, `null` until loaded. */
  const killSwitch = computed<FlagRow | null>(() =>
    definitions.value.status === "success"
      ? (definitions.value.data.find((r) => r.key === KILL_SWITCH_KEY) ?? null)
      : null,
  );

  /** One row per declared category. Empty until a feature declares a type. */
  const categories = computed<CategoryRow[]>(() =>
    definitions.value.status === "success" ? groupCategories(definitions.value.data) : [],
  );

  async function load() {
    await run(definitions, async () => {
      // The registry is app-agnostic here: notification keys are server-side send
      // configuration, declared under the shared `all` app.
      const resp = await api().flags.listFlagDefinitions({ appFilter: "" });
      return resp.definitions.map(toRow).filter((r) => r.key.startsWith(NOTIFICATIONS_PREFIX));
    });
  }

  /** Flip the kill-switch or a category's enable. */
  async function setEnabled(row: FlagRow, enabled: boolean) {
    const outcome = await run(op, async () => {
      await api().flags.setFlag({
        key: row.key,
        app: row.app,
        enabled,
        rolloutScope: row.rolloutScope,
        confirm: row.sensitive,
      });
    });
    if (outcome.status === "success") await load();
    return outcome;
  }

  /** Set a category's local schedule hour (0–23). Rejects anything else so a
   *  typo can never schedule a send at an unintended time. */
  async function setHour(row: FlagRow, hour: number) {
    const outcome = await run(op, async () => {
      if (!Number.isInteger(hour) || hour < 0 || hour > 23) {
        throw new Error(`"${hour}" is not an hour between 0 and 23`);
      }
      await api().flags.setConfig({
        key: row.key,
        app: row.app,
        value: { kind: { case: "intValue", value: BigInt(hour) } },
        rolloutScope: row.rolloutScope,
        confirm: row.sensitive,
      });
    });
    if (outcome.status === "success") await load();
    return outcome;
  }

  /** Drop a key's override so it falls back to its code default. */
  async function reset(row: FlagRow) {
    const outcome = await run(op, async () => {
      await api().flags.clearOverride({ key: row.key, app: row.app, confirm: row.sensitive });
    });
    if (outcome.status === "success") await load();
    return outcome;
  }

  /** Drop every stored override of a category (enabled, hour, foreground) so the
   *  whole row falls back to code defaults — the "Reset to default" action. The
   *  override tag lights up for *any* of the three keys, so resetting only one
   *  would leave the tag on and read as a no-op. */
  async function resetCategory(category: CategoryRow) {
    const outcome = await run(op, async () => {
      for (const row of [category.enabled, category.hour, category.foreground]) {
        if (!row?.hasOverride) continue;
        await api().flags.clearOverride({ key: row.key, app: row.app, confirm: row.sensitive });
      }
    });
    if (outcome.status === "success") await load();
    return outcome;
  }

  return {
    definitions,
    op,
    killSwitch,
    categories,
    load,
    setEnabled,
    setHour,
    reset,
    resetCategory,
  };
});
