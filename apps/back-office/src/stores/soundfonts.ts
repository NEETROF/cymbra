import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { humanError, SoundFontUploadError } from "@/lib/errors";
import type { AdminSoundFont, CatalogHit, SoundFont } from "@/gen/score_pb";
import { useAuthStore } from "./auth";

// SoundFont catalog administration (change: add-soundfont-back-office-management).
// The metadata list/edit/delete go through the ScoreService admin RPCs behind the
// `api()` seam; adding a font uploads its bytes through the admin HTTP route (bytes
// don't fit gRPC-web). Both the catalog and the last mutation are `Async` unions so
// the view matches on them — a denied/failed op lands in `op` as an error, not a throw.

/** A new font to add: its metadata plus the `.sf2` file to upload. */
export interface NewSoundFont {
  id: string;
  label: string;
  license: string;
  attribution: string;
  instrument: string;
  file: File;
}

/** Metadata edit for an existing font (id, bytes, and instrument are immutable). */
export interface SoundFontEdit {
  id: string;
  label: string;
  license: string;
  attribution: string;
}

/** Base origin of the SoundFont HTTP routes (delivery + upload) — the web-auth/API
 *  host, overridable per environment; trailing slashes trimmed. */
export function soundfontBaseUrl(): string {
  let base = (import.meta.env.VITE_WEB_AUTH_URL as string | undefined) ?? "http://localhost:8081";
  while (base.endsWith("/")) base = base.slice(0, -1);
  return base;
}

/** Uploads a `.sf2` + metadata to the admin route. Injected in tests so the store is
 *  driven without a real network. */
export type UploadFn = (font: NewSoundFont, token: string | null) => Promise<void>;

async function httpUpload(font: NewSoundFont, token: string | null): Promise<void> {
  const query = new URLSearchParams({
    label: font.label,
    license: font.license,
    attribution: font.attribution,
    instrument: font.instrument,
  });
  const resp = await fetch(`${soundfontBaseUrl()}/soundfonts/${encodeURIComponent(font.id)}?${query}`, {
    method: "POST",
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: font.file,
  });
  if (!resp.ok) {
    // Typed so humanError can map the status (e.g. 409 → "already exists").
    throw new SoundFontUploadError(resp.status);
  }
}

// The upload transport is a module-level seam so tests can replace it (the multipart
// route is not part of the gRPC `api()` client).
let uploadImpl: UploadFn = httpUpload;
export function setUploadForTest(fn: UploadFn): void {
  uploadImpl = fn;
}

export const useSoundFontsStore = defineStore("soundfonts", () => {
  const catalog = ref<Async<AdminSoundFont[]>>(idle);
  const op = ref<Async<void>>(idle);

  /** Load the admin catalog listing. */
  async function list() {
    await run(catalog, async () => (await api().score.adminListSoundFonts({})).soundfonts);
  }

  /** Add a font: upload its bytes + metadata, then re-list. */
  async function add(font: NewSoundFont) {
    const outcome = await run(op, async () => {
      await uploadImpl(font, useAuthStore().accessToken);
    });
    if (outcome.status === "success") await list();
    return outcome;
  }

  /** Edit a font's metadata, then re-list. */
  async function update(edit: SoundFontEdit) {
    const outcome = await run(op, async () => {
      await api().score.updateSoundFont(edit);
    });
    if (outcome.status === "success") await list();
    return outcome;
  }

  /** Remove a font (row + object), then re-list. */
  async function remove(id: string) {
    const outcome = await run(op, async () => {
      await api().score.deleteSoundFont({ id });
    });
    if (outcome.status === "success") await list();
    return outcome;
  }

  /** Set a font's moderation status (accept/reject/re-queue), then re-list
   *  (change: add-soundfont-moderation). */
  async function setModerationStatus(id: string, status: string) {
    const outcome = await run(op, async () => {
      await api().score.setSoundFontModerationStatus({ id, status });
    });
    if (outcome.status === "success") await list();
    return outcome;
  }

  /** The public catalog listing (id/label/…), for the preview font picker on the review
   *  and detail screens — usable by moderators, unlike the admin listing. */
  async function publicList(): Promise<SoundFont[]> {
    return (await api().score.listSoundFonts({})).soundfonts;
  }

  // --- Preview data (auditioning a font against a catalog piece) ---

  /** A handful of accepted catalog pieces to audition a font against. */
  async function previewPieces(query = ""): Promise<CatalogHit[]> {
    const resp = await api().score.searchCatalog({
      query,
      moderationStatus: "accepted",
      sort: [],
      limit: 20,
      offset: 0,
    });
    return resp.hits;
  }

  /** MusicXML bytes of a catalog piece, for the preview player. */
  async function pieceBytes(catalogId: string): Promise<Uint8Array> {
    return (await api().score.getCatalogScoreBytes({ catalogId })).data;
  }

  /** Bytes of an existing font from the delivery route (edit-mode preview). */
  async function fontBytes(id: string): Promise<Uint8Array> {
    const token = useAuthStore().accessToken;
    const resp = await fetch(`${soundfontBaseUrl()}/soundfonts/${encodeURIComponent(id)}`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    if (!resp.ok) throw new Error(`soundfont fetch failed: HTTP ${resp.status}`);
    return new Uint8Array(await resp.arrayBuffer());
  }

  return {
    catalog,
    op,
    list,
    publicList,
    add,
    update,
    remove,
    setModerationStatus,
    previewPieces,
    pieceBytes,
    fontBytes,
  };
});

// Re-export so the view can render an upload failure through the same mapper.
export { humanError };
