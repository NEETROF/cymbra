import { existsSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AccountFlow } from "@/account/flow.ts";
import {
  type AccountLanguage,
  accountLanguage,
  accountLocale,
  chosenLanguage,
  CYMBRA_LANGUAGES,
  deleteAccountUrl,
} from "@/account/locale.ts";
import type { AccountMessage, AccountReply } from "@/account/messages.ts";
import { INTERFACE_LANGUAGES, type InterfaceLanguage } from "@/i18n/index.ts";
import {
  changeNativeLanguage,
  NATIVE_CHOSEN_KEY,
  NATIVE_LANGUAGE_MESSAGE,
  type NativeLanguageHost,
  onExtensionInstalled,
} from "@/state/native-language.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import { LAST_NATIVE_KEY, ownerArea } from "@/state/store.ts";
import { pageArea, refusingArea } from "./helpers.ts";

// The account's e-mails follow the interface language once the reader has chosen it
// (localise-lingua-account-onboarding D2, M12, and the owner's choice of D2's second way on
// 2026-10-10, with its warning about several devices). The account's locale is shared — every device,
// every Cymbra app — and Cymbra ID records a non-empty one over the one it has. So, until the reader
// has chosen on this device: the browser's whole tag where nothing is written over (sign-up, setting
// a password), no locale where the account's would be (resending the code, requesting a reset), and
// the deletion page by the browser's tag, as before. Once they have — both of
// add-lingua-native-language-choice's records — a bare primary subtag on all four, the interface
// language or the browser's when Cymbra speaks it and the extension does not, and the deletion link
// by the interface language alone. The flow is built as page.ts builds it.

const FRENCH_PAGE = "https://cymbra.app/suppression-compte/";
const ENGLISH_PAGE = "https://cymbra.app/en/delete-account/";

/** Music's locales, from its ARB files alone: `apps/music/lib/l10n/app_<code>.arb`. */
const MUSIC_L10N = join(dirname(fileURLToPath(import.meta.url)), "../../music/lib/l10n");

/** es-en shipping beside today's pairs (change 34): the first time a reader can choose. */
const MIXED = ["en-fr", "es-fr", "es-en"];

afterEach(() => {
  vi.restoreAllMocks();
});

type Send = (message: AccountMessage) => Promise<AccountReply | null>;

/** A flow as page.ts builds it from what `accountLanguage` said; `pending`, an address awaiting its code. */
function flowFor(
  said: AccountLanguage,
  language: InterfaceLanguage,
  send: Send,
  pending: string | null = null,
): AccountFlow {
  return new AccountFlow({
    send,
    pending: { get: async () => pending, set: async () => {} },
    pendingPassword: { get: async () => null, set: async () => {} },
    locale: said.locale,
    keepAccountLocale: said.keepAccountLocale,
    deletionLanguage: said.deletion,
    language,
    clearPersistedError: async () => {},
  });
}

/**
 * The four requests a page sends — sign-up, resend the code, request a reset, set a password, in that
 * order — and the deletion page it links to; `chosen` is what `chosenLanguage` read, null while the
 * reader has not chosen.
 */
async function sentLocales(language: InterfaceLanguage, browserLanguage: string, chosen: string | null) {
  const sent: AccountMessage[] = [];
  const replies: Partial<Record<AccountMessage["type"], AccountReply>> = {
    "account:state": { ok: true, state: { signedIn: false } },
    "account:providers": { ok: true, providers: { google: false, apple: false } },
  };
  const flow = flowFor(accountLanguage(language, browserLanguage, chosen), language, async (message) => {
    sent.push(message);
    return replies[message.type] ?? { ok: true };
  });
  await flow.init("#signup");
  await flow.signUp("new@example.com", "a long passphrase");
  await flow.resend();
  await flow.requestReset("new@example.com");
  // « Définir un mot de passe » is offered signed in, on the connected accounts.
  replies["account:state"] = { ok: true, state: { signedIn: true } };
  replies["account:profile"] = { ok: true, handle: "ada" };
  replies["account:identities"] = { ok: true, identities: [], linkable: { google: false, apple: false } };
  await flow.init("#connected");
  flow.showPasswordForm();
  await flow.setPassword("ada@example.com", "a long passphrase");
  const carrying = sent.filter((m): m is Extract<AccountMessage, { locale: string }> => "locale" in m);
  return {
    types: carrying.map((m): string => m.type),
    locales: carrying.map((m) => m.locale),
    deleteAccountUrl: flow.view().deleteAccountUrl,
  };
}

