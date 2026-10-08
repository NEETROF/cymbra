import type { InterfaceLanguage } from "../i18n/language.ts";
import type { Provider, Providers } from "./oidc.ts";

// Apple and Google on Safari (add-lingua-connected-clients, design D3/D6). Safari has no
// identity.launchWebAuthFlow, so the host app (apps/lingua-apple) runs the native Apple or
// Google sheet and hands the id_token back through this extension's native handler, which
// returns it once. The same handler keeps the interface language for the app's activation page
// (localise-lingua-apple-host, D2). Pure helpers; the background supplies the browser APIs.

/** The host app's URL scheme (CFBundleURLTypes in apps/lingua-apple). */
export const HOST_APP_SCHEME = "cymbra-lingua";

/** The application id given to sendNativeMessage: Safari always routes to the containing app's handler. */
export const NATIVE_APP_ID = "com.cymbra.lingua";

/**
 * The URL that opens the host app on a provider's sign-in sheet, in the interface language when
 * one is given (localise-lingua-apple-host D3): the sheet is a step of a flow begun on a page in
 * that language. The app reads `lang` from a closed list and shows the link's language only among
 * the ones it offers; a link naming none shows the app's preferred localisation.
 */
export function hostAppSignInUrl(provider: Provider, language?: InterfaceLanguage): string {
  const url = `${HOST_APP_SCHEME}://signin?provider=${provider}`;
  return language ? `${url}&lang=${language}` : url;
}

/** What the extension sends the host app's native handler (SafariWebExtensionHandler, NativeMessage.swift). */
export type NativeMessage =
  | { type: "auth.providers" }
  | { type: "auth.takeIdToken" }
  /** The interface language, kept in the App Group for the activation page (localise-lingua-apple-host D2). */
  | { type: "interface.language"; language: InterfaceLanguage };

export interface HandedIdToken {
  provider: Provider;
  idToken: string;
}

/** Read the handler's answer to `auth.takeIdToken`: a token, or null when nothing is pending or the reply is malformed. */
export function parseHandedIdToken(reply: unknown): HandedIdToken | null {
  const r = reply as { provider?: unknown; idToken?: unknown } | null;
  if (r?.provider !== "apple" && r?.provider !== "google") return null;
  return typeof r.idToken === "string" && r.idToken.length > 0 ? { provider: r.provider, idToken: r.idToken } : null;
}

export type NativeSend = (message: NativeMessage) => Promise<unknown>;

/**
 * The providers the host app offers: Apple always, Google once the app is built with a Google
 * client. A missing or failing handler offers Apple only.
 */
export async function nativeProviders(send: NativeSend): Promise<Providers> {
  try {
    const reply = (await send({ type: "auth.providers" })) as { google?: unknown } | null;
    return { apple: true, google: reply?.google === true };
  } catch {
    return { apple: true, google: false };
  }
}

/** Ask the native handler for a pending id_token. A missing or failing handler means nothing is pending. */
export async function takeHandedIdToken(send: NativeSend): Promise<HandedIdToken | null> {
  try {
    return parseHandedIdToken(await send({ type: "auth.takeIdToken" }));
  } catch {
    return null;
  }
}

/**
 * Tell the host app the interface language, so its activation page follows the extension rather
 * than the device (M22). A missing or failing handler is nothing to act on: the page then follows
 * the device, as before the extension first ran.
 */
export async function tellInterfaceLanguage(send: NativeSend, language: InterfaceLanguage): Promise<void> {
  try {
    await send({ type: "interface.language", language });
  } catch {
    /* the page follows the device until the handler answers */
  }
}
