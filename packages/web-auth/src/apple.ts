import { ref } from "vue";
import { type Async, idle, run } from "./async";
import type { SignInOptions } from "./google";

// Sign in with Apple (web) — Apple's JS SDK, loaded on demand. Unlike Google's GSI,
// Apple renders no button for us: we init the SDK, show our own Apple-styled button,
// and call `AppleID.auth.signIn()` (popup flow) on click. The resulting id_token is
// handed to the caller, who exchanges it for a Cymbra token via the auth store.
//
// Web needs a Services ID as the client id (NOT the app bundle id) and a registered
// Return URL (each consuming origin — bo.cymbra.app, cymbra.app — is declared on the
// Services ID). Until those exist, `signIn()` errors; the button stays hidden entirely
// when no client id is configured.
const SDK_BASE = "https://appleid.cdn-apple.com/appleauth/static/jsapi/appleid/1";

// Apple locale segment in the SDK URL (drives the SDK's own copy). We ship en/fr.
const SDK_LOCALE: Record<string, string> = { en: "en_US", fr: "fr_FR" };

interface AppleAuthResponse {
  readonly authorization?: { readonly id_token?: string };
}
interface AppleAuthInit {
  clientId: string;
  scope?: string;
  redirectURI: string;
  usePopup?: boolean;
}
interface AppleId {
  auth: {
    init(config: AppleAuthInit): void;
    signIn(): Promise<AppleAuthResponse>;
  };
}
declare global {
  interface Window {
    AppleID?: AppleId;
  }
}

// Load the Apple SDK at most once per page; concurrent callers share the promise.
let loader: Promise<AppleId> | null = null;
function loadAppleId(locale: string): Promise<AppleId> {
  if (window.AppleID) return Promise.resolve(window.AppleID);
  loader ??= new Promise<AppleId>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = `${SDK_BASE}/${SDK_LOCALE[locale] ?? "en_US"}/appleid.auth.js`;
    script.async = true;
    script.defer = true;
    script.addEventListener(
      "load",
      () => {
        if (window.AppleID) resolve(window.AppleID);
        else reject(new Error("Apple ID SDK unavailable after load"));
      },
      { once: true },
    );
    script.addEventListener(
      "error",
      () => {
        loader = null; // let a later attempt retry the load
        reject(new Error("Failed to load Apple ID SDK"));
      },
      { once: true },
    );
    document.head.appendChild(script);
  });
  return loader;
}

/**
 * Load + initialise the Apple SDK, then start the popup sign-in on demand. `status`
 * is the SDK load state so the view can keep the button hidden until it is ready and
 * show a fallback if the SDK never loads. `signIn()` opens Apple's popup and passes
 * the resulting id_token to `onCredential`; a cancelled popup rejects (the caller
 * swallows it — Apple's own UI already told the user).
 */
export function useAppleSignIn(
  clientId: string,
  redirectUri: string,
  onCredential: (idToken: string) => void,
  opts: SignInOptions = {},
) {
  const status = ref<Async<void>>(idle);
  let sdk: AppleId | null = null;

  async function load(locale: string): Promise<void> {
    await run(
      status,
      async () => {
        sdk = await loadAppleId(locale);
        sdk.auth.init({ clientId, scope: "name email", redirectURI: redirectUri, usePopup: true });
      },
      opts.mapError,
    );
  }

  async function signIn(): Promise<void> {
    if (!sdk) return;
    const response = await sdk.auth.signIn();
    const idToken = response.authorization?.id_token;
    if (idToken) onCredential(idToken);
  }

  return { status, load, signIn };
}
