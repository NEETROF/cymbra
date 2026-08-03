<script setup lang="ts">
import { computed, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { type NewSoundFont, type SoundFontEdit, useSoundFontsStore } from "@/stores/soundfonts";
import { useScorePlayer } from "@/composables/useScorePlayer";
import { uuidv7 } from "@/lib/uuid";
import type { AdminSoundFont, CatalogHit } from "@/gen/score_pb";

// Right-to-left drawer to create OR edit a catalog font (change:
// add-soundfont-back-office-management). On create it uploads the picked `.sf2` +
// metadata; on edit it saves metadata only (id/bytes are immutable — change bytes by
// removing and re-adding). Either way a moderator can audition the sound on a chosen
// catalog piece before saving. The component only calls the store (no direct API).
const props = defineProps<{ mode: "create" | "edit" | null; entry?: AdminSoundFont | null }>();
const emit = defineEmits<{ (e: "close"): void }>();

const store = useSoundFontsStore();
const { t } = useI18n();

const isEdit = computed(() => props.mode === "edit");

// --- Form state ---
const form = ref({ id: "", label: "", license: "CC0-1.0", attribution: "", instrument: "piano" });

// Selectable SoundFont licences (a fixed dropdown, not free text).
const LICENSES = ["CC0-1.0", "CC-BY 3.0", "CC-BY 4.0", "CC-BY-SA 4.0"];
// Instruments a font can be for. Only piano for now (mandatory); more added later.
const INSTRUMENTS = ["piano"];

// Options always include the current value (so an existing font's licence, even if not
// in the predefined list, stays selectable in edit mode).
const licenseOptions = computed(() => {
  const opts = [...LICENSES];
  const cur = form.value.license;
  if (cur && !opts.includes(cur)) opts.unshift(cur);
  return opts;
});

// A plain-language gloss of what the selected licence's acronym means.
const licenseHint = computed(() => {
  const l = form.value.license;
  if (l.startsWith("CC0")) return t("soundfonts.licenseDesc.cc0");
  if (l.startsWith("CC-BY-SA")) return t("soundfonts.licenseDesc.ccbysa");
  if (l.startsWith("CC-BY")) return t("soundfonts.licenseDesc.ccby");
  return "";
});
const file = ref<File | null>(null);

// (Re)seed the form whenever the drawer opens on a new target.
watch(
  () => [props.mode, props.entry?.id] as const,
  () => {
    file.value = null;
    if (props.mode === "edit" && props.entry) {
      const e = props.entry;
      form.value = {
        id: e.id,
        label: e.label,
        license: e.license,
        attribution: e.attribution,
        instrument: e.instrument || "piano",
      };
    } else {
      // The id is auto-minted (uuidv7) and never shown/edited on create.
      form.value = { id: uuidv7(), label: "", license: "CC0-1.0", attribution: "", instrument: "piano" };
    }
  },
  { immediate: true },
);

function onFile(e: Event) {
  file.value = (e.target as HTMLInputElement).files?.[0] ?? null;
}

const canSave = computed(() => {
  const f = form.value;
  const meta = f.label.trim() !== "" && f.license.trim() !== "";
  return isEdit.value ? meta : meta && f.id.trim() !== "" && file.value != null;
});
const acting = computed(() => store.op.status === "loading");
const opError = computed(() => (store.op.status === "error" ? store.op.error : null));

async function save() {
  if (!canSave.value) return;
  if (isEdit.value) {
    const edit: SoundFontEdit = { id: form.value.id, ...metaFields() };
    if ((await store.update(edit)).status === "success") emit("close");
  } else {
    if (!file.value) return;
    const font: NewSoundFont = {
      id: form.value.id.trim(),
      ...metaFields(),
      instrument: form.value.instrument,
      file: file.value,
    };
    if ((await store.add(font)).status === "success") emit("close");
  }
}
function metaFields() {
  return {
    label: form.value.label.trim(),
    license: form.value.license.trim(),
    attribution: form.value.attribution.trim(),
  };
}

// --- Preview (audition the font on a catalog piece) ---
const pieces = ref<CatalogHit[]>([]);
const selectedPiece = ref("");
const scoreBytes = shallowRef<Uint8Array | null>(null);
const sf2Bytes = shallowRef<Uint8Array | null>(null);
// Two independent, self-clearing errors: the chosen piece couldn't load, and (edit
// mode) the stored font couldn't load. Each clears when its own thing succeeds, so a
// stale piece error never lingers after a good piece is selected.
const pieceError = ref<string | null>(null);
const fontError = ref<string | null>(null);
const previewError = computed(() => fontError.value ?? pieceError.value);

const player = useScorePlayer(scoreBytes, sf2Bytes);

// Load a few pieces to choose from when the drawer opens.
watch(
  () => props.mode,
  async (mode) => {
    if (!mode) return;
    selectedPiece.value = "";
    scoreBytes.value = null;
    sf2Bytes.value = null;
    pieceError.value = null;
    fontError.value = null;
    try {
      pieces.value = await store.previewPieces();
      // Default to the first piece so the preview is ready without an extra click.
      selectedPiece.value = pieces.value[0]?.id ?? "";
    } catch {
      pieces.value = [];
    }
    // In edit mode the candidate font is the stored one; fetch it eagerly.
    if (mode === "edit" && props.entry) {
      try {
        sf2Bytes.value = await store.fontBytes(props.entry.id);
      } catch {
        fontError.value = t("soundfonts.previewNoFont");
      }
    }
  },
  { immediate: true },
);

// On create, the candidate font is the picked file.
watch(file, async (f) => {
  player.stop();
  fontError.value = null;
  sf2Bytes.value = f ? new Uint8Array(await f.arrayBuffer()) : null;
});

// Fetch the chosen piece's bytes. Clear any stale piece error first, so a good
// selection after a failed one no longer shows the error.
watch(selectedPiece, async (id) => {
  player.stop();
  pieceError.value = null;
  if (!id) {
    scoreBytes.value = null;
    return;
  }
  try {
    scoreBytes.value = await store.pieceBytes(id);
  } catch {
    scoreBytes.value = null;
    pieceError.value = t("soundfonts.previewNoPiece");
  }
});

const audioState = computed(() => player.audio.value.status);
</script>

<template>
  <Teleport to="body">
    <div v-if="mode" class="overlay" @click.self="emit('close')">
      <dialog class="drawer" open aria-modal="true" @keydown.esc="emit('close')">
        <header>
          <h2>{{ isEdit ? t("soundfonts.editTitle") : t("soundfonts.createTitle") }}</h2>
          <button type="button" class="x" :aria-label="t('soundfonts.cancel')" @click="emit('close')">✕</button>
        </header>

        <p v-if="opError" class="error" role="alert">{{ opError }}</p>

        <form class="body" @submit.prevent="save">
          <!-- The id is auto-generated (uuidv7) and immutable: hidden on create,
               shown read-only on edit. -->
          <label v-if="isEdit">
            <span>{{ t("soundfonts.id") }}</span>
            <input v-model="form.id" aria-label="id" disabled readonly />
          </label>
          <label>
            <span>{{ t("soundfonts.label") }}</span>
            <input v-model="form.label" aria-label="label" />
          </label>
          <label>
            <span>{{ t("soundfonts.instrument") }}</span>
            <!-- Only piano is offered for now (mandatory); the list grows later. -->
            <select v-model="form.instrument" aria-label="instrument" :disabled="isEdit">
              <option v-for="i in INSTRUMENTS" :key="i" :value="i">{{ t(`soundfonts.instr.${i}`) }}</option>
            </select>
          </label>
          <label>
            <span>{{ t("soundfonts.license") }}</span>
            <select v-model="form.license" aria-label="license">
              <option v-for="l in licenseOptions" :key="l" :value="l">{{ l }}</option>
            </select>
            <small v-if="licenseHint" class="fieldhint">{{ licenseHint }}</small>
          </label>
          <label>
            <span>{{ t("soundfonts.attribution") }}</span>
            <input v-model="form.attribution" aria-label="attribution" />
          </label>
          <label v-if="!isEdit">
            <span>{{ t("soundfonts.file") }}</span>
            <input type="file" accept=".sf2" :aria-label="t('soundfonts.file')" @change="onFile" />
          </label>

          <!-- Preview -->
          <fieldset class="preview">
            <legend>{{ t("soundfonts.preview") }}</legend>
            <p class="hint">{{ t("soundfonts.previewHint") }}</p>
            <div class="row">
              <select v-model="selectedPiece" :aria-label="t('soundfonts.choosePiece')">
                <option value="">{{ t("soundfonts.choosePiece") }}</option>
                <option v-for="p in pieces" :key="p.id" :value="p.id">
                  {{ p.title || p.id }}{{ p.composer ? ` — ${p.composer}` : "" }}
                </option>
              </select>
              <button
                type="button"
                class="play"
                :disabled="!player.canPlay.value || sf2Bytes == null"
                @click="player.toggle()"
              >
                {{ player.playing.value ? t("soundfonts.pause") : t("soundfonts.play") }}
              </button>
            </div>
            <p v-if="previewError" class="hint err">{{ previewError }}</p>
            <p v-else-if="audioState === 'loading'" class="hint">…</p>
            <p v-else-if="audioState === 'error'" class="hint err">{{ t("soundfonts.previewFailed") }}</p>
          </fieldset>

          <footer>
            <button type="submit" class="primary" :disabled="!canSave || acting">
              {{ isEdit ? t("soundfonts.save") : t("soundfonts.add") }}
            </button>
            <button type="button" class="ghost" :disabled="acting" @click="emit('close')">
              {{ t("soundfonts.cancel") }}
            </button>
          </footer>
        </form>
      </dialog>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 50;
}
.drawer {
  position: fixed;
  top: 0;
  right: 0;
  /* Cancel the browser's UA `dialog { left: 0 }`, which otherwise wins over `right`
     when the width is fixed and pins the drawer to the LEFT. */
  left: auto;
  width: min(30rem, 94vw);
  height: 100vh;
  max-height: 100vh;
  max-width: none;
  margin: 0;
  border: none;
  border-left: 1px solid var(--outline, #2a2d3a);
  background: var(--surface, #12141c);
  color: var(--fg, #e8e8ef);
  padding: 0;
  display: flex;
  flex-direction: column;
  animation: slide-in 0.18s ease-out;
}
@keyframes slide-in {
  from {
    transform: translateX(100%);
  }
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--outline, #2a2d3a);
}
header h2 {
  margin: 0;
  font-size: 1.1rem;
}
.x {
  background: none;
  border: none;
  color: inherit;
  font-size: 1.1rem;
  cursor: pointer;
}
.body {
  padding: 1.25rem;
  overflow-y: auto;
  overflow-x: hidden;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}
.body label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  min-width: 0;
}
.body input,
.body select {
  padding: 0.45rem 0.55rem;
  width: 100%;
  min-width: 0;
  max-width: 100%;
  box-sizing: border-box;
}
.preview {
  border: 1px solid var(--outline, #2a2d3a);
  border-radius: 0.5rem;
  padding: 0.75rem;
  /* A <fieldset> defaults to `min-width: min-content`, which lets a long <option>
     push it past the drawer width (clipping the hint text). Let it shrink. */
  min-width: 0;
}
.preview .row {
  display: flex;
  gap: 0.5rem;
  min-width: 0;
  /* Breathing room between the help text and the piece selector. */
  margin-top: 0.75rem;
}
.preview select {
  flex: 1;
  min-width: 0;
}
.hint {
  color: var(--muted, #8a8f9c);
  font-size: 0.8rem;
  margin: 0.35rem 0 0;
  overflow-wrap: anywhere;
}
.hint.err {
  color: var(--danger, #e06c75);
}
/* Sub-text explaining the selected licence acronym; sits just under the select,
   spaced only by the label's own gap (no extra top margin). */
.fieldhint {
  color: var(--muted, #8a8f9c);
  font-size: 0.75rem;
  line-height: 1.35;
  overflow-wrap: anywhere;
}
footer {
  display: flex;
  gap: 0.5rem;
  margin-top: auto;
  padding-top: 0.5rem;
}
.primary {
  background: var(--accent, #7c5cff);
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.4rem;
  cursor: pointer;
}
.primary:disabled {
  opacity: 0.5;
}
.ghost {
  background: none;
  border: 1px solid var(--outline, #2a2d3a);
  color: inherit;
  padding: 0.5rem 1rem;
  border-radius: 0.4rem;
  cursor: pointer;
}
.play {
  padding: 0.45rem 0.9rem;
  cursor: pointer;
}
.error {
  color: var(--danger, #e06c75);
  padding: 0 1.25rem;
}
</style>
