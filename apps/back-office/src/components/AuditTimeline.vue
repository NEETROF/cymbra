<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { AuditRow } from "@/stores/flags";
import AppTag from "./AppTag.vue";

const props = defineProps<{
  entries: AuditRow[];
  loading?: boolean;
  // Show the app/key on each entry (used by the global audit view).
  showKey?: boolean;
  // Resolve an actor uuid to a display name (falls back to the uuid).
  resolve?: (uuid: string) => string;
}>();

const { t } = useI18n();
const actorName = (uuid: string) => (props.resolve ? props.resolve(uuid) : uuid);

function when(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}
</script>

<template>
  <div class="timeline">
    <p v-if="loading" class="muted">{{ t("common.loading") }}</p>
    <p v-else-if="!entries.length" class="muted">{{ t("flags.noAudit") }}</p>
    <ol v-else>
      <li v-for="(c, i) in entries" :key="i">
        <span class="dot">{{ entries.length - i }}</span>
        <div class="body">
          <div class="who">
            <span class="actor" :title="c.actor">{{ actorName(c.actor) }}</span>
            <span class="at">{{ when(c.at) }}</span>
          </div>
          <div class="val">
            <AppTag v-if="showKey" variant="neutral" mono>{{ c.app }}/{{ c.key }}</AppTag>
            <code class="to">{{ c.newValue }}</code>
            <span v-if="c.oldValue" class="from">← {{ c.oldValue }}</span>
          </div>
        </div>
      </li>
    </ol>
  </div>
</template>

<style scoped>
.timeline ol {
  list-style: none;
  margin: 0;
  padding: 0;
}
.timeline li {
  display: flex;
  gap: 0.7rem;
  padding-bottom: 1rem;
  position: relative;
}
/* connecting line */
.timeline li:not(:last-child)::before {
  content: "";
  position: absolute;
  left: 12px;
  top: 26px;
  bottom: 0;
  width: 1px;
  background: var(--border);
}
.dot {
  flex: 0 0 24px;
  height: 24px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  font-size: 0.72rem;
  border: 1px solid var(--border);
  color: var(--muted);
  background: var(--bg-deep, #14141c);
  z-index: 1;
}
.timeline li:first-child .dot {
  border-color: var(--accent);
  color: var(--accent);
}
.body {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}
.who {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.5rem;
}
.actor {
  font-size: 0.82rem;
  word-break: break-all;
}
.at {
  color: var(--muted);
  font-size: 0.75rem;
}
.val {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.to {
  color: var(--text);
}
.from {
  color: var(--muted);
  font-size: 0.78rem;
}
.muted {
  color: var(--muted);
  font-size: 0.85rem;
}
</style>
