import { ref } from "vue";
import { type Async, idle, run } from "@/lib/async";

// Google Identity Services (GSI) — the browser SDK that mints a Google id_token.
// We load it on demand (only when a client id is configured), render its official
// button, and hand the resulting credential to the caller. The backend exchange
// (SignInOidc) stays in the auth store; this composable is pure browser glue.
const GSI_SRC = "https://accounts.google.com/gsi/client";

interface CredentialResponse {
  readonly credential?: string;
}

// Minimal shape of the `google.accounts.id` surface we use — GSI ships no types.
interface GsiButtonOptions {
  type?: "standard" | "icon";
  theme?: "outline" | "filled_blue" | "filled_black";
  size?: "large" | "medium" | "small";
  text?: "signin_with" | "signup_with" | "continue_with" | "signin";
  shape?: "rectangular" | "pill" | "circle" | "square";
  width?: number;
  locale?: string;
}
interface GsiId {
  initialize(config: { client_id: string; callback: (response: CredentialResponse) => void }): void;
  renderButton(parent: HTMLElement, options: GsiButtonOptions): void;
}
declare global {
  interface Window {
    google?: { accounts?: { id?: GsiId } };
  }
}

// Load the GSI script at most once per page; concurrent callers share the promise.
let loader: Promise<GsiId> | null = null;
function loadGsi(): Promise<GsiId> {
  const ready = window.google?.accounts?.id;
  if (ready) return Promise.resolve(ready);
  loader ??= new Promise<GsiId>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = GSI_SRC;
    script.async = true;
    script.defer = true;
    script.addEventListener(
      "load",
      () => {
        const id = window.google?.accounts?.id;
        if (id) resolve(id);
        else reject(new Error("Google Identity Services unavailable after load"));
      },
      { once: true },
    );
    script.addEventListener(
      "error",
      () => {
        loader = null; // let a later attempt retry the load
        reject(new Error("Failed to load Google Identity Services"));
      },
      { once: true },
    );
    document.head.appendChild(script);
  });
  return loader;
}

/**
 * Load GSI and render its sign-in button into `parent`. The credential (a Google
 * id_token) is passed to `onCredential`; the caller exchanges it for a Cymbra token
 * via the auth store. `status` is the load state so the view can keep the slot
 * hidden until the button is ready and show a fallback if GSI never loads.
 */
export function useGoogleSignIn(clientId: string, onCredential: (idToken: string) => void) {
  const status = ref<Async<void>>(idle);

  async function render(parent: HTMLElement, options: GsiButtonOptions = {}): Promise<void> {
    await run(status, async () => {
      const id = await loadGsi();
      id.initialize({
        client_id: clientId,
        callback: (response) => {
          if (response.credential) onCredential(response.credential);
        },
      });
      id.renderButton(parent, {
        type: "standard",
        theme: "outline",
        size: "large",
        text: "signin_with",
        shape: "pill",
        ...options,
      });
    });
  }

  return { status, render };
}