const FOUR_REQUESTS = [
  "account:signUp",
  "account:resendVerification",
  "account:requestPasswordReset",
  "account:setPassword",
];

/**
 * Cymbra ID as it answers the four requests (backend/auth/src/module.rs, user-locale-preference): a
 * non-empty locale is recorded on the account — at sign-up, on a resend and on a reset request, last
 * writer wins — and an empty one leaves the account's as it was; the e-mail is written in the
 * request's locale, else the account's (resend, reset), else English; setting a password records
 * nothing and writes in the request's locale, else English. A language is read by its primary
 * subtag. Cymbra Music adopts the account's locale only when it is one of its codes, whole
 * (`AppLanguage.fromCode`: `fr` yes, `fr-FR` no).
 */
function cymbraId() {
  const account: { locale: string | null } = { locale: null };
  const mails: string[] = [];
  const parse = (tag: string | null): string => {
    const primary = (tag ?? "").trim().toLowerCase().split(/[-_]/)[0];
    return ["fr", "es", "it"].includes(primary) ? primary : "en";
  };
  const record = (locale: string): void => {
    if (locale !== "") account.locale = locale;
  };
  const send: Send = async (message) => {
    switch (message.type) {
      case "account:signUp":
        record(message.locale);
        mails.push(parse(message.locale));
        break;
      case "account:resendVerification":
      case "account:requestPasswordReset":
        record(message.locale);
        mails.push(parse(message.locale !== "" ? message.locale : account.locale));
        break;
      case "account:setPassword":
        mails.push(parse(message.locale));
        break;
      case "account:state":
        return { ok: true, state: { signedIn: false } };
      case "account:providers":
        return { ok: true, providers: { google: false, apple: false } };
      default:
        break;
    }
    return { ok: true };
  };
  const music = (): string | null =>
    account.locale !== null && CYMBRA_LANGUAGES.includes(account.locale) ? account.locale : null;
  return { account, mails, send, music };
}

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

/** A backup as the engine writes the profile: under its enums' names (state/profile.ts). */
function backupOf(native: "French" | "English" | "Spanish", studied: ("English" | "Spanish")[]): string {
  return JSON.stringify({ schema_version: 2, profile: { native_language: native, studied_languages: studied } });
}

/**
 * A device as add-lingua-native-language-choice leaves it — the background's store behind its owner,
 * chrome.storage.local beside it — with the choice offered (es-en shipping): what its own code writes
 * when the reader answers, a preset applies or an update lands is what the account page reads.
 */
function device(store: Record<string, unknown> = {}) {
  const NAMES = { fr: "French", en: "English", es: "Spanish" } as const;
  const preferences = fakeArea();
  const owned = ownerArea(fakeArea(store), () => {}, preferences, MIXED);
  const host = {
    store: owned,
    preferences,
    ensureBackup: async () => {
      await owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) } });
    },
    reprofile: async (backup: string, native: "fr" | "en" | "es", studied: ("en" | "es")[]) =>
      JSON.stringify({
        ...(JSON.parse(backup) as object),
        profile: { native_language: NAMES[native], studied_languages: studied.map((language) => NAMES[language]) },
      }),
    exclusive: <T>(task: () => Promise<T>): Promise<T> => task(),
    rehydrate: () => {},
    pairs: MIXED,
  } satisfies NativeLanguageHost;
  const choose = (native: string, preset = false) =>
    changeNativeLanguage(
      host,
      preset ? { type: NATIVE_LANGUAGE_MESSAGE, native, preset } : { type: NATIVE_LANGUAGE_MESSAGE, native },
    );
  return { preferences, owned, choose, chosen: () => chosenLanguage(preferences, owned) };
}

