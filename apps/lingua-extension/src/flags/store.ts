import type { AsyncStorageArea } from "../state/storage.ts";

// The extension's effective server flags (change: add-lingua-feature-betas): one snapshot per
// account, fetched by the background and read locally by every surface. The server stays the
// authority for anything a flag gates — this only decides what a surface shows, so a stale
// snapshot can show a feature that then answers "unavailable", never grant one.

/** The persisted snapshot, in chrome.storage.local. */
export const FLAGS_KEY = "cymbra-lingua-flags";
/** Outside sign-in, a fetch at most this often (a surface opens often). */
export const FLAG_REFRESH_MS = 5 * 60_000;

/** A flag's value as a surface reads it. */
export type FlagScalar = boolean | number | string;
export type FlagValues = Readonly<Record<string, FlagScalar>>;

/** One flag as the server sends it (the shape of `EffectiveFlag`, without the generated type). */
export interface WireFlag {
  key: string;
  value?: { kind: { case?: string; value?: unknown } };
}

export interface WireFlags {
  version: string;
  unchanged: boolean;
  flags: readonly WireFlag[];
}

interface Snapshot {
  account: string;
  version: string;
  values: Record<string, FlagScalar>;
  fetchedAt: number;
}

export interface FlagStoreDeps {
  /** `GetEffectiveFlags` with the last known version ("" for none). */
  fetch(knownVersion: string): Promise<WireFlags>;
  area: AsyncStorageArea;
  /** The signed-in account (the access token's subject), or null when signed out. */
  account(): string | null;
  now?: () => number;
}

/** A wire value as a scalar; a JSON value stays its string, an empty one is dropped. */
function scalar(value: WireFlag["value"]): FlagScalar | undefined {
  const v = value?.kind.value;
  switch (value?.kind.case) {
    case "boolValue":
      return v === true;
    case "intValue":
      return Number(v);
    case "numberValue":
      return typeof v === "number" ? v : Number(v);
    case "stringValue":
    case "jsonValue":
      return typeof v === "string" ? v : undefined;
    default:
      return undefined;
  }
}

export function toValues(flags: readonly WireFlag[]): Record<string, FlagScalar> {
  const out: Record<string, FlagScalar> = {};
  for (const f of flags) {
    const v = scalar(f.value);
    if (v !== undefined) out[f.key] = v;
  }
  return out;
}

/** A flag read as a boolean: anything but `true` — absent, never fetched, signed out — is off. */
export function flagOn(values: FlagValues, key: string): boolean {
  return values[key] === true;
}

export class FlagStore {
  private inFlight: Promise<void> | null = null;

  constructor(private readonly deps: FlagStoreDeps) {}

  private now(): number {
    return (this.deps.now ?? Date.now)();
  }

  private async load(): Promise<Snapshot | null> {
    const got = (await this.deps.area.get(FLAGS_KEY))[FLAGS_KEY] as Snapshot | null | undefined;
    return got && typeof got.account === "string" ? got : null;
  }

  /** The signed-in account's flags; empty when signed out or for another account's snapshot. */
  async read(): Promise<FlagValues> {
    const account = this.deps.account();
    if (account == null) return {};
    const snap = await this.load();
    return snap?.account === account ? snap.values : {};
  }

  /** Forget the snapshot (sign-out): no flag of this account may apply to the next one. */
  async clear(): Promise<void> {
    await this.deps.area.set({ [FLAGS_KEY]: null });
  }

  /**
   * Fetch the flags unless a fetch for this account ran within `FLAG_REFRESH_MS` (`force`
   * ignores that: sign-in). One fetch at a time; a failed one keeps the last-good snapshot.
   */
  refresh(opts: { force?: boolean } = {}): Promise<void> {
    this.inFlight ??= this.run(opts.force === true).finally(() => {
      this.inFlight = null;
    });
    return this.inFlight;
  }

  private async run(force: boolean): Promise<void> {
    // Signed out, or the session not resumed yet: nothing to ask for. The snapshot is not
    // touched — sign-out clears it explicitly, and another account never reads it.
    const account = this.deps.account();
    if (account == null) return;
    const prev = await this.load();
    const mine = prev?.account === account ? prev : null;
    if (!force && mine && this.now() - mine.fetchedAt < FLAG_REFRESH_MS) return;
    let wire: WireFlags;
    try {
      wire = await this.deps.fetch(mine?.version ?? "");
    } catch (e) {
      console.warn("[Cymbra Lingua] flags fetch failed; keeping the last snapshot:", e);
      return;
    }
    // The account may have changed while the fetch was in flight: never file one account's
    // answer under another.
    if (this.deps.account() !== account) return;
    const values = wire.unchanged && mine ? mine.values : toValues(wire.flags);
    const snap: Snapshot = { account, version: wire.version, values, fetchedAt: this.now() };
    await this.deps.area.set({ [FLAGS_KEY]: snap });
  }
}

/** The subject (`sub`) of an access token, or null when it cannot be read. */
export function accountOfToken(token: string | null): string | null {
  const payload = token?.split(".")[1];
  if (!payload) return null;
  try {
    const sub = (JSON.parse(atob(payload.replace(/-/g, "+").replace(/_/g, "/"))) as { sub?: unknown }).sub;
    return typeof sub === "string" && sub !== "" ? sub : null;
  } catch {
    return null;
  }
}
