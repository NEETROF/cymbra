import type { InterfaceLanguage } from "../i18n/index.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import type { UsageClient } from "./client.ts";
import {
  afterReport,
  type BrowserFamily,
  type Consent,
  type CountsMode,
  countingOn,
  due,
  forget,
  install,
  leavingUrl,
  type NoticeKind,
  noticeKind,
  parseState,
  recordAnalysis,
  recording,
  record,
  recordUse,
  localDay,
  sending,
  shortVersion,
  type SwitchView,
  switchView,
  type UsageState,
  USAGE_COUNTS_KEY,
  USAGE_COUNTS_OFF_KEY,
} from "./counts.ts";
import type { SwitchReply, UsageMessage } from "./messages.ts";

// The background's side of the anonymous usage counts (change: add-lingua-usage-counts, D1, D5,
// D6, D7, D12): it records a step when the background already wakes for a message it receives —
// an installation, a page analysed, a surface opened, a level saved — and sends what waits in one
// report while it is awake. No alarm, no timer. Every operation runs one after the other, so two
// tabs' messages never send the same steps twice.

/** What the host needs from the background (wired in background.ts, doubled in tests). */
export interface UsageHostDeps {
  /** `chrome.storage.local`: the counts' state and the switch. */
  area: AsyncStorageArea;
  mode: CountsMode;
  browser: BrowserFamily;
  /** The extension's version (`chrome.runtime.getManifest().version`). */
  version: string;
  /** The reader's choice, read fresh each time (Firefox's consent, or the switch's key). */
  consent: () => Promise<Consent>;
  /** The counts' client, with no account on it (usage/client.ts). */
  client: () => UsageClient;
  now: () => Date;
  /** The interface language: the leaving page follows it. */
  language: () => Promise<InterfaceLanguage>;
  /** `runtime.setUninstallURL`; null where the browser ignores it (Safari). */
  setUninstallURL: ((url: string) => Promise<void>) | null;
  /** Firefox's consent, asked or withdrawn (D6); absent elsewhere. */
  dataCollection?: {
    request: () => Promise<boolean>;
    remove: () => Promise<boolean>;
  };
}

export class UsageHost {
  private queue: Promise<unknown> = Promise.resolve();
  /**
   * Set once this page has seen an installation with no day 0 and nothing waiting — one that predates
   * the counts, or whose day 59 passed, or that was turned off: it will never record again, and a
   * page's every analysis need not read the storage to find that out. Only an installation clears it.
   */
  private inert = false;

  constructor(private readonly deps: UsageHostDeps) {}

  /** One operation at a time, whatever the one before did. */
  private serial<T>(task: () => Promise<T>): Promise<T> {
    const run = this.queue.then(task, task);
    this.queue = run.catch(() => undefined);
    return run;
  }

  private async load(): Promise<UsageState> {
    const got = await this.deps.area.get(USAGE_COUNTS_KEY);
    return parseState(got[USAGE_COUNTS_KEY]);
  }

  private async save(before: UsageState, after: UsageState): Promise<void> {
    if (JSON.stringify(before) !== JSON.stringify(after)) await this.deps.area.set({ [USAGE_COUNTS_KEY]: after });
  }

  private today(): number {
    return localDay(this.deps.now());
  }

  /**
   * Change the state with `change` if this installation still records, then send what waits. Off,
   * whatever this device held is forgotten.
   */
  private step(change: (state: UsageState, today: number) => UsageState, skipWhenInert = true): Promise<void> {
    return this.serial(async () => {
      if (skipWhenInert && this.inert) return;
      const consent = await this.deps.consent();
      const before = await this.load();
      if (!recording(consent)) {
        await this.save(before, forget(before));
        return;
      }
      const after = change(before, this.today());
      await this.save(before, after);
      this.inert = after.day0 === null && after.waiting.length === 0;
      await this.flush(after, consent);
    });
  }

  /** Report every waiting step in one report, when this browser's consent and the backoff allow. */
  private async flush(state: UsageState, consent: Consent): Promise<void> {
    const now = this.deps.now().getTime();
    if (!sending(this.deps.mode, consent, state) || !due(state, now) || state.week === null) return;
    const steps = [...state.waiting];
    const outcome = await this.deps
      .client()
      .report({ installWeek: state.week, browser: this.deps.browser, version: shortVersion(this.deps.version), steps })
      .catch(() => "retry" as const);
    await this.save(state, afterReport(state, steps, outcome, this.deps.now().getTime()));
  }

