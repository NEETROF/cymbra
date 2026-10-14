<script setup lang="ts">
// The Lingua leaving page (spec `site-lingua-page`, change add-lingua-usage-counts D12):
// six fixed answers, no free text. Nothing is sent on load; one tap sends one answer,
// disables the six and thanks the reader whatever the outcome. State + the call live in
// `useLeaveReason`.
import { onMounted } from "vue";
import { useLeaveReason } from "../composables/useLeaveReason";
import { t, type Lang, type MessageKey } from "../lib/i18n";
import { LEAVE_REASONS, SUPPORT_HREF, type LeaveReason } from "../lib/lingua-leave";

const props = defineProps<{
  lang: Lang;
  /** The community invite (`PUBLIC_DISCORD_URL`); `null` leaves its link out. */
  communityUrl: string | null;
}>();
const { chosen, view, boot, choose } = useLeaveReason();

const LABEL: Record<LeaveReason, MessageKey> = {
  not_using: "leaveNotUsing",
  highlighting: "leaveHighlighting",
  missing_language: "leaveMissingLanguage",
  slow: "leaveSlow",
  unclear: "leaveUnclear",
  other: "leaveOther",
};
const supportHref = SUPPORT_HREF[props.lang];

onMounted(() => boot(globalThis.location?.search ?? ""));
</script>

<template>
  <div class="island">
    <section class="card leave">
      <p class="muted">{{ t(lang, "leaveIntro") }}</p>
      <div class="leave-answers" role="group" :aria-label="t(lang, 'leaveQuestion')">
        <button
          v-for="reason in LEAVE_REASONS"
          :key="reason"
          type="button"
          class="btn"
          :data-reason="reason"
          :aria-pressed="chosen === reason"
          :disabled="view.answered"
          @click="choose(reason)"
        >
          {{ t(lang, LABEL[reason]) }}
        </button>
      </div>
      <div role="status">
        <div v-if="view.answered" class="outcome" data-testid="leave-thanks">
          <h3>{{ t(lang, "leaveThanksTitle") }}</h3>
          <p>{{ t(lang, "leaveThanksBody") }}</p>
          <p class="muted">{{ t(lang, "leaveMore") }}</p>
          <p class="cta-row left">
            <a class="btn btn-ghost" :href="supportHref">{{ t(lang, "leaveSupport") }}</a>
            <a v-if="communityUrl" class="btn btn-discord" :href="communityUrl" rel="noopener">{{ t(lang, "leaveCommunity") }}</a>
          </p>
        </div>
      </div>
    </section>
  </div>
</template>
