import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";

// The signed-in administrator's own identity, for the sidebar. The console used to show
// a role chip and nothing else, so nothing on screen said WHICH account was signed in —
// and every scope-gated page (Jobs, Takedowns, Lingua) depends on that account's roles.
// The API call lives here, never in a component; the view matches on the `Async` union.

/** What the console knows about the signed-in account. */
export interface Me {
  userId: string;
  handle: string | null;
  displayName: string | null;
}

/** The name to show: the handle, else the display name, else a short id — never empty. */
export function meLabel(me: Me | null, fallbackUserId?: string): string {
  const id = me?.userId ?? fallbackUserId ?? "";
  return me?.handle || me?.displayName || (id ? id.slice(0, 8) : "");
}

export const useMeStore = defineStore("me", () => {
  const profile = ref<Async<Me>>(idle);

  /** Read the caller's own account (the identity comes from the token, not a parameter). */
  async function load() {
    await run(profile, async () => {
      const a = await api().user.getAccount({});
      return {
        userId: a.userId,
        handle: a.handle ?? null,
        displayName: a.displayName ?? null,
      } satisfies Me;
    });
  }

  /** The loaded account, or null while loading or after a failure. */
  const me = computed<Me | null>(() => (profile.value.status === "success" ? profile.value.data : null));

  /** Forget the identity on sign-out, so the next session never shows the previous one. */
  function clear() {
    profile.value = idle;
  }

  return { profile, me, load, clear };
});
