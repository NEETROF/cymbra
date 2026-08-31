// The `/suppression-compte` (`/en/delete-account`) island's state + API calls
// (rule: components never call an API — composables do, behind the injectable
// `session.ts` seam).
//
// The route exists because Google Play requires a **web** path to account deletion
// for any app that offers sign-up; the in-app screen alone does not satisfy it.
import { computed, ref } from "vue";
import { type Async, idle, run } from "@cymbra/web-auth";
import { t, type Lang } from "../lib/i18n";
import { humanError } from "../lib/plan-view";
import { usePlans, useSession } from "../lib/session";
import type { AccountView } from "../lib/web-plans";

export function useDeleteAccount(lang: Lang) {
  const session = useSession(lang);
  const plans = usePlans();

  const booted = ref(false);
  /** Handle + sign-in methods, so the page names the account it is about to erase. */
  const account = ref<Async<AccountView>>(idle);
  /** The deletion itself; `success` switches the island to its final state. */
  const deletion = ref<Async<{ deleted: boolean }>>(idle);
  const typed = ref("");

  const signedIn = computed(() => session.state.value.kind === "signedIn");
  const deleting = computed(() => deletion.value.status === "loading");
  const deleted = computed(() => deletion.value.status === "success");

  /** What the user must type to arm the button: their handle, else a fixed word. */
  const confirmWord = computed(() =>
    account.value.status === "success" && account.value.data.handle
      ? account.value.data.handle
      : t(lang, "deleteConfirmWord"),
  );
  // Case-insensitive and trimmed: this gate is a deliberate pause, not a spelling test.
  const armed = computed(
    () => !deleting.value && typed.value.trim().toLocaleLowerCase() === confirmWord.value.toLocaleLowerCase(),
  );

  async function load(): Promise<void> {
    const token = session.accessToken.value;
    if (!token) return;
    await run(account, () => plans.account(token), (e) => humanError(lang, e));
  }

  async function boot(): Promise<void> {
    await session.boot();
    booted.value = true;
    await load();
  }

  /** Erase the account, then drop the local session so the page cannot act again. */
  async function confirmDelete(): Promise<void> {
    const token = session.accessToken.value;
    if (!token || !armed.value) return;
    const out = await run(deletion, () => plans.deleteAccount(token), (e) => humanError(lang, e));
    if (out.status === "success") {
      typed.value = "";
      account.value = idle;
      await session.signOut();
    }
  }

  async function signOut(): Promise<void> {
    await session.signOut();
    account.value = idle;
    deletion.value = idle;
    typed.value = "";
  }

  return {
    booted,
    account,
    deletion,
    typed,
    signedIn,
    deleting,
    deleted,
    confirmWord,
    armed,
    boot,
    load,
    confirmDelete,
    signOut,
  };
}
