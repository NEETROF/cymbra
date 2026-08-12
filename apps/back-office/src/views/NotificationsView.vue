<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { type CategoryRow, useNotificationsStore } from "@/stores/notifications";
import type { FlagRow } from "@/stores/flags";
import AppTag from "@/components/AppTag.vue";

// The back-office "Notifications" screen (change: add-push-notifications, task
// 5.1): the global push kill-switch, plus each declared category's enable and its
// local schedule hour. It NEVER calls the API directly — the Pinia store does,
// behind the injectable client seam — and every async resource is a single
// ts-pattern union, matched exhaustively. Admin-scope gated by the router.

const store = useNotificationsStore();
const { t } = useI18n();

onMounted(() => void store.load());

const vm = computed(() =>
  match(store.definitions)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null }))
    .with({ status: "loading" }, () => ({ loading: true, error: null }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error }))
    .with({ status: "success" }, () => ({ loading: false, error: null }))
    .exhaustive(),
);

// The last write's outcome — shown next to the controls, never thrown.
const opError = computed(() =>
  match(store.op)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);
const busy = computed(() => store.op.status === "loading" || store.definitions.status === "loading");

/** Hour currently being edited, keyed by category — `undefined` while idle. */
const hourDraft = ref<Record<string, string>>({});

function draftFor(row: CategoryRow): string {
  return hourDraft.value[row.category] ?? row.hour?.effectiveDisplay ?? "";
}

function toggle(row: FlagRow) {
  void store.setEnabled(row, !row.effectiveBool);
}

function saveHour(row: CategoryRow) {
  if (!row.hour) return;
  void store.setHour(row.hour, Number(draftFor(row)));
}
</script>

<template>
  <section class="notifications">
    <header class="head">
      <div>
        <h1>{{ t("notifications.title") }}</h1>
        <p class="intro">{{ t("notifications.intro") }}</p>
      </div>
    </header>

    <p v-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
    <p v-if="opError" class="error" role="alert" data-testid="op-error">{{ opError }}</p>
    <p v-if="vm.loading" class="muted">{{ t("common.loading") }}</p>

    <template v-else>
      <!-- Global kill-switch: off suppresses every category, whatever the rest says. -->
      <div class="card kill">
        <div class="labels">
          <h2>{{ t("notifications.killSwitch") }}</h2>
          <p class="doc">{{ t("notifications.killSwitchDoc") }}</p>
        </div>
        <button
          v-if="store.killSwitch"
          type="button"
          class="toggle"
          :class="{ on: store.killSwitch.effectiveBool }"
          :disabled="busy || !store.killSwitch.editable"
          data-testid="kill-switch"
          @click="toggle(store.killSwitch)"
        >
          {{ store.killSwitch.effectiveBool ? t("notifications.on") : t("notifications.off") }}
        </button>
        <span v-else class="muted" data-testid="kill-switch-missing">{{ t("notifications.keyMissing") }}</span>
      </div>

      <!-- One row per declared category. The list is discovered from the flag
           registry, so a feature's new notification type shows up on its own. -->
      <p v-if="store.categories.length === 0" class="muted" data-testid="no-categories">
        {{ t("notifications.noCategories") }}
      </p>

      <div v-else class="table-card">
        <table>
          <thead>
            <tr>
              <th>{{ t("notifications.colCategory") }}</th>
              <th>{{ t("notifications.colEnabled") }}</th>
              <th>{{ t("notifications.colHour") }}</th>
              <th>{{ t("notifications.colActions") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in store.categories" :key="row.category" :data-testid="`category-${row.category}`">
              <td>
                <code>{{ row.category }}</code>
              </td>
              <td>
                <button
                  v-if="row.enabled"
                  type="button"
                  class="toggle"
                  :class="{ on: row.enabled.effectiveBool }"
                  :disabled="busy || !row.enabled.editable"
                  :data-testid="`enable-${row.category}`"
                  @click="toggle(row.enabled)"
                >
                  {{ row.enabled.effectiveBool ? t("notifications.on") : t("notifications.off") }}
                </button>
                <span v-else class="muted">{{ t("notifications.keyMissing") }}</span>
              </td>
              <td class="hour">
                <template v-if="row.hour">
                  <input
                    type="number"
                    min="0"
                    max="23"
                    :value="draftFor(row)"
                    :aria-label="t('notifications.hourFor', { category: row.category })"
                    :disabled="busy || !row.hour.editable"
                    :data-testid="`hour-${row.category}`"
                    @input="hourDraft[row.category] = ($event.target as HTMLInputElement).value"
                  />
                  <button
                    type="button"
                    class="btn-sm"
                    :disabled="busy || !row.hour.editable"
                    :data-testid="`save-hour-${row.category}`"
                    @click="saveHour(row)"
                  >
                    {{ t("notifications.save") }}
                  </button>
                </template>
                <span v-else class="muted">{{ t("notifications.eventTriggered") }}</span>
              </td>
              <td>
                <AppTag v-if="row.enabled?.hasOverride || row.hour?.hasOverride">{{
                  t("notifications.override")
                }}</AppTag>
                <button
                  v-if="row.enabled"
                  type="button"
                  class="btn-sm"
                  :disabled="busy || !row.enabled.editable"
                  :data-testid="`reset-${row.category}`"
                  @click="store.reset(row.enabled)"
                >
                  {{ t("notifications.reset") }}
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </section>
</template>

<style scoped>
.head {
  margin-bottom: 1rem;
}
.intro {
  color: var(--muted);
  max-width: 60ch;
}
.card {
  border: 1px solid var(--line);
  border-radius: 0.6rem;
  padding: 1rem;
  margin-bottom: 1.25rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}
.labels h2 {
  font-size: 1rem;
  margin: 0 0 0.25rem;
}
.doc {
  color: var(--muted);
  margin: 0;
  max-width: 55ch;
}
.toggle {
  min-width: 4rem;
}
.toggle.on {
  border-color: var(--accent);
  color: var(--accent);
}
.table-card {
  border: 1px solid var(--line);
  border-radius: 0.6rem;
  overflow-x: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  text-align: left;
  padding: 0.6rem 0.75rem;
  border-bottom: 1px solid var(--line);
}
tbody tr:last-child td {
  border-bottom: none;
}
.hour {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.hour input {
  width: 4.5rem;
}
.muted {
  color: var(--muted);
}
.error {
  color: var(--danger, #d33);
}
</style>
