<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { type FlagRow, useFlagsStore } from "@/stores/flags";
import { flagDescription } from "@/i18n/flag-descriptions";
import { appLabel } from "@/i18n/app-label";
import FlagDrawer from "@/components/FlagDrawer.vue";
import GlobalAuditDrawer from "@/components/GlobalAuditDrawer.vue";
import AppTag from "@/components/AppTag.vue";

const store = useFlagsStore();
const { t, locale } = useI18n();

onMounted(() => {
  void store.load("");
  void store.loadDirectory(); // resolve actor uuids → names
});

const vm = computed(() =>
  match(store.definitions)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, rows: [] as FlagRow[] }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, rows: [] as FlagRow[] }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: [] as FlagRow[] }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, rows: data }))
    .exhaustive(),
);

// App chips derive from the FULL set (stable); filtering is client-side.
const apps = computed(() => [...new Set(vm.value.rows.map((r) => r.app))].sort());
const selectedApp = ref("");
const visibleRows = computed(() =>
  selectedApp.value === "" ? vm.value.rows : vm.value.rows.filter((r) => r.app === selectedApp.value),
);

const acting = computed(() => store.op.status === "loading");
// Fold the write outcome's error union into the drawer's message. The two
// campaign-integrity refusals read differently on purpose (change:
// add-flag-campaign-integrity): a scope naming no campaign says fix the scope,
// an unverifiable check says retry later — never that the campaign is missing.
const opError = computed(() =>
  store.op.status === "error"
    ? match(store.op.error)
        .with({ kind: "unknownCampaign" }, () => t("flags.scopeNoCampaign"))
        .with({ kind: "campaignUnverifiable" }, () => t("flags.scopeUnverifiable"))
        .with({ kind: "other" }, ({ message }) => message)
        .exhaustive()
    : null,
);

const desc = (r: FlagRow) => flagDescription(r.key, r.doc, locale.value);
// Plan-/beta-scoped rollouts are marked in the row so an operator sees at a glance
// which features are gated by plan or by beta (change: add-premium-subscription).
const scopeTags = (r: FlagRow): { label: string; title: string }[] => {
  const s = r.rolloutScope;
  if (s === "premium_only") return [{ label: t("flags.planTag"), title: s }];
  if (s.startsWith("beta:")) return [{ label: t("flags.betaTag"), title: s }];
  if (s === "staff_only") return [{ label: t("flags.staffTag"), title: s }];
  return [];
};
const short = (s: string) => (s.length > 42 ? `${s.slice(0, 42)}…` : s);
function dateShort(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString();
}

// --- per-row edit drawer ---
const activeRow = ref<FlagRow | null>(null);
const keyAuditVm = computed(() =>
  match(store.keyAudit)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => []),
);
const keyAuditLoading = computed(() => store.keyAudit.status === "loading");

function openDrawer(r: FlagRow) {
  if (!r.editable) return;
  activeRow.value = r;
  void store.loadKeyAudit(r.app, r.key);
}
function closeDrawer() {
  activeRow.value = null;
}

function confirmSensitive(r: FlagRow): boolean | null {
  if (!r.sensitive) return false;
  if (globalThis.confirm && !globalThis.confirm(t("flags.sensitiveConfirm", { key: r.key }))) return null;
  return true;
}

// Re-point activeRow at the freshly-loaded row so the drawer shows the new state.
function syncActive() {
  if (!activeRow.value) return;
  const cur = activeRow.value;
  const rows = store.definitions.status === "success" ? store.definitions.data : [];
  activeRow.value = rows.find((r) => r.app === cur.app && r.key === cur.key) ?? cur;
}

async function onSave(payload: { input: string; rolloutScope: string }) {
  const r = activeRow.value;
  if (!r) return;
  const confirm = confirmSensitive(r);
  if (confirm === null) return;
  const outcome =
    r.valueType === "bool"
      ? await store.setFlag(r.key, r.app, payload.input === "true", payload.rolloutScope, confirm)
      : await store.setConfig(r.key, r.app, r.valueType, payload.input, payload.rolloutScope, confirm);
  if (outcome.status === "success") {
    syncActive();
    void store.loadKeyAudit(r.app, r.key);
  }
}

async function onClear() {
  const r = activeRow.value;
  if (!r) return;
  const confirm = confirmSensitive(r);
  if (confirm === null) return;
  const outcome = await store.clearOverride(r.key, r.app, confirm);
  if (outcome.status === "success") {
    syncActive();
    void store.loadKeyAudit(r.app, r.key);
  }
}

// --- global audit drawer ---
const globalOpen = ref(false);
const globalAuditVm = computed(() =>
  match(store.audit)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => []),
);
const globalLoading = computed(() => store.audit.status === "loading");

function openGlobal() {
  globalOpen.value = true;
  void store.loadAudit("", "");
}
</script>

