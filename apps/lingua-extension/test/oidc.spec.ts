import { describe, expect, it, vi } from "vitest";
import {
  appleAuthorizeRequest,
  availableProviders,
  googleAuthorizeRequest,
  idTokenFromRedirect,
  isUserCancel,
  runAuthFlow,
} from "@/state/oidc.ts";

const params = {
  clientId: "app.cymbra.web",
  redirectUri: "https://figfjglfdiffocldficbimecjnhnkhkh.chromiumapp.org/",
  state: "S1",
  nonce: "N1",
};
const REDIRECT = params.redirectUri;

describe("appleAuthorizeRequest", () => {
  it("asks for no scope, in the fragment, with state and nonce", () => {
    const req = appleAuthorizeRequest(params);
    const url = new URL(req.url);
    expect(url.origin + url.pathname).toBe("https://appleid.apple.com/auth/authorize");
    expect(url.searchParams.get("client_id")).toBe("app.cymbra.web");
    expect(url.searchParams.get("redirect_uri")).toBe(REDIRECT);
    expect(url.searchParams.get("response_type")).toBe("code id_token");
    expect(url.searchParams.get("response_mode")).toBe("fragment");
    expect(url.searchParams.has("scope")).toBe(false);
    expect(url.searchParams.get("state")).toBe("S1");
    expect(url.searchParams.get("nonce")).toBe("N1");
    expect(req.state).toBe("S1");
  });
});

describe("googleAuthorizeRequest", () => {
  it("runs the OpenID implicit flow with state", () => {
    const url = new URL(googleAuthorizeRequest(params).url);
    expect(url.origin + url.pathname).toBe("https://accounts.google.com/o/oauth2/v2/auth");
    expect(url.searchParams.get("response_type")).toBe("id_token");
    expect(url.searchParams.get("scope")).toBe("openid email");
    expect(url.searchParams.get("state")).toBe("S1");
  });
});

describe("idTokenFromRedirect", () => {
  it("reads the id_token from the fragment and ignores the code", () => {
    expect(idTokenFromRedirect(`${REDIRECT}#code=C&id_token=T&state=S1`, "S1")).toBe("T");
  });

  it("rejects a state mismatch", () => {
    expect(() => idTokenFromRedirect(`${REDIRECT}#id_token=T&state=OTHER`, "S1")).toThrow(/state/);
  });

  it("rejects a redirect without id_token", () => {
    expect(() => idTokenFromRedirect(`${REDIRECT}#code=C&state=S1`, "S1")).toThrow(/id_token/);
  });

  it("returns null when the provider reports a cancel (fragment or query)", () => {
    expect(idTokenFromRedirect(`${REDIRECT}#error=user_cancelled_authorize&state=S1`, "S1")).toBeNull();
    expect(idTokenFromRedirect(`${REDIRECT}?error=access_denied`, "S1")).toBeNull();
  });

  it("throws on another provider error", () => {
    expect(() => idTokenFromRedirect(`${REDIRECT}#error=invalid_request`, "S1")).toThrow(/invalid_request/);
  });
});

describe("isUserCancel", () => {
  it("recognizes Chrome's and Firefox's closed-window rejections", () => {
    expect(isUserCancel(new Error("The user did not approve access."))).toBe(true);
    expect(isUserCancel(new Error("User cancelled or denied access."))).toBe(true);
  });

  it("does not swallow real failures", () => {
    expect(isUserCancel(new Error("Authorization page could not be loaded."))).toBe(false);
  });
});

describe("runAuthFlow", () => {
  const req = appleAuthorizeRequest(params);

  it("returns the id_token from the final redirect", async () => {
    const launch = vi.fn(async () => `${REDIRECT}#id_token=T&state=S1`);
    expect(await runAuthFlow(launch, req)).toBe("T");
    expect(launch).toHaveBeenCalledWith(req.url);
  });

  it("returns null when the reader closes the window", async () => {
    const launch = async (): Promise<string> => {
      throw new Error("The user did not approve access.");
    };
    expect(await runAuthFlow(launch, req)).toBeNull();
    expect(await runAuthFlow(async () => undefined, req)).toBeNull();
  });

  it("rethrows other flow failures", async () => {
    const launch = async (): Promise<string> => {
      throw new Error("Authorization page could not be loaded.");
    };
    await expect(runAuthFlow(launch, req)).rejects.toThrow(/could not be loaded/);
  });
});

describe("availableProviders", () => {
  const ids = { google: "g.apps.googleusercontent.com", apple: "app.cymbra.web" };

  it("offers a provider only when configured and the identity API exists", () => {
    expect(availableProviders(ids, { launchWebAuthFlow: () => {} })).toEqual({ google: true, apple: true });
    expect(availableProviders({ google: ids.google, apple: "" }, { launchWebAuthFlow: () => {} })).toEqual({
      google: true,
      apple: false,
    });
  });

  it("offers neither without identity.launchWebAuthFlow (Firefox for Android)", () => {
    expect(availableProviders(ids, undefined)).toEqual({ google: false, apple: false });
    expect(availableProviders(ids, {})).toEqual({ google: false, apple: false });
  });
});
