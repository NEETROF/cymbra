<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { AuditRow, FlagRow } from "@/stores/flags";
import { flagDescription } from "@/i18n/flag-descriptions";
import { appLabel } from "@/i18n/app-label";
import AuditTimeline from "./AuditTimeline.vue";
import AppTag from "./AppTag.vue";

const props = defineProps<{
  row: FlagRow | null;
  audit: AuditRow[];
  auditLoading?: boolean;
  busy?: boolean;
  error?: string | null;
  resolve?: (uuid: string) => string;
}>();

const emit = defineEmits<{
  // The edited value as a string (bool: "true"/"false"; json: JSON string) + rollout.
  (e: "save", payload: { input: string; rolloutScope: string }): void;
  (e: "clear"): void;
  (e: "close"): void;
}>();

const { t, locale } = useI18n();

const boolVal = ref(false);
const scalarVal = ref("");
const rollout = ref("global");
type Row = { key: string; val: string };
const jsonMode = ref<"object" | "array" | "scalar">("object");
const jsonRows = ref<Row[]>([]);

const isJson = computed(() => props.row?.valueType === "json");
const isBool = computed(() => props.row?.valueType === "bool");

function toInput(v: unknown): string {
  if (v === null) return "null";
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}
function coerce(s: string): unknown {
  const x = s.trim();
  if (x === "true") return true;
  if (x === "false") return false;
  if (x === "null") return null;
  if (x !== "" && !Number.isNaN(Number(x))) return Number(x);
  if (x.startsWith("{") || x.startsWith("[")) {
    try {
      return JSON.parse(x);
    } catch {
      /* keep string */
    }
  }
  return s;
}

// (Re)seed the editor whenever the drawer opens on a new row.
watch(
  () => props.row,
  (row) => {
    if (!row) return;
    rollout.value = row.rolloutScope || "global";
    if (row.valueType === "bool") {
      boolVal.value = row.effectiveBool;
    } else if (row.valueType === "json") {
      try {
        const parsed: unknown = JSON.parse(row.effectiveDisplay || "null");
        if (Array.isArray(parsed)) {
          jsonMode.value = "array";
          jsonRows.value = parsed.map((v) => ({ key: "", val: toInput(v) }));
        } else if (parsed && typeof parsed === "object") {
          jsonMode.value = "object";
          jsonRows.value = Object.entries(parsed).map(([k, v]) => ({ key: k, val: toInput(v) }));
        } else {
          jsonMode.value = "scalar";
          scalarVal.value = row.effectiveDisplay;
        }
      } catch {
        jsonMode.value = "scalar";
        scalarVal.value = row.effectiveDisplay;
      }
    } else {
      scalarVal.value = row.effectiveDisplay;
    }
  },
  { immediate: true },
);

const canSave = computed(() => {
  if (isJson.value && jsonMode.value === "object") {
    const keys = jsonRows.value.map((r) => r.key.trim());
    return keys.every((k) => k !== "") && new Set(keys).size === keys.length;
  }
  return true;
});

function addRow() {
  jsonRows.value.push({ key: "", val: "" });
}
function removeRow(i: number) {
  jsonRows.value.splice(i, 1);
}

function buildInput(): string {
  if (isBool.value) return boolVal.value ? "true" : "false";
  if (isJson.value) {
    if (jsonMode.value === "array") return JSON.stringify(jsonRows.value.map((r) => coerce(r.val)));
    if (jsonMode.value === "object")
      return JSON.stringify(Object.fromEntries(jsonRows.value.map((r) => [r.key.trim(), coerce(r.val)])));
    return scalarVal.value;
  }
  return scalarVal.value;
}

function save() {
  emit("save", { input: buildInput(), rolloutScope: rollout.value });
}

const desc = computed(() => (props.row ? flagDescription(props.row.key, props.row.doc, locale.value) : ""));
</script>

