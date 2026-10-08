import { existsSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { AccountFlow } from "@/account/flow.ts";
import { accountLocale, CYMBRA_LANGUAGES, deleteAccountUrl } from "@/account/locale.ts";
import type { AccountMessage, AccountReply } from "@/account/messages.ts";
import { INTERFACE_LANGUAGES, type InterfaceLanguage } from "@/i18n/index.ts";

// The account's e-mails follow the interface language (localise-lingua-account-onboarding D2, M12):
// the locale the extension sends Cymbra ID on the four requests that carry one is a bare primary
// subtag — the interface language, or the browser's language when Cymbra speaks it and the extension
// does not — and the deletion link is chosen by the interface language alone. The flow is built as
// account.ts builds it: `locale: accountLocale(language, navigator.language)`, `language`.

const FRENCH_PAGE = "https://cymbra.app/suppression-compte/";
const ENGLISH_PAGE = "https://cymbra.app/en/delete-account/";

/** Music's locales, from its ARB files alone: `apps/music/lib/l10n/app_<code>.arb`. */
const MUSIC_L10N = join(dirname(fileURLToPath(import.meta.url)), "../../music/lib/l10n");

/** The locale-carrying messages a page sends, as account.ts wires it for a reader and a browser. */
async function sentLocales(language: InterfaceLanguage, browserLanguage: string) {
  const sent: AccountMessage[] = [];
  const replies: Partial<Record<AccountMessage["type"], AccountReply>> = {
    "account:state": { ok: true, state: { signedIn: false } },
    "account:providers": { ok: true, providers: { google: false, apple: false } },
  };
  const flow = new AccountFlow({
    send: async (message) => {
      sent.push(message);
      return replies[message.type] ?? { ok: true };
    },
    pending: { get: async () => null, set: async () => {} },
    pendingPassword: { get: async () => null, set: async () => {} },
    locale: accountLocale(language, browserLanguage),
    language,
    clearPersistedError: async () => {},
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

describe("accountLocale", () => {
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

describe("deleteAccountUrl", () => {
  it("opens the French page for French and the English one otherwise, until the site has a Spanish one", () => {
    expect(deleteAccountUrl("fr")).toBe(FRENCH_PAGE);
    expect(deleteAccountUrl("en")).toBe(ENGLISH_PAGE);
    expect(deleteAccountUrl("es")).toBe(ENGLISH_PAGE);
  });
});

describe("The account's e-mails follow the interface language", () => {
  it("A French reader with a French browser: `fr` (not `fr-FR`) on each of the four requests, the French page", async () => {
    const got = await sentLocales("fr", "fr-FR");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["fr", "fr", "fr", "fr"]);
    expect(got.deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("A Spanish-native reader: `es`, and the English page until the site has a Spanish one", async () => {
    const got = await sentLocales("es", "es-ES");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["es", "es", "es", "es"]);
    expect(got.deleteAccountUrl).toBe(ENGLISH_PAGE);
  });

  it("An Italian browser with a French interface: `it`, so Music's Italian e-mails stay Italian; the French page", async () => {
    const got = await sentLocales("fr", "it-IT");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["it", "it", "it", "it"]);
    expect(got.deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("A browser in a language Cymbra does not speak: an English interface on a German browser sends `en`", async () => {
    const got = await sentLocales("en", "de-DE");
    expect(got.types).toEqual(FOUR_REQUESTS);
    expect(got.locales).toEqual(["en", "en", "en", "en"]);
    expect(got.deleteAccountUrl).toBe(ENGLISH_PAGE);
  });
});

describe("Cymbra account deletion is reachable from Lingua (lingua-privacy, MODIFIED)", () => {
  it("A French-speaking reader: a French interface in an English browser opens the French page", async () => {
    const got = await sentLocales("fr", "en-GB");
    expect(got.locales).toEqual(["fr", "fr", "fr", "fr"]);
    expect(got.deleteAccountUrl).toBe(FRENCH_PAGE);
  });

  it("Another language: an English interface in a French browser opens the English page", async () => {
    const got = await sentLocales("en", "fr-FR");
    expect(got.deleteAccountUrl).toBe(ENGLISH_PAGE);
  });
});

describe("the three lingua-account requirements that carry the account locale", () => {
  it("New reader signs up: SignUpLocal gets the email, the password and the account locale, then the code step", async () => {
    const sent: AccountMessage[] = [];
    const flow = new AccountFlow({
      send: async (message) => {
        sent.push(message);
        return { ok: true };
      },
      pending: { get: async () => null, set: async () => {} },
      pendingPassword: { get: async () => null, set: async () => {} },
      locale: accountLocale("fr", "en-GB"),
      language: "fr",
      clearPersistedError: async () => {},
    });
    const s = await flow.signUp("new@example.com", "a long passphrase");
    expect(sent).toEqual([
      { type: "account:signUp", email: "new@example.com", password: "a long passphrase", locale: "fr" },
    ]);
    expect(s.view).toBe("verify");
    expect(s.email).toBe("new@example.com");
  });

  it("Email verification by code and password reset: the resend and the request carry the account locale", async () => {
    const got = await sentLocales("en", "it-IT");
    const at = (type: string): string => got.locales[got.types.indexOf(type)];
    expect(at("account:resendVerification")).toBe("it");
    expect(at("account:requestPasswordReset")).toBe("it");
  });
});
