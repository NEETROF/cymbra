import { reactive, ref, toRef } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run, success } from "@/lib/async";
import { ScorePreviewError } from "@/lib/errors";
import type { CatalogHit } from "@/gen/score_pb";
import { useAuthStore } from "./auth";
import { soundfontBaseUrl } from "./soundfonts";

export type ModerationStatus = "pending" | "accepted" | "rejected";
/** The status filter's value in the BO catalog view: a specific status, or "" = all
 * statuses (the default "Tous"). */
export type StatusFilter = ModerationStatus | "";
/** The audio-teaser filter (change: add-score-daily-access-rewards): "" = any,
 * "yes" = pieces with a rendered sample, "no" = pieces WITHOUT one (the backfill view). */
export type PreviewFilter = "" | "yes" | "no";
/** The instrument-family filter (change: add-drums-access): "" = all instruments.
 * Replaces the retired piano checkbox (`is_piano` staff-count proxy). */
export type InstrumentFilter = "" | "keyboard" | "percussion";
export interface SortKeyInit {
  field: string;
  descending: boolean;
}

/** The filter bar's current selection (kept here so both the bar and views share it). */
export interface Filters {
  query: string;
  author: string;
  level: string;
  // Instrument family (change: add-drums-access): "" = all.
  instrument: InstrumentFilter;
  // "" = all statuses (Tous) — the BO catalog default (change: add-score-catalog-proposal).
  moderationStatus: StatusFilter;
  // Origin filter: "" = any source, else e.g. "user-proposal".
  source: string;
  // Audio-teaser filter (change: add-score-daily-access-rewards).
  hasPreview: PreviewFilter;
}

/** The catalog screen's live browse state (filters + sort + page). Lifted into the
 * store so it survives leaving for a score's detail page and coming back — the view
 * is remounted on return (no keep-alive), so local refs would reset to defaults. */
export interface CatalogView {
  filters: Filters;
  sort: SortKeyInit[];
  offset: number;
}

/** A fresh, unfiltered browse state: all statuses (Tous), any source, no sort, first
 * page — the BO catalog default (change: add-score-catalog-proposal). */
export function defaultCatalogView(): CatalogView {
  return {
    filters: {
      query: "",
      author: "",
      level: "",
      instrument: "",
      moderationStatus: "",
      source: "",
      hasPreview: "",
    },
    sort: [],
    offset: 0,
  };
}

// The hub filters (text/author/level/facets) plus the BO-only status filter and
// the structured sort. Only fields the console actually surfaces are modelled.
export interface SearchParams {
  query?: string;
  author?: string;
  level?: string;
  // Instrument family: "keyboard" | "percussion"; undefined = not constrained.
  // Narrows only — the server always constrains results by the caller's drum
  // eligibility, whatever filter is supplied (change: add-drums-access).
  instrument?: "keyboard" | "percussion";
  moderationStatus?: ModerationStatus;
  // Review-queue mode: the moderation work list = pending scores PLUS accepted
  // scores flagged for re-review by community ratings. Overrides moderationStatus.
  reviewQueue?: boolean;
  // BO catalog "Tous": return every moderation status (privileged). Overrides the
  // accepted-only default when no specific `moderationStatus` is set.
  allStatuses?: boolean;
  // Origin filter (e.g. "user-proposal"); undefined/"" = any source.
  source?: string;
  // Audio-teaser filter (privileged): true = rendered only, false = missing only.
  hasPreview?: boolean;
  sort?: SortKeyInit[];
  limit?: number;
  offset?: number;
}

export interface CatalogResult {
  hits: CatalogHit[];
  total: number;
  nextOffset: number;
}

/** Server page size — one screen of rows per SearchCatalog request. */
export const PAGE_SIZE = 50;

// The queue's default "review priority" ordering (design D5): flagged re-reviews
// first (now wired to #2's community ratings), then pending, then the most
// substantial scores. Used with reviewQueue mode so flagged accepted scores rank
// above the pending backlog.
export const QUEUE_SORT: SortKeyInit[] = [
  { field: "needs_review", descending: true },
  { field: "status_rank", descending: true },
  { field: "measure_count", descending: true },
  { field: "staff_count", descending: true },
];

export interface CatalogStats {
  total: number;
  accepted: number;
  pending: number;
  rejected: number;
}

