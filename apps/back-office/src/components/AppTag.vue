<script setup lang="ts">
// The back-office's single tag/badge component.
// - Label variants (outline): `accent` (violet, default), `warn` (legal/infra),
//   `neutral` (plain labels).
// - Status variants (soft fill), for the moderation console: `pending`,
//   `accepted`, `rejected`, `review`.
// `mono` renders in the mono font; `cap` capitalizes (role names). Extra attrs
// (e.g. title) fall through to the root span.
withDefaults(
  defineProps<{
    variant?: "accent" | "warn" | "neutral" | "pending" | "accepted" | "rejected" | "review";
    mono?: boolean;
    cap?: boolean;
  }>(),
  { variant: "accent", mono: false, cap: false },
);
</script>

<template>
  <span class="tag" :class="[variant, { mono, cap }]"><slot /></span>
</template>

<style scoped>
.tag {
  display: inline-block;
  padding: 0.08rem 0.45rem;
  border-radius: 999px;
  font-size: 0.72rem;
  font-weight: 600;
  border: 1px solid var(--border);
  white-space: nowrap;
}
.mono {
  font-family: var(--mono);
  font-weight: 500;
}
.cap {
  text-transform: capitalize;
}

/* outline label variants */
.accent {
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
}
.warn {
  color: #d98324;
  border-color: color-mix(in srgb, #d98324 40%, transparent);
}
.neutral {
  color: var(--muted);
}

/* soft-fill status variants (moderation console) */
.pending {
  color: var(--amber);
  background: color-mix(in srgb, var(--amber) 12%, transparent);
  border-color: color-mix(in srgb, var(--amber) 28%, transparent);
}
.accepted {
  color: var(--green);
  background: color-mix(in srgb, var(--green) 12%, transparent);
  border-color: color-mix(in srgb, var(--green) 28%, transparent);
}
.rejected {
  color: var(--coral);
  background: color-mix(in srgb, var(--coral) 12%, transparent);
  border-color: color-mix(in srgb, var(--coral) 28%, transparent);
}
.review {
  color: var(--amber);
  background: color-mix(in srgb, var(--amber) 14%, transparent);
  border-color: color-mix(in srgb, var(--amber) 45%, transparent);
}
</style>
