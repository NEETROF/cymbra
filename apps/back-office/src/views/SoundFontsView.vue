<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { useSoundFontsStore } from "@/stores/soundfonts";
import type { AdminSoundFont } from "@/gen/score_pb";
import AppTag from "@/components/AppTag.vue";
import SoundFontDrawer from "@/components/SoundFontDrawer.vue";

const store = useSoundFontsStore();
const { t } = useI18n();

onMounted(() => {
  void store.list();
});

const vm = computed(() =>
  match(store.catalog)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, rows: [] as AdminSoundFont[] }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, rows: [] as AdminSoundFont[] }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: [] as AdminSoundFont[] }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, rows: data }))
    .exhaustive(),
);

const acting = computed(() => store.op.status === "loading");
const opError = computed(() => (store.op.status === "error" ? store.op.error : null));

// Drawer for create/edit (right-to-left).
const drawerMode = ref<"create" | "edit" | null>(null);
const drawerEntry = ref<AdminSoundFont | null>(null);
function openCreate() {
  drawerEntry.value = null;
  drawerMode.value = "create";
}
function openEdit(row: AdminSoundFont) {
  drawerEntry.value = row;
  drawerMode.value = "edit";
}
function closeDrawer() {
  drawerMode.value = null;
}

async function remove(id: string) {
  if (!window.confirm(t("soundfonts.confirmRemove"))) return;
  await store.remove(id);
}

// Plain-language gloss of a licence acronym, shown as a hover tooltip on the licence
// cell (same mapping as the drawer's help text). Empty for an unrecognised licence.
function licenseDesc(license: string): string {
  if (license.startsWith("CC0")) return t("soundfonts.licenseDesc.cc0");
  if (license.startsWith("CC-BY-SA")) return t("soundfonts.licenseDesc.ccbysa");
  if (license.startsWith("CC-BY")) return t("soundfonts.licenseDesc.ccby");
  return "";
}
</script>

<template>
  <section class="soundfonts">
    <header class="head">
      <div>
        <h1>{{ t("soundfonts.title") }}</h1>
        <p class="intro">{{ t("soundfonts.intro") }}</p>
      </div>
      <button type="button" class="primary" @click="openCreate">{{ t("soundfonts.add") }}</button>
    </header>

    <p v-if="opError" class="error" role="alert">{{ opError }}</p>

    <p v-if="vm.loading" class="muted">…</p>
    <p v-else-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
    <p v-else-if="vm.rows.length === 0" class="muted">{{ t("soundfonts.empty") }}</p>

    <table v-else class="grid">
      <thead>
        <tr>
          <th>{{ t("soundfonts.label") }}</th>
          <th>{{ t("soundfonts.id") }}</th>
          <th>{{ t("soundfonts.instrument") }}</th>
          <th>{{ t("soundfonts.license") }}</th>
          <th>{{ t("soundfonts.attribution") }}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in vm.rows" :key="row.id">
          <td>{{ row.label }}</td>
          <td class="mono">{{ row.id }}</td>
          <td><AppTag variant="neutral">{{ t(`soundfonts.instr.${row.instrument || "piano"}`) }}</AppTag></td>
          <td>
            <span v-if="licenseDesc(row.license)" class="license-help" :title="licenseDesc(row.license)">{{
              row.license
            }}</span>
            <template v-else>{{ row.license }}</template>
          </td>
          <td>{{ row.attribution }}</td>
          <td class="actions">
            <button type="button" @click="openEdit(row)">{{ t("soundfonts.edit") }}</button>
            <button type="button" :disabled="acting" @click="remove(row.id)">{{ t("soundfonts.remove") }}</button>
          </td>
        </tr>
      </tbody>
    </table>

    <SoundFontDrawer :mode="drawerMode" :entry="drawerEntry" @close="closeDrawer" />
  </section>
</template>

<style scoped>
.soundfonts {
  padding: 1.5rem;
  max-width: 60rem;
}
.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}
.intro {
  color: var(--muted, #888);
  margin: 0.25rem 0 1rem;
  max-width: 46rem;
}
.primary {
  background: var(--accent, #7c5cff);
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.4rem;
  cursor: pointer;
  white-space: nowrap;
}
.grid {
  width: 100%;
  border-collapse: collapse;
}
.grid th,
.grid td {
  text-align: left;
  padding: 0.5rem;
  border-bottom: 1px solid var(--outline, #333);
  vertical-align: middle;
}
.mono {
  font-family: ui-monospace, monospace;
  font-size: 0.85em;
}
/* Cue that the licence has a hover explanation (native title tooltip). */
.license-help {
  cursor: help;
  text-decoration: underline dotted;
  text-underline-offset: 2px;
}
.actions {
  display: flex;
  gap: 0.4rem;
}
.error {
  color: var(--danger, #c0392b);
}
.muted {
  color: var(--muted, #888);
}
</style>