  /** `runtime.onInstalled`: an installation (not an update) starts day 0 with « installed ». */
  installed(reason: string): Promise<void> {
    if (reason !== "install") return this.wake();
    return this.step((state, today) => install(state, today), false);
  }

  /** A page analysed in a studied language: the first analysis, and a use. */
  analysed(): Promise<void> {
    return this.step((state, today) => recordAnalysis(state, today));
  }

  /** A Lingua surface the reader opened: a use. */
  used(): Promise<void> {
    return this.step((state, today) => recordUse(state, today));
  }

  /** A level saved: « set up ». */
  setUp(): Promise<void> {
    return this.step((state, today) => record(state, "set_up", today));
  }

  /** A wake with nothing new (a browser start): try what waits. */
  wake(): Promise<void> {
    return this.step((state) => state);
  }

  /** The first-run sentence was shown: in Safari, that opens the gate (D7). */
  noticeShown(): Promise<void> {
    return this.step((state) => ({ ...state, noticeShown: true }), false);
  }

  /** The reader answered the sentence's question (Chromium): yes counts, no is the switch off. */
  async answer(yes: boolean): Promise<void> {
    await this.serial(async () => {
      await this.deps.area.set({ [USAGE_COUNTS_OFF_KEY]: !yes });
      const before = await this.load();
      await this.save(before, { ...before, noticeShown: true });
    });
    await this.consentChanged();
  }

  /** What the first-run sentence must do now. */
  status(): Promise<NoticeKind> {
    return this.serial(async () => noticeKind(this.deps.mode, await this.deps.consent(), await this.load()));
  }

  /** What Settings → Data shows. */
  async switchState(): Promise<SwitchView> {
    return switchView(this.deps.mode, await this.deps.consent());
  }

  /** The switch moved (D6): its key on Chromium and Safari — on Chromium, ticking it is a yes —, Firefox's consent there. */
  async setSwitch(on: boolean): Promise<SwitchReply> {
    const consent = await this.deps.consent();
    let fromToolbar = false;
    if (consent.kind === "switch") {
      await this.deps.area.set({ [USAGE_COUNTS_OFF_KEY]: !on });
    } else if (consent.supported && this.deps.dataCollection) {
      if (!on) await this.deps.dataCollection.remove().catch(() => false);
      // Granted already when the page asked from its click; asked here, it needs a gesture this
      // message does not carry, and the reader is pointed at the toolbar.
      else if (!consent.granted) fromToolbar = !(await this.deps.dataCollection.request().catch(() => false));
    }
    await this.consentChanged();
    const view = await this.switchState();
    return fromToolbar && !view.on ? { view, fromToolbar } : { view };
  }

  /** A full reset of this device's Lingua data: day 0 and the steps not sent go; the switch stays. */
  forgetAll(): Promise<void> {
    return this.serial(async () => {
      const before = await this.load();
      await this.save(before, forget(before));
    });
  }

  /** The reader's choice changed (the switch, the answer, Firefox's consent): follow it. */
  async consentChanged(): Promise<void> {
    await this.step((state) => state);
    await this.registerLeavingPage();
  }

  /**
   * The leaving page while counting is on, in the interface language, with the browser and the
   * version only; nothing otherwise (D12). Safari ignores the address: it is never set there.
   */
  async registerLeavingPage(): Promise<void> {
    const set = this.deps.setUninstallURL;
    if (!set) return;
    try {
      const on = countingOn(this.deps.mode, await this.deps.consent());
      await set(on ? leavingUrl(await this.deps.language(), this.deps.browser, this.deps.version) : "");
    } catch {
      // Unsupported or refused: the leaving page is a bonus, never a failure.
    }
  }

  /** Answer one of the surfaces' messages. */
  async handle(message: UsageMessage): Promise<unknown> {
    switch (message.type) {
      case "usage:step":
        await this.setUp();
        return { ok: true };
      case "usage:notice-shown":
        await this.noticeShown();
        return { ok: true };
      case "usage:consent":
        await this.answer(message.yes);
        return { ok: true };
      case "usage:status":
        return { notice: await this.status() };
      case "usage:switch-state":
        return await this.switchState();
      case "usage:switch":
        return await this.setSwitch(message.on);
      case "usage:forget":
        await this.forgetAll();
        return { ok: true };
    }
  }
}
