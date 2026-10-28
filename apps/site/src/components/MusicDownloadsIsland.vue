<script setup lang="ts">
// The desktop downloads of Cymbra Music (spec `site-music-downloads`): a Windows card
// (the Microsoft Store web installer, or "coming soon" while no product id is
// configured) and a Linux card filled from the backend's latest-release view. The
// server render is the idle state — the releases-page fallback and the Store button —
// so the section works without scripts (design D8). State + call live in
// `useLatestDesktopRelease`; this component only renders.
import { computed, onMounted, ref } from "vue";
import { useLatestDesktopRelease } from "../composables/useLatestDesktopRelease";
import { detectPlatform, type DesktopPlatform, formatSize, msStoreUrl, RELEASES_PAGE } from "../lib/desktop-release";
import { formatDate, t, type Lang } from "../lib/i18n";

const props = defineProps<{ lang: Lang; storeProductId: string | null }>();
const { linux, load } = useLatestDesktopRelease();
const storeUrl = computed(() => msStoreUrl(props.storeProductId));
const platform = ref<DesktopPlatform>(null);

onMounted(() => {
  platform.value = detectPlatform(globalThis.navigator?.userAgent ?? "");
  void load();
});
</script>

<template>
  <div class="desktop-downloads">
    <article class="card" :class="{ highlight: platform === 'windows' }" data-testid="dl-windows">
      <p v-if="platform === 'windows'" class="badge">{{ t(lang, "dlYourSystem") }}</p>
      <h3>{{ t(lang, "dlWindowsTitle") }}</h3>
      <template v-if="storeUrl">
        <p>{{ t(lang, "dlWindowsBody") }}</p>
        <p class="action">
          <a class="btn btn-primary" :href="storeUrl" rel="noopener">{{ t(lang, "dlWindowsButton") }}</a>
        </p>
      </template>
      <p v-else>{{ t(lang, "dlWindowsSoon") }}</p>
    </article>

    <article class="card" :class="{ highlight: platform === 'linux' }" data-testid="dl-linux">
      <p v-if="platform === 'linux'" class="badge">{{ t(lang, "dlYourSystem") }}</p>
      <h3>{{ t(lang, "dlLinuxTitle") }}</h3>

      <template v-if="linux.kind === 'ready'">
        <p>{{ t(lang, "dlLinuxRelease", { version: linux.version, date: formatDate(lang, linux.releasedAt) }) }}</p>
        <p class="action">
          <a class="btn btn-primary" :href="linux.url" rel="noopener" data-testid="dl-linux-download">
            {{ t(lang, "dlLinuxButton", { size: formatSize(lang, linux.size) }) }}
          </a>
        </p>
        <p class="sha">
          <span>{{ t(lang, "dlLinuxSha") }}</span>
          <code data-testid="dl-linux-sha">{{ linux.sha256 }}</code>
        </p>
        <p class="links">
          <a v-if="linux.notesUrl" :href="linux.notesUrl" rel="noopener" data-testid="dl-linux-notes">
            {{ t(lang, "dlLinuxNotes") }}
          </a>
          <a :href="linux.tagUrl" rel="noopener">{{ t(lang, "dlLinuxTarball") }}</a>
        </p>
      </template>
      <template v-else-if="linux.kind === 'unavailable'">
        <p>{{ t(lang, "dlLinuxUnavailable") }}</p>
      </template>
      <p v-else data-testid="dl-linux-fallback">{{ t(lang, "dlLinuxFallback") }}</p>

      <ul class="steps">
        <li>{{ t(lang, "dlLinuxOneFile") }}</li>
        <li>{{ t(lang, "dlLinuxExecutable") }}</li>
        <li>{{ t(lang, "dlLinuxFolder") }}</li>
        <li>{{ t(lang, "dlLinuxFuse") }}</li>
        <li>{{ t(lang, "dlLinuxMenu") }}</li>
      </ul>
      <p class="links">
        <a :href="RELEASES_PAGE" rel="noopener" data-testid="dl-releases">{{ t(lang, "dlReleasesPage") }}</a>
      </p>
    </article>
  </div>
</template>

<style scoped>
.desktop-downloads {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1rem;
  align-items: start;
  text-align: left;
}
.highlight { border-color: var(--accent); order: -1; }
.badge {
  display: inline-block; margin: 0 0 0.5rem; padding: 0.1rem 0.6rem; border-radius: 999px;
  border: 1px solid var(--accent); color: var(--accent); font-size: 0.8rem;
}
.card p + p, .card .steps { margin-top: 0.6rem; }
.action { margin: 0.9rem 0; }
.sha code { display: block; font-size: 0.75rem; word-break: break-all; color: var(--text); }
.links { display: flex; flex-wrap: wrap; gap: 0.4rem 1rem; }
.steps { margin: 0.8rem 0 0; padding-left: 1.2rem; color: var(--muted); font-size: 0.9rem; }
.steps li + li { margin-top: 0.3rem; }
</style>
