// Platform capabilities the extension's own surfaces adapt to, detected at runtime so
// one bundle serves desktop and mobile browsers alike.

/**
 * Whether the browser can run a provider OAuth flow (`identity.launchWebAuthFlow`). Safari
 * has no identity API, so provider buttons that would fail there are not offered — a feature
 * detection, not a target check, so a browser gaining the API gets the buttons back.
 */
export function hasWebAuthFlow(identity: { launchWebAuthFlow?: unknown } | undefined = globalThis.chrome?.identity): boolean {
  return typeof identity?.launchWebAuthFlow === "function";
}

/** Whether the primary pointer is coarse: a touch-first device (phone, tablet). */
export function isTouchPrimary(): boolean {
  return typeof matchMedia === "function" && matchMedia("(pointer: coarse)").matches;
}

/**
 * Whether the browser has a keyboard-shortcut editor worth linking to. Safari has none on
 * any platform, and a touch-primary device (Firefox for Android) has no keyboard to use the
 * shortcuts and no editor page to open: `about:addons` / `chrome://extensions/shortcuts`
 * land on an error page there.
 */
export function hasShortcutEditor(target: typeof __TARGET__ = __TARGET__): boolean {
  return target !== "safari" && !isTouchPrimary();
}
