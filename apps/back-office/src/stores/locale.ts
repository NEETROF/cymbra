import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { currentLocale, type Locale, setLocale, SUPPORTED_LOCALES } from "@/i18n";
import { useAuthStore } from "@/stores/auth";

/** Narrow an arbitrary tag to a locale this console can display. */
function isSupported(tag: string | undefined): tag is Locale {
  return !!tag && (SUPPORTED_LOCALES as readonly string[]).includes(tag);
}

// Account language sync (change: sync-account-language-preference). The console is a
// second client of the shared account language: it pushes the user's selection to the
// account and, after sign-in, reconciles the account's stored language into the UI.
// The i18n `setLocale` stays a pure UI mutation — only this store talks to the API. The
// last push/reconcile outcome is one `Async` union so a denied/offline call lands in
// state rather than throwing.
export const useLocaleStore = defineStore("locale", () => {
  const sync = ref<Async<void>>(idle);

  /** Apply [locale] to the UI (local + localStorage) and, when signed in, record it
   * on the account. A signed-out change stays local (there is no account to write). */
  async function choose(locale: Locale) {
    setLocale(locale);
    if (!useAuthStore().isAuthenticated) return;
    await run(sync, async () => {
      await api().user.setLocale({ locale });
    });
  }

  /** Reconcile the account's stored language into the UI after sign-in (design D4):
   * - a supported server locale wins → apply it;
   * - an unset server locale → keep the local choice and push it up;
   * - an undisplayable server locale → leave the UI and the stored value untouched. */
  async function reconcile() {
    await run(sync, async () => {
      const account = await api().user.getAccount({});
      const server = account.locale ?? "";
      if (server === "") {
        await api().user.setLocale({ locale: currentLocale() });
      } else if (isSupported(server)) {
        setLocale(server);
      }
    });
  }

  return { sync, choose, reconcile };
});