describe("the languages Cymbra speaks", () => {
  // The AMO source archive carries the extension without apps/music: there is nothing to hold the
  // list against there, and the check runs in the repository (and in CI, whose `ext` filter watches
  // the ARB files).
  it.skipIf(!existsSync(MUSIC_L10N))(
    "are Music's locales, its app_<code>.arb files, so a language added to Music is noticed here",
    () => {
      const music = readdirSync(MUSIC_L10N)
        .map((name) => /^app_(.+)\.arb$/.exec(name)?.[1])
        .filter((code): code is string => code != null);
      // Every file is a bare primary subtag: an `app_pt_BR.arb` would be a locale Music matches whole,
      // which accountLocale's bare subtag would not reach.
      for (const code of music) expect(code, `app_${code}.arb`).toMatch(/^[a-z]{2,3}$/);
      expect([...CYMBRA_LANGUAGES].sort()).toEqual(music.sort());
    },
  );

  it("include every language the extension speaks: its e-mails exist in each", () => {
    for (const language of INTERFACE_LANGUAGES) expect(CYMBRA_LANGUAGES).toContain(language);
  });
});

describe("accountLocale, once the reader has chosen", () => {
  it("is the interface language, a bare primary subtag, never the browser's whole tag", () => {
    expect(accountLocale("fr", "fr-FR")).toBe("fr");
    expect(accountLocale("fr", "en-GB")).toBe("fr");
    expect(accountLocale("en", "fr-FR")).toBe("en");
    expect(accountLocale("es", "es-419")).toBe("es");
    expect(accountLocale("es", "en-US")).toBe("es");
  });

  it("is the browser's language when Cymbra speaks it and the extension does not", () => {
    expect(accountLocale("fr", "it-IT")).toBe("it");
    expect(accountLocale("en", "it")).toBe("it");
    expect(accountLocale("es", "IT_ch")).toBe("it");
  });

  it("is the interface language when the browser's language is one Cymbra does not speak", () => {
    expect(accountLocale("en", "de-DE")).toBe("en");
    expect(accountLocale("fr", "pt-BR")).toBe("fr");
    expect(accountLocale("es", "")).toBe("es");
  });
});

describe("accountLanguage: until the reader has chosen the language the page is in, nothing written over", () => {
  it("not chosen: the browser's whole tag as it gives it, the account's own kept, the deletion page by the tag", () => {
    expect(accountLanguage("fr", "en-GB", null)).toEqual({
      locale: "en-GB",
      keepAccountLocale: true,
      deletion: "en-GB",
    });
    expect(accountLanguage("fr", "fr-FR", null)).toEqual({
      locale: "fr-FR",
      keepAccountLocale: true,
      deletion: "fr-FR",
    });
    expect(accountLanguage("fr", "it-IT", null)).toEqual({
      locale: "it-IT",
      keepAccountLocale: true,
      deletion: "it-IT",
    });
    expect(accountLanguage("en", "de-DE", null)).toEqual({
      locale: "de-DE",
      keepAccountLocale: true,
      deletion: "de-DE",
    });
  });

  it('not chosen, a browser that gives no language: `fr`, as `navigator.language || "fr"` always sent', () => {
    expect(accountLanguage("fr", "", null)).toEqual({ locale: "fr", keepAccountLocale: true, deletion: "fr" });
  });

  it("a choice of another language than the page is in is no choice of it", () => {
    expect(accountLanguage("fr", "en-GB", "en")).toEqual({
      locale: "en-GB",
      keepAccountLocale: true,
      deletion: "en-GB",
    });
  });

  it("chosen: the account locale, written over the account's, and the deletion page by the interface language", () => {
    expect(accountLanguage("fr", "en-GB", "fr")).toEqual({ locale: "fr", keepAccountLocale: false, deletion: "fr" });
    expect(accountLanguage("fr", "it-IT", "fr")).toEqual({ locale: "it", keepAccountLocale: false, deletion: "fr" });
    expect(accountLanguage("en", "", "en")).toEqual({ locale: "en", keepAccountLocale: false, deletion: "en" });
  });
});

