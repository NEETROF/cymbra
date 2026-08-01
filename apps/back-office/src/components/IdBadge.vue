<script setup lang="ts">
// The short catalog id + a click-to-open popover with the full UUID and a copy
// button. Catalog ids are UUID v7, whose leading hex is a millisecond timestamp
// shared across a crawl — so the short form shows the trailing (random) hex,
// and the popover reveals the whole id for copy/paste.
import { onBeforeUnmount, ref } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{ id: string }>();
const { t } = useI18n();

const open = ref(false);
const copied = ref(false);
const root = ref<HTMLElement | null>(null);
let resetTimer: ReturnType<typeof setTimeout> | undefined;

// The random tail distinguishes rows; the timestamp prefix does not.
const short = (id: string): string => id.replaceAll("-", "").slice(-8).toUpperCase();

function toggle(): void {
  open.value = !open.value;
  if (open.value) {
    // Dismiss on any click outside or Escape. Capture so we see the event even
    // if a child stops propagation; the trigger sits inside `root`, so
    // re-clicking it is treated as inside and the trailing `click` toggles shut.
    window.addEventListener("pointerdown", onOutside, true);
    window.addEventListener("keydown", onKey, true);
  } else {
    detach();
  }
}

function close(): void {
  if (!open.value) return;
  open.value = false;
  detach();
}

function detach(): void {
  window.removeEventListener("pointerdown", onOutside, true);
  window.removeEventListener("keydown", onKey, true);
}

function onOutside(e: PointerEvent): void {
  if (root.value && !root.value.contains(e.target as Node)) close();
}

function onKey(e: KeyboardEvent): void {
  if (e.key === "Escape") close();
}

async function copy(): Promise<void> {
  try {
    await navigator.clipboard.writeText(props.id);
    copied.value = true;
    clearTimeout(resetTimer);
    resetTimer = setTimeout(() => (copied.value = false), 1200);
  } catch {
    // Clipboard blocked (e.g. insecure context): keep the popover open so the
    // user can select the id manually. No error surfaced — it's a convenience.
  }
}

onBeforeUnmount(() => {
  detach();
  clearTimeout(resetTimer);
});
</script>

<template>
  <span ref="root" class="id-badge">
    <button type="button" class="trigger" :aria-expanded="open" :title="t('id.reveal')" @click.stop="toggle">
      ID: {{ short(id) }}
    </button>

    <span v-if="open" class="pop" role="dialog" :aria-label="t('id.full')" @click.stop>
      <code class="full">{{ id }}</code>
      <button type="button" class="copy" :class="{ done: copied }" @click="copy">
        {{ copied ? t("id.copied") : t("id.copy") }}
      </button>
    </span>
  </span>
</template>

<style scoped>
.id-badge {
  position: relative;
  display: inline-block;
}

.trigger {
  font-family: var(--mono);
  font-size: 0.68rem;
  color: var(--faint);
  letter-spacing: 0.05em;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}
.trigger:hover,
.trigger[aria-expanded="true"] {
  color: var(--accent);
  text-decoration: underline dotted;
}

.pop {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.5rem;
  background: var(--surface, var(--bg));
  border: 1px solid var(--border-2);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  white-space: nowrap;
}

.full {
  font-family: var(--mono);
  font-size: 0.72rem;
  color: var(--text);
  user-select: all;
}

.copy {
  font-size: 0.68rem;
  font-weight: 600;
  color: var(--accent);
  background: color-mix(in srgb, var(--accent-strong) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
  border-radius: 6px;
  padding: 0.15rem 0.45rem;
  cursor: pointer;
}
.copy:hover {
  background: color-mix(in srgb, var(--accent-strong) 20%, transparent);
}
.copy.done {
  color: var(--green);
  border-color: color-mix(in srgb, var(--green) 40%, transparent);
  background: color-mix(in srgb, var(--green) 14%, transparent);
}
</style>
