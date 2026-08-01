import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { setClientsForTest } from "@/lib/api";
import { currentLocale, setLocale } from "@/i18n";
import { useAuthStore } from "@/stores/auth";
import { useLocaleStore } from "@/stores/locale";
import { makeFakeClients, makeJwt } from "./fakes";

// The console is a second client of the shared account language (change:
// sync-account-language-preference): `choose` pushes the user's selection to the
// account when signed in, and `reconcile` folds the account's stored language back
// into the UI after sign-in.
describe("locale store — account language sync", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
    setLocale("en"); // reset the shared i18n singleton between tests
  });

  function signIn() {
    useAuthStore().setToken(makeJwt({ roles: ["moderator"], sub: "u1" }));
  }

  it("choose pushes the selection to the account when signed in", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    signIn();

    await useLocaleStore().choose("fr");

    expect(currentLocale()).toBe("fr"); // UI switched
    expect(state.setLocaleCalls).toEqual(["fr"]); // recorded on the account
  });

  it("choose stays local and pushes nothing when signed out", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    // no sign-in

    await useLocaleStore().choose("fr");

    expect(currentLocale()).toBe("fr"); // UI switched locally
    expect(state.setLocaleCalls).toEqual([]); // but never pushed
  });

  it("reconcile applies a supported server locale over the local choice", async () => {
    const { clients, state } = makeFakeClients({ accountLocale: "fr" });
    setClientsForTest(clients);
    signIn();

    await useLocaleStore().reconcile();

    expect(currentLocale()).toBe("fr"); // server wins
    expect(state.setLocaleCalls).toEqual([]); // no echo push
  });

  it("reconcile ignores an undisplayable server locale (UI + stored value intact)", async () => {
    // `es` is not one of the console's supported locales (en/fr).
    const { clients, state } = makeFakeClients({ accountLocale: "es" });
    setClientsForTest(clients);
    signIn();
    setLocale("en");

    await useLocaleStore().reconcile();

    expect(currentLocale()).toBe("en"); // UI unchanged
    expect(state.setLocaleCalls).toEqual([]); // stored value untouched
    expect(state.accountLocale).toBe("es");
  });

  it("reconcile adopts the local choice and pushes it up when the account is unset", async () => {
    const { clients, state } = makeFakeClients(); // accountLocale undefined
    setClientsForTest(clients);
    signIn();
    setLocale("fr"); // the user's current local choice

    await useLocaleStore().reconcile();

    expect(currentLocale()).toBe("fr"); // kept
    expect(state.setLocaleCalls).toEqual(["fr"]); // adopted by the account
  });
});