describe("deleteAccountUrl, the rule it always had", () => {
  it("opens the French page for a tag that starts with `fr`, the English one otherwise", () => {
    expect(deleteAccountUrl("fr")).toBe(FRENCH_PAGE);
    expect(deleteAccountUrl("fr-CA")).toBe(FRENCH_PAGE);
    expect(deleteAccountUrl("FR-fr")).toBe(FRENCH_PAGE);
    expect(deleteAccountUrl("en")).toBe(ENGLISH_PAGE);
    expect(deleteAccountUrl("en-GB")).toBe(ENGLISH_PAGE);
    expect(deleteAccountUrl("it-IT")).toBe(ENGLISH_PAGE);
  });

  it("opens the English page for Spanish until the site has a Spanish one", () => {
    expect(deleteAccountUrl("es")).toBe(ENGLISH_PAGE);
    expect(deleteAccountUrl("es-ES")).toBe(ENGLISH_PAGE);
  });
});

describe("chosenLanguage reads what add-lingua-native-language-choice records on this device", () => {
  it("Every reader today: nothing recorded, no choice — and the store is not asked", async () => {
    const store = { get: vi.fn(async () => ({})) };
    expect(await chosenLanguage(pageArea(), store)).toBeNull();
    expect(store.get).not.toHaveBeenCalled();
  });

  it("An update marks an installed extension as chosen without asking it: no choice", async () => {
    const { preferences, choose, chosen } = device({
      [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) },
    });
    await onExtensionInstalled("update", preferences);
    expect(preferences.store[NATIVE_CHOSEN_KEY]).toBe(true);
    expect(await chosen()).toBeNull();
    // The preset that may follow is then left alone: still no choice.
    expect(await choose("en", true)).toEqual({ ok: true, changed: false });
    expect(await chosen()).toBeNull();
  });

  it("A preset that finds the reader's data marks the choice as made and records nothing: no choice", async () => {
    const backup = JSON.stringify({
      ...(JSON.parse(backupOf("French", ["English"])) as object),
      knowledge: { statuses: { English: { house: "Known" } } },
    });
    const { preferences, choose, chosen } = device({ [ROOT_KEY]: { v: 2, backup } });
    expect(await choose("en", true)).toEqual({ ok: true, changed: false });
    expect(preferences.store[NATIVE_CHOSEN_KEY]).toBe(true);
    expect(await chosen()).toBeNull();
  });

  it("A new install's preset records its language before the reader answers: no choice yet", async () => {
    const { owned, choose, chosen } = device();
    expect(await choose("en", true)).toEqual({ ok: true, changed: true });
    expect((await owned.get(LAST_NATIVE_KEY))[LAST_NATIVE_KEY]).toBe("en");
    expect(await chosen()).toBeNull();
  });

  it("The reader's answer — the preset confirmed, or another language — is the choice", async () => {
    const confirmed = device();
    await confirmed.choose("en", true);
    await confirmed.choose("en");
    expect(await confirmed.chosen()).toBe("en");

    const other = device();
    await other.choose("en", true);
    await other.choose("fr");
    expect(await other.chosen()).toBe("fr");
  });

  it("An installed extension whose reader chooses another language in Réglages has chosen it", async () => {
    const { preferences, choose, chosen } = device({
      [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) },
    });
    await onExtensionInstalled("update", preferences);
    await choose("en");
    expect(await chosen()).toBe("en");
  });

  it("an answer that keeps the language the extension already had records nothing: no choice", async () => {
    const { choose, chosen, preferences } = device({
      [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) },
    });
    expect(await choose("fr")).toEqual({ ok: true, changed: false });
    expect(preferences.store[NATIVE_CHOSEN_KEY]).toBe(true);
    expect(await chosen()).toBeNull();
  });

  it("a read that fails is no choice, and is said", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(await chosenLanguage(refusingArea(), pageArea({ [LAST_NATIVE_KEY]: "en" }))).toBeNull();
    expect(await chosenLanguage(pageArea({ [NATIVE_CHOSEN_KEY]: true }), refusingArea())).toBeNull();
    expect(warn).toHaveBeenCalledTimes(2);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("a record that names no language is no choice", async () => {
    expect(
      await chosenLanguage(pageArea({ [NATIVE_CHOSEN_KEY]: true }), pageArea({ [LAST_NATIVE_KEY]: 7 })),
    ).toBeNull();
  });
});

