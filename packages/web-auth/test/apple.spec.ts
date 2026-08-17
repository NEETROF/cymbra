import { afterEach, describe, expect, it, vi } from "vitest";

// Apple's JS SDK is external browser glue; `loadAppleId` short-circuits when
// `window.AppleID` exists, so a preset stub lets us test init + the popup
// credential-forwarding without a network script load. A separate test covers the
// actual <script> append/load path.

function installApple(over: { signIn?: () => Promise<unknown> } = {}) {
  const init = vi.fn();
  const signIn = vi.fn(over.signIn);
  vi.stubGlobal("AppleID", { auth: { init, signIn } });
  return { init, signIn };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  document.querySelectorAll("script").forEach((s) => s.remove());
});

describe("useAppleSignIn", () => {
  it("loads + initialises the SDK with the Services ID and popup flow", async () => {
    const { init } = installApple();
    const { useAppleSignIn } = await import("../src/apple");
    const { status, load } = useAppleSignIn("com.cymbra.bo.web", "https://bo.cymbra.app", vi.fn());

    await load("fr");

    expect(status.value.status).toBe("success");
    expect(init).toHaveBeenCalledWith(
      expect.objectContaining({
        clientId: "com.cymbra.bo.web",
        redirectURI: "https://bo.cymbra.app",
        usePopup: true,
      }),
    );
  });

  it("signIn is a no-op before the SDK is loaded", async () => {
    const { signIn } = installApple();
    const onCredential = vi.fn();
    const { useAppleSignIn } = await import("../src/apple");

    await useAppleSignIn("c", "r", onCredential).signIn();

    expect(signIn).not.toHaveBeenCalled();
    expect(onCredential).not.toHaveBeenCalled();
  });

  it("forwards the Apple id_token to onCredential", async () => {
    installApple({ signIn: () => Promise.resolve({ authorization: { id_token: "apple-tok" } }) });
    const onCredential = vi.fn();
    const { useAppleSignIn } = await import("../src/apple");
    const apple = useAppleSignIn("c", "r", onCredential);

    await apple.load("en");
    await apple.signIn();

    expect(onCredential).toHaveBeenCalledWith("apple-tok");
  });

  it("ignores an Apple response with no id_token", async () => {
    installApple({ signIn: () => Promise.resolve({ authorization: {} }) });
    const onCredential = vi.fn();
    const { useAppleSignIn } = await import("../src/apple");
    const apple = useAppleSignIn("c", "r", onCredential);

    await apple.load("en");
    await apple.signIn();

    expect(onCredential).not.toHaveBeenCalled();
  });

  it("captures an init failure in the status union", async () => {
    const { init } = installApple();
    init.mockImplementation(() => {
      throw new Error("bad services id");
    });
    const { useAppleSignIn } = await import("../src/apple");
    const { status, load } = useAppleSignIn("c", "r", vi.fn());

    await load("en");

    expect(status.value).toMatchObject({ status: "error" });
  });

  it("loads the Apple <script> when the SDK is absent, then initialises", async () => {
    const { useAppleSignIn } = await import("../src/apple");
    const { status, load } = useAppleSignIn("c", "r", vi.fn());

    const pending = load("en");
    const script = document.querySelector<HTMLScriptElement>('script[src*="appleid.auth.js"]');
    expect(script).not.toBeNull();

    const { init } = installApple();
    script!.dispatchEvent(new Event("load"));
    await pending;

    expect(status.value.status).toBe("success");
    expect(init).toHaveBeenCalled();
  });
});
