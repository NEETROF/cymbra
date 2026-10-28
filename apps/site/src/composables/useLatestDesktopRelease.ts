// The Downloads island's state + API call (rule: components never call an API —
// composables do, behind an injectable client seam). One `Async` union: `success`
// with `null` is the backend's `204` ("nothing to show"); it and `error` render the
// same fallback (change: add-music-download-page, D7).
import { computed, ref } from "vue";
import { type Async, idle, run } from "@cymbra/web-auth";
import { config } from "../lib/config";
import {
  createDesktopReleaseClient,
  type DesktopRelease,
  type DesktopReleaseClient,
  linuxCard,
} from "../lib/desktop-release";

let client: DesktopReleaseClient | null = null;

function useClient(): DesktopReleaseClient {
  client ??= createDesktopReleaseClient(config.apiUrl);
  return client;
}

/** Test seam: inject a fake client. */
export function setDesktopReleaseClientForTest(fake: DesktopReleaseClient | null): void {
  client = fake;
}

export function useLatestDesktopRelease() {
  const release = ref<Async<DesktopRelease | null>>(idle);
  const linux = computed(() => linuxCard(release.value));

  async function load(): Promise<void> {
    await run(
      release,
      () => useClient().latest(),
      (e) => {
        // Never shown: the card falls back to the releases page. Kept for the console.
        console.warn("desktop release view unavailable", e);
        return "unavailable";
      },
    );
  }

  return { release, linux, load };
}