describe("The account's e-mails follow the interface language", () => {
  it("A reader who has not chosen: the browser's whole tag at sign-up and on a password, no locale on a resend or a reset", async () => {
    const english = await sentLocales("fr", "en-GB", null);
    expect(english.types).toEqual(FOUR_REQUESTS);
    expect(english.locales).toEqual(["en-GB", "", "", "en-GB"]);
    expect(english.deleteAccountUrl).toBe(ENGLISH_PAGE);

    const french = await sentLocales("fr", "fr-FR", null);
    expect(french.locales).toEqual(["fr-FR", "", "", "fr-FR"]);
    expect(french.deleteAccountUrl).toBe(FRENCH_PAGE);

    const italian = await sentLocales("fr", "it-IT", null);
    expect(italian.locales).toEqual(["it-IT", "", "", "it-IT"]);
    expect(italian.deleteAccountUrl).toBe(ENGLISH_PAGE);
  });

  it("A reader who has not chosen, on one device: their e-mails as before, Cymbra Music's language untouched", async () => {
    // What this device always sent, the browser's tag on all four: the same e-mails, and an account
    // locale Music, matching whole codes alone, never adopted.
    const before = cymbraId();
    const today = flowFor({ locale: "en-GB", keepAccountLocale: false, deletion: "en-GB" }, "fr", before.send);
    const now = cymbraId();
    const notChosen = flowFor(accountLanguage("fr", "en-GB", null), "fr", now.send);
    for (const flow of [today, notChosen]) {
      await flow.signUp("new@example.com", "a long passphrase");
      await flow.resend();
      await flow.requestReset("new@example.com");
      await flow.setPassword("new@example.com", "a long passphrase");
    }
    expect(now.mails).toEqual(before.mails);
    expect(now.mails).toEqual(["en", "en", "en", "en"]);
    expect(now.account.locale).toBe("en-GB");
    expect(now.music()).toBeNull();
  });

  it("A browser that gives no language, not chosen: `fr` at sign-up and the French page, as before", async () => {
    const got = await sentLocales("fr", "", null);
    expect(got.locales).toEqual(["fr", "", "", "fr"]);
    expect(got.deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("A French reader with a French browser, French chosen: `fr` (not `fr-FR`) on each of the four requests, the French page", async () => {
    const got = await sentLocales("fr", "fr-FR", "fr");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["fr", "fr", "fr", "fr"]);
    expect(got.deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("A Spanish-native reader, Spanish chosen: `es`, and the English page until the site has a Spanish one", async () => {
    const got = await sentLocales("es", "es-ES", "es");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["es", "es", "es", "es"]);
    expect(got.deleteAccountUrl).toBe(ENGLISH_PAGE);
  });

  it("An Italian browser, French chosen: `it`, so Music's Italian e-mails stay Italian; the French page", async () => {
    const got = await sentLocales("fr", "it-IT", "fr");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["it", "it", "it", "it"]);
    expect(got.deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("A browser in a language Cymbra does not speak: English chosen on a German browser sends `en`", async () => {
    const got = await sentLocales("en", "de-DE", "en");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["en", "en", "en", "en"]);
    expect(got.deleteAccountUrl).toBe(ENGLISH_PAGE);
  });
});

describe("A choice made on another device is kept (the owner's warning of 2026-10-10)", () => {
  it("English chosen on laptop A; laptop B, a French browser where nothing is chosen, resends and resets: English stays", async () => {
    const id = cymbraId();
    // Laptop A: a French browser, English chosen there.
    const a = flowFor(accountLanguage("en", "fr-FR", "en"), "en", id.send);
    await a.signUp("ada@example.com", "a long passphrase");
    expect(id.account.locale).toBe("en");
    expect(id.music()).toBe("en");

    // Laptop B: a French browser, its interface French, nothing chosen on it. The reader forgot
    // their password, then, signed in, sets one.
    const b = flowFor(accountLanguage("fr", "fr-FR", null), "fr", id.send);
    await b.requestReset("ada@example.com");
    await b.setPassword("ada@example.com", "a long passphrase");

    expect(id.account.locale).toBe("en");
    expect(id.music()).toBe("en");
    // The reset in the account's English; setting a password records nothing, and writes in this
    // browser's language, as it always did.
    expect(id.mails).toEqual(["en", "en", "fr"]);
  });

  it("an account Cymbra Music gave its language keeps it, and Music does not flip, whatever B's browser", async () => {
    const id = cymbraId();
    id.account.locale = "es"; // created in Music, in « Español »
    // Laptop B, an American browser where nothing is chosen, on the code step of that account.
    const b = flowFor(accountLanguage("fr", "en-US", null), "fr", id.send, "ada@example.com");
    await b.init("#verify");
    await b.resend();
    await b.requestReset("ada@example.com");
    expect(id.account.locale).toBe("es");
    expect(id.music()).toBe("es");
    expect(id.mails).toEqual(["es", "es"]);
  });

  it("the device where the reader chose writes their choice — and only there", async () => {
    const id = cymbraId();
    id.account.locale = "en";
    const b = flowFor(accountLanguage("fr", "fr-FR", "fr"), "fr", id.send);
    await b.requestReset("ada@example.com");
    expect(id.account.locale).toBe("fr");
    expect(id.music()).toBe("fr");
  });
});

describe("Cymbra account deletion is reachable from Lingua (lingua-privacy, MODIFIED)", () => {
  it("A French-speaking reader: French chosen in an English browser opens the French page; a French browser, not chosen, too", async () => {
    expect((await sentLocales("fr", "en-GB", "fr")).deleteAccountUrl).toBe(FRENCH_PAGE);
    expect((await sentLocales("fr", "fr-FR", null)).deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("Another language: English chosen in a French browser, or an English browser that has not chosen, opens the English page", async () => {
    expect((await sentLocales("en", "fr-FR", "en")).deleteAccountUrl).toBe(ENGLISH_PAGE);
    expect((await sentLocales("fr", "en-GB", null)).deleteAccountUrl).toBe(ENGLISH_PAGE);
  });
});

describe("the three lingua-account requirements that carry the locale", () => {
  it("New reader signs up: SignUpLocal gets the email, the password and the locale, then the code step", async () => {
    for (const [chosen, locale] of [
      [null, "en-GB"],
      ["fr", "fr"],
    ] as const) {
      const sent: AccountMessage[] = [];
      const flow = flowFor(accountLanguage("fr", "en-GB", chosen), "fr", async (message) => {
        sent.push(message);
        return { ok: true };
      });
      const s = await flow.signUp("new@example.com", "a long passphrase");
      expect(sent).toEqual([
        { type: "account:signUp", email: "new@example.com", password: "a long passphrase", locale },
      ]);
      expect(s.view).toBe("verify");
      expect(s.email).toBe("new@example.com");
    }
  });

  it("Email verification by code and password reset: the resend and the request carry the locale once chosen, none before", async () => {
    const chosen = await sentLocales("en", "it-IT", "en");
    const at = (got: typeof chosen, type: string): string => got.locales[got.types.indexOf(type)];
    expect(at(chosen, "account:resendVerification")).toBe("it");
    expect(at(chosen, "account:requestPasswordReset")).toBe("it");
    const notYet = await sentLocales("fr", "en-US", null);
    expect(at(notYet, "account:resendVerification")).toBe("");
    expect(at(notYet, "account:requestPasswordReset")).toBe("");
  });
});
