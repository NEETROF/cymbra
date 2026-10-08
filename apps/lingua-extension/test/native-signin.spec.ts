import { describe, expect, expectTypeOf, it, type Mock, vi } from "vitest";
import type { InterfaceLanguage } from "@/i18n/language.ts";
import {
  hostAppSignInUrl,
  interfaceLanguageTeller,
  type NativeSend,
  nativeProviders,
  parseHandedIdToken,
  takeHandedIdToken,
  tellInterfaceLanguage,
} from "@/state/native-signin.ts";
import type { Provider } from "@/state/oidc.ts";

describe("nativeProviders", () => {
  it("offers Apple always and Google as the host app reports it", async () => {
    const send = vi.fn(async () => ({ apple: true, google: true }));
    expect(await nativeProviders(send)).toEqual({ apple: true, google: true });
    expect(send).toHaveBeenCalledWith({ type: "auth.providers" });
    expect(await nativeProviders(async () => ({ apple: true, google: false }))).toEqual({ apple: true, google: false });
  });

  it("offers Apple only when the handler is missing, failing or unclear", async () => {
    for (const send of [
      async () => {
        throw new Error("no native handler");
      },
      async () => null,
      async () => ({ google: "yes" }),
    ]) {
      expect(await nativeProviders(send)).toEqual({ apple: true, google: false });
    }
  });
});

describe("hostAppSignInUrl", () => {
  it("opens the host app on the requested provider, in the interface language the sheet should speak", () => {
    expect(hostAppSignInUrl("apple", "fr")).toBe("cymbra-lingua://signin?provider=apple&lang=fr");
    expect(hostAppSignInUrl("google", "es")).toBe("cymbra-lingua://signin?provider=google&lang=es");
    expect(hostAppSignInUrl("apple", "en")).toBe("cymbra-lingua://signin?provider=apple&lang=en");
  });

  it("requires the language, so every caller names one", () => {
    expectTypeOf(hostAppSignInUrl).parameters.toEqualTypeOf<[Provider, InterfaceLanguage]>();
  });
});

describe("tellInterfaceLanguage", () => {
  it("sends the host app the interface language", async () => {
    const send = vi.fn(async () => ({ language: "es" }));
    await expect(tellInterfaceLanguage(send, "es")).resolves.toBe(true);
    expect(send).toHaveBeenCalledWith({ type: "interface.language", language: "es" });
  });

  it("treats a missing or failing handler as nothing to act on", async () => {
    await expect(
      tellInterfaceLanguage(async () => {
        throw new Error("no native handler");
      }, "fr"),
    ).resolves.toBe(false);
    await expect(tellInterfaceLanguage(async () => null, "en")).resolves.toBe(true);
  });
});

describe("interfaceLanguageTeller", () => {
  const told = (send: Mock<NativeSend>): string[] =>
    send.mock.calls.map(([message]) => (message.type === "interface.language" ? message.language : message.type));

  it("tells only a language the key actually holds", async () => {
    const send = vi.fn<NativeSend>(async () => ({}));
    const tell = interfaceLanguageTeller(send);
    // A key not yet written, removed, or holding anything else: nothing, not a defaulted French.
    for (const value of [undefined, null, "", "de", "EN", 42, { language: "es" }]) await tell(value);
    expect(send).not.toHaveBeenCalled();
    await tell("es");
    expect(send).toHaveBeenCalledWith({ type: "interface.language", language: "es" });
  });

  it("tells a language once, and again only once it changes", async () => {
    const send = vi.fn<NativeSend>(async () => ({}));
    const tell = interfaceLanguageTeller(send);
    await tell("en");
    await tell("en");
    await tell("es");
    await tell("es");
    await tell("en");
    expect(told(send)).toEqual(["en", "es", "en"]);
  });

  it("tells a language once when the start and a change race with it", async () => {
    const send = vi.fn<NativeSend>(async () => ({}));
    const tell = interfaceLanguageTeller(send);
    await Promise.all([tell("es"), tell("es")]);
    expect(told(send)).toEqual(["es"]);
  });

  it("tells a language again after a send that failed", async () => {
    const send = vi.fn<NativeSend>().mockRejectedValueOnce(new Error("no native handler")).mockResolvedValue({});
    const tell = interfaceLanguageTeller(send);
    await tell("es");
    await tell("es");
    await tell("es");
    expect(told(send)).toEqual(["es", "es"]);
  });
});

describe("parseHandedIdToken", () => {
  it("accepts a known provider with a non-empty id_token", () => {
    expect(parseHandedIdToken({ provider: "apple", idToken: "tok" })).toEqual({ provider: "apple", idToken: "tok" });
  });

  it("reads nothing pending and malformed replies as no token", () => {
    for (const reply of [
      null,
      undefined,
      "tok",
      {},
      { provider: "apple" },
      { provider: "apple", idToken: "" },
      { provider: "github", idToken: "tok" },
      { provider: "google", idToken: 42 },
    ]) {
      expect(parseHandedIdToken(reply)).toBeNull();
    }
  });
});

describe("takeHandedIdToken", () => {
  it("asks the native handler for the pending token", async () => {
    const send = vi.fn(async () => ({ provider: "google", idToken: "tok" }));
    expect(await takeHandedIdToken(send)).toEqual({ provider: "google", idToken: "tok" });
    expect(send).toHaveBeenCalledWith({ type: "auth.takeIdToken" });
  });

  it("treats a missing or failing handler as nothing pending", async () => {
    expect(
      await takeHandedIdToken(async () => {
        throw new Error("no native handler");
      }),
    ).toBeNull();
  });
});