<template>
  <div v-if="row" class="overlay" @click.self="emit('close')">
    <dialog class="drawer" open aria-modal="true" @keydown.esc="emit('close')">
      <header>
        <div class="titles">
          <code class="k">{{ row.key }}</code>
          <div class="tags">
            <AppTag variant="neutral" :mono="row.app !== 'all'">{{ appLabel(row.app, t) }}</AppTag>
            <AppTag v-if="row.sensitive" variant="warn">{{ t("flags.sensitive") }}</AppTag>
            <AppTag variant="neutral">{{ row.valueType }}</AppTag>
          </div>
        </div>
        <button type="button" class="x" :aria-label="t('flags.cancel')" @click="emit('close')">✕</button>
      </header>
      <p class="desc">{{ desc }}</p>

      <p v-if="error" class="error" role="alert">{{ error }}</p>

      <section class="field">
        <h4>{{ t("flags.colValue") }}</h4>

        <!-- boolean -->
        <button
          v-if="isBool"
          type="button"
          class="toggle"
          :class="{ on: boolVal }"
          :disabled="busy"
          @click="boolVal = !boolVal"
        >
          {{ boolVal ? t("flags.on") : t("flags.off") }}
        </button>

        <!-- json / list -->
        <template v-else-if="isJson">
          <div class="rows">
            <div v-for="(r, i) in jsonRows" :key="i" class="jrow">
              <input
                v-if="jsonMode === 'object'"
                v-model="r.key"
                class="jk"
                :placeholder="t('flags.jsonKey')"
                :aria-label="t('flags.jsonKey')"
                :disabled="busy"
              />
              <span v-else class="jidx">{{ i }}</span>
              <input
                v-model="r.val"
                class="jv"
                :placeholder="t('flags.jsonValue')"
                :aria-label="t('flags.jsonValue')"
                :disabled="busy"
              />
              <button
                type="button"
                class="ghost"
                :disabled="busy"
                :aria-label="t('flags.remove')"
                @click="removeRow(i)"
              >
                ✕
              </button>
            </div>
            <input
              v-if="jsonMode === 'scalar'"
              v-model="scalarVal"
              class="jv"
              :aria-label="t('flags.editValue', { key: row.key })"
              :disabled="busy"
            />
          </div>
          <button v-if="jsonMode !== 'scalar'" type="button" class="add" :disabled="busy" @click="addRow">
            + {{ jsonMode === "array" ? t("flags.jsonAddItem") : t("flags.jsonAddEntry") }}
          </button>
        </template>

        <!-- scalar (int / number / string) -->
        <input
          v-else
          v-model="scalarVal"
          class="scalar"
          :aria-label="t('flags.editValue', { key: row.key })"
          :disabled="busy"
        />

        <p class="hint">
          {{ t("flags.colDefault") }}: <code>{{ row.defaultDisplay }}</code>
        </p>
      </section>

      <section class="field">
        <h4>{{ t("flags.colRollout") }}</h4>
        <select v-model="rollout" :aria-label="t('flags.rolloutFor', { key: row.key })" :disabled="busy">
          <option value="global">{{ t("flags.global") }}</option>
          <option value="staff_only">{{ t("flags.staffOnly") }}</option>
        </select>
      </section>

      <div class="actions">
        <button type="button" class="primary" :disabled="busy || !canSave" @click="save">{{ t("flags.save") }}</button>
        <button v-if="row.hasOverride" type="button" class="ghost" :disabled="busy" @click="emit('clear')">
          {{ t("flags.reset") }}
        </button>
        <button type="button" class="ghost" :disabled="busy" @click="emit('close')">{{ t("flags.cancel") }}</button>
      </div>

      <section class="field audit">
        <h4>{{ t("flags.auditTitle") }}</h4>
        <AuditTimeline :entries="audit" :loading="auditLoading" :resolve="resolve" />
      </section>
    </dialog>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 90;
}
.drawer {
  position: fixed;
  top: 0;
  right: 0;
  /* Cancel the browser's UA `dialog { left: 0 }`, which otherwise wins over `right`
     when the width is fixed and pins the drawer to the LEFT. */
  left: auto;
  height: 100vh;
  max-height: 100vh;
  width: min(460px, 94vw);
  max-width: none;
  margin: 0;
  overflow-y: auto;
  background: var(--panel, #1a1a24);
  color: inherit;
  border: none;
  border-left: 1px solid var(--border);
  padding: 1.35rem 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
  animation: slideIn 0.22s ease;
}
@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}
header {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
}
.titles {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.k {
  font-size: 0.95rem;
  word-break: break-all;
}
.tags {
  display: flex;
  gap: 0.35rem;
  flex-wrap: wrap;
}
.x {
  background: transparent;
  color: var(--muted);
  padding: 0.2rem 0.5rem;
}
.desc {
  color: var(--muted);
  font-size: 0.85rem;
  margin: 0;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.field h4 {
  margin: 0;
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted);
}
.toggle {
  align-self: flex-start;
  min-width: 4rem;
}
.toggle.on {
  border-color: var(--accent);
  color: var(--accent);
}
.scalar,
.jv,
.jk {
  width: 100%;
}
.rows {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.jrow {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}
.jrow .jk {
  flex: 0 0 42%;
  font-family: var(--mono);
}
.jrow .jv {
  flex: 1;
}
.jidx {
  flex: 0 0 1.5rem;
  text-align: right;
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.8rem;
}
.add {
  align-self: flex-start;
  background: transparent;
  color: var(--accent);
}
.hint {
  margin: 0;
  color: var(--muted);
  font-size: 0.78rem;
}
.actions {
  display: flex;
  gap: 0.5rem;
  border-top: 1px solid var(--border);
  padding-top: 0.9rem;
}
.primary {
  border-color: var(--accent);
  color: var(--accent);
}
.ghost {
  background: transparent;
  color: var(--muted);
}
.audit {
  border-top: 1px solid var(--border);
  padding-top: 0.9rem;
}
.error {
  color: var(--danger, #e55);
  font-size: 0.85rem;
  margin: 0;
}
</style>