<template>
  <section class="flags">
    <header class="head">
      <div>
        <h1>{{ t("flags.title") }}</h1>
        <p class="intro">{{ t("flags.intro") }}</p>
      </div>
      <button type="button" class="audit-btn" @click="openGlobal">{{ t("flags.globalChanges") }}</button>
    </header>

    <div class="filter" role="toolbar" :aria-label="t('flags.filterLabel')">
      <button type="button" :class="{ active: selectedApp === '' }" @click="selectedApp = ''">
        {{ t("flags.allApps") }}
      </button>
      <button v-for="a in apps" :key="a" type="button" :class="{ active: selectedApp === a }" @click="selectedApp = a">
        {{ appLabel(a, t) }}
      </button>
    </div>

    <p v-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
    <!-- Flag save/clear errors surface in the edit drawer (where the action happens);
         only the list load error stays inline here. -->
    <p v-if="vm.loading" class="muted">{{ t("common.loading") }}</p>

    <div v-else class="table-card">
      <table>
        <thead>
          <tr>
            <th>{{ t("flags.colApp") }}</th>
            <th>{{ t("flags.colKey") }}</th>
            <th>{{ t("flags.colValue") }}</th>
            <th>{{ t("flags.colEditor") }}</th>
            <th>{{ t("flags.colActions") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in visibleRows" :key="`${r.app}/${r.key}`" :class="{ sensitive: r.sensitive }">
            <td>
              <AppTag variant="neutral" :mono="r.app !== 'all'">{{ appLabel(r.app, t) }}</AppTag>
            </td>
            <td>
              <div class="key">
                <span class="keyline">
                  <code>{{ r.key }}</code>
                  <span v-if="!r.editable" class="lock" :title="t('flags.locked')">🔒</span>
                  <AppTag v-if="r.sensitive" variant="warn" :title="t('flags.sensitiveHint')">{{
                    t("flags.sensitive")
                  }}</AppTag>
                </span>
                <span class="doc">{{ desc(r) }}</span>
              </div>
            </td>
            <td class="value">
              <div class="stack">
                <code>{{
                  r.valueType === "bool"
                    ? r.effectiveBool
                      ? t("flags.on")
                      : t("flags.off")
                    : short(r.effectiveDisplay)
                }}</code>
                <AppTag v-if="r.hasOverride">{{ t("flags.override") }}</AppTag>
                <AppTag v-for="tag in scopeTags(r)" :key="tag.label" variant="warn" mono :title="tag.title">
                  {{ tag.label }}
                </AppTag>
              </div>
            </td>
            <td>
              <div class="editor">
                <template v-if="r.updatedBy">
                  <span class="uid" :title="r.updatedBy">{{ store.nameFor(r.updatedBy) }}</span>
                  <span class="when">{{ dateShort(r.updatedAt) }}</span>
                </template>
                <template v-else>
                  <span class="dash">—</span>
                  <AppTag>{{ t("flags.defaultTag") }}</AppTag>
                </template>
              </div>
            </td>
            <td class="actions-col">
              <button type="button" class="btn-sm" :disabled="!r.editable || acting" @click="openDrawer(r)">
                {{ t("flags.edit") }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <FlagDrawer
      :row="activeRow"
      :audit="keyAuditVm"
      :audit-loading="keyAuditLoading"
      :busy="acting"
      :error="opError"
      :resolve="store.nameFor"
      @save="onSave"
      @clear="onClear"
      @close="closeDrawer"
    />

    <GlobalAuditDrawer
      :open="globalOpen"
      :apps="apps"
      :audit="globalAuditVm"
      :loading="globalLoading"
      :resolve="store.nameFor"
      @search="({ app, key }) => store.loadAudit(app, key)"
      @close="globalOpen = false"
    />
  </section>
</template>

<style scoped>
.flags {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}
.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}
.head h1 {
  margin: 0;
}
.intro {
  color: var(--muted);
  max-width: 62ch;
}
.audit-btn {
  flex: none;
}
.filter {
  display: inline-flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}
.filter button.active {
  border-color: var(--accent);
  color: var(--accent);
}
/* Stacked cells (key + doc, value + override, editor) read better top-aligned; the
   card container, header, dividers and padding come from the global table styles. */
.table-card td {
  vertical-align: top;
}
.actions-col {
  width: 1%;
  white-space: nowrap;
}
.btn-sm {
  padding: 0.32rem 0.6rem;
  font-size: 0.8rem;
  white-space: nowrap;
}
.key {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}
.keyline {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}
.lock {
  font-size: 0.8rem;
  opacity: 0.7;
}
.doc {
  color: var(--muted);
  font-size: 0.8rem;
}
.value .stack {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.28rem;
}
.editor {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.28rem;
}
.editor .dash {
  color: var(--muted);
}
.editor .uid {
  font-size: 0.8rem;
}
.editor .when {
  color: var(--muted);
  font-size: 0.75rem;
}
.ghost {
  background: transparent;
  color: var(--muted);
}
tr.sensitive .key code {
  color: var(--amber);
}
.error {
  color: var(--coral);
}
.muted {
  color: var(--muted);
}
</style>
