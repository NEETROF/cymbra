import { type AuthErrorKind, authErrorOf } from "../state/auth-errors.ts";

// When the background syncs (add-lingua-connected-clients §2.4). One run at a time; a burst of
// mutations coalesces into one debounced run; a trigger arriving mid-run is drained after it.
// Opening a surface or loading a page asks for a run, granted once per interval across
// event-page restarts (the last success is persisted); Réglages forces one and hears how it
// went. The erasure holds every run while it clears the server and the device.
//
// `onOpen` resolves only once its run is over, so the caller's pending message response keeps
// Safari's event page alive: answering first and syncing after had the page suspended
// mid-exchange, and nothing ever arrived unless the reader pressed « Synchroniser maintenant ».

/** A surface the reader just opened (popup, drawer, side panel). */
export const SURFACE_INTERVAL_MS = 10_000;
/** A page load or a return to a tab — frequent, so a longer wait between runs. */
export const PAGE_INTERVAL_MS = 60_000;

export interface SchedulerDeps {
  /** One full exchange; throws on failure. */
  sync: () => Promise<void>;
  signedIn: () => boolean;
  now: () => number;
  /** When the last run succeeded (epoch millis, 0 if never), persisted by `onSynced`. */
  lastSynced: () => Promise<number>;
  onSynced: (at: number) => Promise<void>;
  /** A failed run, after which the caller rebuilds whatever it memoised. */
  onError: (e: unknown) => void;
}

export class SyncScheduler {
  private running: Promise<AuthErrorKind | null> | null = null;
  private pending = false;
  private holding = false;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private lastAttempt = 0;

  constructor(private readonly deps: SchedulerDeps) {}

  /** Run after `delayMs`, restarting the delay on each call. Nothing while signed out. */
  schedule(delayMs: number): void {
    if (!this.deps.signedIn()) return;
    if (this.running || this.holding) {
      this.pending = true;
      return;
    }
    this.cancelTimer();
    this.timer = setTimeout(() => {
      this.timer = null;
      if (this.running || this.holding) this.pending = true;
      else void this.start();
    }, delayMs);
  }

  /**
   * A surface opened or a page loaded: run now unless one started or succeeded within
   * `minIntervalMs`. Resolves when the run is over (or at once when there is nothing to do),
   * so the caller can hold its message response until then.
   */
  async onOpen(minIntervalMs: number = SURFACE_INTERVAL_MS): Promise<void> {
    const last = Math.max(this.lastAttempt, await this.deps.lastSynced());
    if (this.deps.now() - last < minIntervalMs) return;
    await (this.start() ?? this.running ?? Promise.resolve());
  }

  /**
   * « Synchroniser maintenant »: wait for a run in flight (it may predate the latest change),
   * then run one and report its failure category, or null when it succeeded.
   */
  async syncNow(): Promise<AuthErrorKind | null> {
    if (!this.deps.signedIn()) return "unauthenticated";
    if (this.holding) return "conflict";
    if (this.running) await this.running;
    // The run just awaited may have scheduled a drain: this run covers it.
    this.cancelTimer();
    const run = this.start();
    return run ? await run : "conflict";
  }

  /** Run `task` with every sync held, after the one in flight; drain the triggers it held. */
  async exclusive<T>(task: () => Promise<T>): Promise<T> {
    this.holding = true;
    try {
      this.cancelTimer();
      if (this.running) await this.running;
      return await task();
    } finally {
      this.holding = false;
      this.schedule(0);
    }
  }

  /** Start a run, or return undefined when one cannot start now. */
  private start(): Promise<AuthErrorKind | null> | undefined {
    if (this.running || this.holding || !this.deps.signedIn()) return undefined;
    this.cancelTimer();
    this.pending = false;
    this.lastAttempt = this.deps.now();
    const run = this.exchange().finally(() => {
      this.running = null;
      if (this.pending) this.schedule(0);
    });
    this.running = run;
    return run;
  }

  private async exchange(): Promise<AuthErrorKind | null> {
    try {
      await this.deps.sync();
      await this.deps.onSynced(this.deps.now());
      return null;
    } catch (e) {
      this.deps.onError(e);
      return authErrorOf(e);
    }
  }

  private cancelTimer(): void {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}