// Module-scope data helpers (no store state — kept out of the setup, and flatter,
// per Sonar S7721/S2004). They call the API seam like everything else.
async function countByStatus(moderationStatus: ModerationStatus): Promise<number> {
  const resp = await api().score.searchCatalog({ query: "", moderationStatus, sort: [], limit: 1, offset: 0 });
  return resp.total;
}

async function fetchBytes(catalogId: string): Promise<Uint8Array> {
  const resp = await api().score.getCatalogScoreBytes({ catalogId });
  return resp.data;
}

/** One score's metadata by id — so the detail view is self-sufficient (works on
 * refresh / deep-link, not dependent on the last search's list). */
async function fetchHit(catalogId: string): Promise<CatalogHit> {
  return api().score.getCatalogScore({ catalogId });
}

/** Evaluate a score (raw RPC, no list refresh) — used by the review session, which
 * advances the deck itself instead of re-querying. `reason` is the moderator's
 * rejection motive (change: add-score-catalog-proposal); it is recorded on `rejected`
 * and cleared otherwise, and surfaced back to the proposer. */
async function evaluate(scoreId: string, status: ModerationStatus, reason?: string) {
  await api().score.setModerationStatus({ scoreId, status, reason });
}

/** A moderator's curatorial edit — only descriptive/attribution fields (never the
 * MusicXML-derived facets). Each field present is applied (an empty string clears
 * composer/arranger/level; an empty title is rejected server-side). */
export interface MetadataEdit {
  title?: string;
  composer?: string;
  arranger?: string;
  level?: string;
}

/** Persist a curatorial metadata edit (raw RPC behind the api() seam). The backend
 * recomputes the derived search keys and audits the change; the caller refreshes the
 * hit on success. Guarded server-side by `require_moderator_or_admin`. */
async function updateCatalogScore(scoreId: string, edit: MetadataEdit) {
  await api().score.updateCatalogScore({ scoreId, ...edit });
}

/** Fetch one page of the review queue (pending + community-flagged, priority-sorted)
 * WITHOUT touching `result` — the review session owns its own deck state. */
/** (Re)generates a catalog piece's server-rendered audio teaser via the admin HTTP
 *  route (change: add-score-daily-access-rewards). Injected in tests, like the
 *  SoundFont one. */
export type RegenerateScorePreviewFn = (id: string, token: string | null) => Promise<void>;

