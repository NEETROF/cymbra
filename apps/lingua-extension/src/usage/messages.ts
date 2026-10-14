import { USAGE_COUNTS_KEY, USAGE_COUNTS_OFF_KEY, type NoticeKind, type SwitchView } from "./counts.ts";

// The anonymous usage counts' messages between the surfaces and the background, which owns their
// state (change: add-lingua-usage-counts). A surface never writes the counts itself: it says what
// happened — a level saved, the first-run sentence shown, the reader's answer, the switch moved, a
// full reset — and the background records, forgets and reports. Nothing here fails its caller.

export type UsageMessage =
  /** A level was saved, the onboarding's or Réglages' (D1, « set up »). */
  | { type: "usage:step"; step: "set_up" }
  /** The first-run sentence was shown (Safari's gate, D7). */
  | { type: "usage:notice-shown" }
  /** The reader's answer to the sentence's two buttons (Chromium, D7). */
  | { type: "usage:consent"; yes: boolean }
  /** What the first-run sentence must do now. */
  | { type: "usage:status" }
  /** What Settings → Data shows. */
  | { type: "usage:switch-state" }
  /** The switch moved. */
  | { type: "usage:switch"; on: boolean }
  /** A full reset: forget day 0 and the steps not sent; the switch stays as it is. */
  | { type: "usage:forget" };

/** The answer to a move of the switch. */
export interface SwitchReply {
  view: SwitchView;
  /**
   * Firefox, where the move could not be made from here: its consent can only be asked from a
   * click in one of the extension's own pages (the toolbar's), not from the in-page drawer.
   */
  fromToolbar?: boolean;
}

const TYPES: ReadonlySet<string> = new Set([
  "usage:step",
  "usage:notice-shown",
  "usage:consent",
  "usage:status",
  "usage:switch-state",
  "usage:switch",
  "usage:forget",
]);

export function isUsageMessage(m: unknown): m is UsageMessage {
  const type = (m as { type?: unknown } | null)?.type;
  return typeof type === "string" && TYPES.has(type);
}

export type RuntimeSend = (message: unknown) => Promise<unknown>;

const runtimeSend: RuntimeSend = (message) => chrome.runtime.sendMessage(message);

/** Send, swallowing an unreachable background or a context with no runtime. */
async function quietly(send: RuntimeSend, message: UsageMessage): Promise<unknown> {
  try {
    return await send(message);
  } catch {
    return undefined;
  }
}

/** A level was saved: « set up ». */
export async function reportSetUp(send: RuntimeSend = runtimeSend): Promise<void> {
  await quietly(send, { type: "usage:step", step: "set_up" });
}

/** The seam the first-run sentence goes through (usage/notice.ts). */
export interface NoticeControls {
  status: () => Promise<NoticeKind>;
  shown: () => Promise<void>;
  answer: (yes: boolean) => Promise<void>;
}

export function runtimeNoticeControls(send: RuntimeSend = runtimeSend): NoticeControls {
  return {
    status: async () => {
      const reply = (await quietly(send, { type: "usage:status" })) as { notice?: NoticeKind } | undefined;
      return reply?.notice ?? "none";
    },
    shown: async () => void (await quietly(send, { type: "usage:notice-shown" })),
    answer: async (yes) => void (await quietly(send, { type: "usage:consent", yes })),
  };
}

/** The seam the switch in Settings → Data goes through (reading/settings-view.ts). */
export interface UsageSwitchControls {
  state: () => Promise<SwitchView>;
  set: (on: boolean) => Promise<SwitchReply>;
  forget: () => Promise<void>;
  /** Call `onChange` when the counts' state or switch change; returns how to stop. */
  watch: (onChange: () => void) => (() => void) | void;
}

/** Firefox's built-in consent the counts follow (D6). */
export const DATA_COLLECTION = "technicalAndInteraction";

/** The permissions API with Firefox's `data_collection` (absent from Chrome's types). */
export interface DataCollectionPermissions {
  request: (p: { data_collection: string[] }) => Promise<boolean>;
}

const HIDDEN: SwitchView = { shown: false, on: false };

export function runtimeUsageSwitch(
  send: RuntimeSend = runtimeSend,
  permissions: DataCollectionPermissions | undefined = firefoxPermissions(),
): UsageSwitchControls {
  return {
    state: async () => ((await quietly(send, { type: "usage:switch-state" })) as SwitchView | undefined) ?? HIDDEN,
    set: async (on) => {
      // Firefox asks its consent from the click itself: called before anything is awaited, in an
      // extension page (the popup, the panel). The in-page drawer has no permissions API.
      if (on && permissions) {
        try {
          if (!(await permissions.request({ data_collection: [DATA_COLLECTION] }))) {
            return { view: { shown: true, on: false } };
          }
        } catch {
          // Refused outright: the background says what is possible.
        }
      }
      const reply = (await quietly(send, { type: "usage:switch", on })) as SwitchReply | undefined;
      return reply ?? { view: HIDDEN };
    },
    forget: async () => void (await quietly(send, { type: "usage:forget" })),
    watch: (onChange) => {
      try {
        const listener = (changes: Record<string, chrome.storage.StorageChange>, area: string): void => {
          if (area === "local" && (changes[USAGE_COUNTS_KEY] || changes[USAGE_COUNTS_OFF_KEY])) onChange();
        };
        chrome.storage.onChanged.addListener(listener);
        return () => chrome.storage.onChanged.removeListener(listener);
      } catch {
        return undefined;
      }
    },
  };
}

/** The permissions API of an extension page on Firefox; none elsewhere (folded on other builds). */
function firefoxPermissions(): DataCollectionPermissions | undefined {
  if (__TARGET__ !== "firefox") return undefined;
  try {
    return (chrome.permissions as unknown as DataCollectionPermissions | undefined)?.request
      ? (chrome.permissions as unknown as DataCollectionPermissions)
      : undefined;
  } catch {
    return undefined;
  }
}
