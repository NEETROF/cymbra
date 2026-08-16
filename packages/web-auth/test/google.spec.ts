import { afterEach, describe, expect, it, vi } from "vitest";

// The GSI SDK is external browser glue; we drive it through the seam the composable
// already has — `loadGsi` short-circuits when `window.google.accounts.id` exists, so a
// preset stub lets us test the credential-forwarding + status logic without a network
// script load. A separate test covers the actual <script> append/load path.

function installGsi(over: { renderButton?: () => void } = {}) {
  const initialize = vi.fn();
  const renderButton = vi.fn(over.renderButton);
  vi.stubGlobal("google", { accounts: { id: { initialize, renderButton } } });
  return { initialize, renderButton };
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.resetModules();
  document.querySelectorAll("script").forEach((s) => s.remove());
});

describe("useGoogleSignIn", () => {
  it("initialises GSI and renders the button into the slot", async () => {
    const { initialize, renderButton } = installGsi();
    const { useGoogleSignIn } = await import("../src/google");
    const parent = document.createElement("div");
    const { status, render } = useGoogleSignIn("client-123", vi.fn());

    await render(parent, { locale: "en" });

    expect(status.value.status).toBe("success");
    expect(initialize).toHaveBeenCalledWith(expect.objectContaining({ client_id: "client-123" }));
    expect(renderButton).toHaveBeenCalledWith(parent, expect.objectContaining({ type: "standard", locale: "en" }));
  });

  it("forwards a credential to onCredential", async () => {
    const { initialize } = installGsi();
    const onCredential = vi.fn();
    const { useGoogleSignIn } = await import("../src/google");
    await useGoogleSignIn("c", onCredential).render(document.createElement("div"));

    initialize.mock.calls[0][0].callback({ credential: "id-token-abc" });

    expect(onCredential).toHaveBeenCalledWith("id-token-abc");
  });

  it("ignores a callback with no credential", async () => {
    const { initialize } = installGsi();
    const onCredential = vi.fn();
    const { useGoogleSignIn } = await import("../src/google");
    await useGoogleSignIn("c", onCredential).render(document.createElement("div"));

    initialize.mock.calls[0][0].callback({});

    expect(onCredential).not.toHaveBeenCalled();
  });

  it("captures a render failure in the status union", async () => {
    installGsi({
      renderButton: () => {
        throw new Error("boom");
      },
    });
    const { useGoogleSignIn } = await import("../src/google");
    const { status, render } = useGoogleSignIn("c", vi.fn());

    await render(document.createElement("div"));

    expect(status.value).toMatchObject({ status: "error" });
  });

  it("loads the GSI <script> when the SDK is absent, then renders", async () => {
    const { useGoogleSignIn } = await import("../src/google");
    const { status, render } = useGoogleSignIn("c", vi.fn());

    const pending = render(document.createElement("div"));
    const script = document.querySelector<HTMLScriptElement>('script[src*="gsi/client"]');
    expect(script).not.toBeNull();

    // The SDK "arrives", then its load event fires — the loader resolves and renders.
    const { initialize, renderButton } = installGsi();
    script!.dispatchEvent(new Event("load"));
    await pending;

    expect(status.value.status).toBe("success");
    expect(initialize).toHaveBeenCalled();
    expect(renderButton).toHaveBeenCalled();
  });

  it("fails the status when the script errors", async () => {
    const { useGoogleSignIn } = await import("../src/google");
    const { status, render } = useGoogleSignIn("c", vi.fn());

    const pending = render(document.createElement("div"));
    const script = document.querySelector<HTMLScriptElement>('script[src*="gsi/client"]');
    script!.dispatchEvent(new Event("error"));
    await pending;

    expect(status.value).toMatchObject({ status: "error" });
  });
});