async function httpRegenerateScorePreview(id: string, token: string | null): Promise<void> {
  const resp = await fetch(`${soundfontBaseUrl()}/scores/${encodeURIComponent(id)}/preview`, {
    method: "POST",
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
  if (!resp.ok) {
    // Keep the body: the route answers 412 with the precise reason (which font
    // key is unset, which font is not accepted), and that is the only part an
    // admin can act on. `humanError` turns it into a localized message.
    throw new ScorePreviewError(resp.status, await resp.text().catch(() => ""));
  }
}

let regenerateScorePreviewImpl: RegenerateScorePreviewFn = httpRegenerateScorePreview;
export function setRegenerateScorePreviewForTest(fn: RegenerateScorePreviewFn): void {
  regenerateScorePreviewImpl = fn;
}

async function fetchReviewPage(offset: number): Promise<CatalogResult> {
  const resp = await api().score.searchCatalog({
    query: "",
    reviewQueue: true,
    sort: QUEUE_SORT,
    limit: PAGE_SIZE,
    offset,
  });
  return { hits: resp.hits, total: resp.total, nextOffset: resp.nextOffset };
}

export const useCatalogStore = defineStore("catalog", () => {
  // One value for the whole search lifecycle — views match on it exhaustively.
  const result = ref<Async<CatalogResult>>(idle);
  const lastParams = reactive<SearchParams>({ limit: PAGE_SIZE, offset: 0 });
  // The catalog screen's browse state, persisted across a detail-page round-trip
  // (the view remounts on return, so its filters/sort/page must live here).
  const catalogView = reactive<CatalogView>(defaultCatalogView());
  // Header stat cards. Kept in its own Async so a stats failure never blocks the
  // list — the cards just fall back to "—".
  const stats = ref<Async<CatalogStats>>(idle);
  // Per-row download state, keyed by catalog id, so each row shows its OWN
  // loading/error and one slow/failed download never blocks the rest of the table.
  // Only the loading/error signal lives here — the bytes are handed to the caller and
  // dropped on success (see `downloadBytes`), never pinned in the store.
  const downloads = reactive<Record<string, Async<Uint8Array>>>({});
  // "Generate sample" state (change: add-score-daily-access-rewards): its own
  // `Async` union so the button reflects in-flight/success/error without colliding
  // with the other mutations. `previewTarget` is the piece being (re)generated.
  const preview = ref<Async<void>>(idle);
  const previewTarget = ref<string | null>(null);

  async function search(params: SearchParams) {
    Object.assign(lastParams, { limit: PAGE_SIZE, offset: 0 }, params);
    await run(result, async () => {
      const resp = await api().score.searchCatalog({
        query: params.query ?? "",
        author: params.author,
        level: params.level,
        instrument: params.instrument,
        moderationStatus: params.moderationStatus,
        reviewQueue: params.reviewQueue,
        allStatuses: params.allStatuses,
        source: params.source || undefined,
        hasPreview: params.hasPreview,
        sort: params.sort ?? [],
        limit: params.limit ?? PAGE_SIZE,
        offset: params.offset ?? 0,
      });
      return { hits: resp.hits, total: resp.total, nextOffset: resp.nextOffset };
    });
  }

  /** Evaluate a score; on success re-run the last query so the row reflects it.
   * `reason` is the moderator's rejection motive (change: add-score-catalog-proposal). */
  async function setModerationStatus(scoreId: string, status: ModerationStatus, reason?: string) {
    await evaluate(scoreId, status, reason);
    await search(lastParams);
  }

  /** Fetch one score's decoded MusicXML bytes for a local download, tracking the
   * per-row `Async` state in `downloads`. Reuses `GetCatalogScoreBytes` (already gated
   * to moderator/admin server-side). Returns the bytes on success (which the caller
   * saves to disk) and `null` on failure — the row's error state carries a localized
   * message; a raw gRPC/exception string is never surfaced. Never throws. */
  async function downloadBytes(catalogId: string): Promise<Uint8Array | null> {
    const outcome = await run(toRef(downloads, catalogId), () => fetchBytes(catalogId));
    if (outcome.status === "success") {
      // The row only needed the loading/error signal; drop the bytes so we don't pin
      // every downloaded score in memory.
      delete downloads[catalogId];
      return outcome.data;
    }
    return null;
  }

  /** (Re)generate a piece's server-rendered audio teaser (change:
   *  add-score-daily-access-rewards). On success the route has already STORED the
   *  clip and stamped the row's marker, so the loaded row's `hasPreview` is flipped
   *  optimistically (its control turns into a play button); no re-list. */
  async function regenerateScorePreview(id: string) {
    previewTarget.value = id;
    const outcome = await run(preview, async () => {
      await regenerateScorePreviewImpl(id, useAuthStore().accessToken);
    });
    if (outcome.status === "success" && result.value.status === "success") {
      const target = result.value.data.hits.find((h) => h.id === id);
      if (target) target.hasPreview = true;
      result.value = success({ ...result.value.data, hits: [...result.value.data.hits] });
    }
    return outcome;
  }

  /** Bytes of a piece's audio teaser (`GET /scores/{id}/preview`), for the
   *  back-office play control — the same clip the app auditions on a locked piece. */
  async function scorePreviewClip(id: string): Promise<Uint8Array> {
    const token = useAuthStore().accessToken;
    const resp = await fetch(`${soundfontBaseUrl()}/scores/${encodeURIComponent(id)}/preview`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    if (!resp.ok) throw new Error(`score preview fetch failed: HTTP ${resp.status}`);
    return new Uint8Array(await resp.arrayBuffer());
  }

  /** Per-status counts for the header cards — three cheap count-only queries
   * (limit 1, we only read `total`) run in parallel. */
  async function loadStats() {
    await run(stats, async () => {
      const [pending, accepted, rejected] = await Promise.all([
        countByStatus("pending"),
        countByStatus("accepted"),
        countByStatus("rejected"),
      ]);
      return { pending, accepted, rejected, total: pending + accepted + rejected };
    });
  }

  return {
    result,
    lastParams,
    catalogView,
    stats,
    downloads,
    preview,
    previewTarget,
    search,
    regenerateScorePreview,
    scorePreviewClip,
    loadStats,
    evaluate,
    setModerationStatus,
    fetchReviewPage,
    fetchBytes,
    downloadBytes,
    fetchHit,
    updateCatalogScore,
  };
});
