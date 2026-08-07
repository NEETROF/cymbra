// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The music module's gRPC **server** adapter: exposes `ScoreService` by
//! translating each RPC into a [`ScoreModule`] call. The caller's identity comes
//! from the internal-token interceptor (request extension), never the body, and
//! every op is scoped to that `owner_id`.

// tonic's `Status` makes `Result<_, Status>` large; unavoidable on the generated
// service signatures.
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use cymbra_platform::AuthIdentity;
use tonic::{Request, Response, Status};

use crate::catalog_edit::MetadataChanges;
use crate::catalog_limits::CatalogAccessLimiter;
use crate::catalog_search::{CatalogHit, CatalogQuery, SortKey, is_moderation_sort_field};
use crate::curation_rewards::{CuratorMetrics, LedgerEntry};
use crate::curation_rewards_core::BADGES;
use crate::curation_rewards_module::{CurationRewardsModule, CuratorRewards};
use crate::module::{ScoreModule, UploadInput};
use crate::proto::{
    AdminListSoundFontsRequest, AdminListSoundFontsResponse, AdminSoundFont,
    CatalogHit as ProtoCatalogHit, CuratorBadge, CuratorReliability,
    CuratorRewards as ProtoRewards, DeleteScoreRequest, DeleteScoreResponse,
    DeleteSoundFontRequest, DeleteSoundFontResponse, GetCatalogScoreBytesRequest,
    GetCatalogScoreBytesResponse, GetCatalogScoreRequest, GetCuratorReliabilityRequest,
    GetCuratorRewardsRequest, GetRatingPreviewBytesRequest, GetRatingPreviewBytesResponse,
    GetScoreBytesRequest, GetScoreBytesResponse, ListMyScoresRequest, ListMyScoresResponse,
    ListRatingDeckRequest, ListRatingDeckResponse, ListRewardShopRequest, ListRewardShopResponse,
    ListSavedCatalogScoresRequest, ListSavedCatalogScoresResponse, ListSoundFontsRequest,
    ListSoundFontsResponse, ProposeScoreRequest, ProposeScoreResponse, RedeemRewardRequest,
    RedeemRewardResponse, RemoveSavedCatalogScoreRequest, RemoveSavedCatalogScoreResponse,
    RewardActivity, RewardShopItem, SaveCatalogScoreRequest, SaveCatalogScoreResponse, ScoreRecord,
    SearchCatalogRequest, SearchCatalogResponse, SetModerationStatusRequest,
    SetModerationStatusResponse, SetScoreFavoriteRequest, SetScoreFavoriteResponse,
    SetSoundFontModerationStatusRequest, SetSoundFontModerationStatusResponse,
    SoundFont as ProtoSoundFont, SubmitScoreRatingRequest, SubmitScoreRatingResponse,
    UpdateCatalogScoreRequest, UpdateCatalogScoreResponse, UpdateSoundFontRequest,
    UpdateSoundFontResponse, UploadScoreRequest,
    score_service_server::{ScoreService, ScoreServiceServer},
};
use crate::soundfont::SoundFontRepo;
use crate::user_scores::UserScore;
use cymbra_storage::ObjectStorage;

/// Wraps the score module as a tonic `ScoreService`.
pub struct ScoreGrpc {
    module: Arc<ScoreModule>,
    /// Per-user catalog access guardrail (change: add-catalog-access-limits). `None`
    /// leaves the service un-limited (used by unit tests); production wires it via
    /// [`Self::with_limiter`].
    limiter: Option<Arc<CatalogAccessLimiter>>,
    /// Persisted SoundFont catalog (change: add-soundfont-catalog-db). `None` leaves
    /// `ListSoundFonts` reporting the feature as unavailable; production wires it via
    /// [`Self::with_soundfonts`].
    soundfonts: Option<Arc<dyn SoundFontRepo>>,
    /// Private SoundFont object store (change: add-soundfont-back-office-management),
    /// so `DeleteSoundFont` can remove the object alongside the row and the admin list
    /// can report object presence. `None` when the store is unconfigured.
    soundfont_store: Option<Arc<dyn ObjectStorage>>,
    /// Curation-rewards service (change: add-curation-rewards). `None` leaves the
    /// reward RPCs reporting the feature as unavailable; production wires it via
    /// [`Self::with_rewards`].
    rewards: Option<Arc<CurationRewardsModule>>,
    /// User directory seam (change: add-soundfont-uploader-attribution): resolves a
    /// font's `uploaded_by` to the uploader pseudo (privileged read) and the opt-in
    /// public contributor credit. `None` (tests without attribution) simply leaves
    /// both absent.
    user: Option<Arc<dyn cymbra_user_port::UserPort>>,
}

impl ScoreGrpc {
    pub fn new(module: Arc<ScoreModule>) -> Self {
        Self {
            module,
            limiter: None,
            soundfonts: None,
            soundfont_store: None,
            rewards: None,
            user: None,
        }
    }

    /// Attach the user directory that resolves soundfont uploader attribution
    /// (change: add-soundfont-uploader-attribution).
    pub fn with_user_port(mut self, user: Arc<dyn cymbra_user_port::UserPort>) -> Self {
        self.user = Some(user);
        self
    }

    /// Attach the curation-rewards service that backs the reward RPCs.
    pub fn with_rewards(mut self, rewards: Arc<CurationRewardsModule>) -> Self {
        self.rewards = Some(rewards);
        self
    }

    /// The wired rewards service, or `UNAVAILABLE` when unconfigured.
    fn rewards(&self) -> Result<&Arc<CurationRewardsModule>, Status> {
        self.rewards
            .as_ref()
            .ok_or_else(|| Status::unavailable("curation rewards unavailable"))
    }

    /// Attach the per-user access limiter that guards browse/search/download egress.
    pub fn with_limiter(mut self, limiter: Arc<CatalogAccessLimiter>) -> Self {
        self.limiter = Some(limiter);
        self
    }

    /// Attach the persisted SoundFont catalog that backs `ListSoundFonts` and the
    /// admin catalog RPCs.
    pub fn with_soundfonts(mut self, soundfonts: Arc<dyn SoundFontRepo>) -> Self {
        self.soundfonts = Some(soundfonts);
        self
    }

    /// Attach the private SoundFont object store so `DeleteSoundFont` removes the
    /// object with the row and the admin list can report object presence.
    pub fn with_soundfont_store(mut self, store: Arc<dyn ObjectStorage>) -> Self {
        self.soundfont_store = Some(store);
        self
    }

    /// Like [`Self::with_soundfont_store`] but takes an already-optional store (the
    /// store is `None` when unconfigured at the call site).
    pub fn with_soundfont_store_opt(mut self, store: Option<Arc<dyn ObjectStorage>>) -> Self {
        self.soundfont_store = store;
        self
    }

    /// The wired SoundFont catalog, or `UNAVAILABLE` when unconfigured.
    fn soundfont_repo(&self) -> Result<&Arc<dyn SoundFontRepo>, Status> {
        self.soundfonts
            .as_ref()
            .ok_or_else(|| Status::unavailable("soundfont catalog unavailable"))
    }

    /// Mountable tonic server.
    pub fn into_server(self) -> ScoreServiceServer<Self> {
        ScoreServiceServer::new(self)
    }

    /// Enforce the download guardrail (burst + play-aware volume) when a limiter is
    /// wired; a breach surfaces as `RESOURCE_EXHAUSTED` before any storage read.
    async fn guard_download(&self, id: &AuthIdentity) -> Result<(), Status> {
        if let Some(l) = &self.limiter {
            l.check_download(id).await?;
        }
        Ok(())
    }

    /// Enforce the enumeration (browse/search/deck) request-rate cap when wired.
    async fn guard_enumeration(&self, id: &AuthIdentity) -> Result<(), Status> {
        if let Some(l) = &self.limiter {
            l.check_enumeration(id).await?;
        }
        Ok(())
    }
}

fn owner<T>(req: &Request<T>) -> Result<String, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .map(|id| id.user_id.clone())
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

/// The full authenticated identity (for role checks). Same unauthenticated guard
/// as [`owner`]; cloned so it outlives `req.into_inner()`.
fn identity<T>(req: &Request<T>) -> Result<AuthIdentity, Status> {
    req.extensions()
        .get::<AuthIdentity>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("missing identity"))
}

fn to_record(s: UserScore) -> ScoreRecord {
    to_record_with_proposal(s, None, None)
}

/// Build a [`ScoreRecord`] with the proposal state joined from the linked catalog row
/// (change: add-score-catalog-proposal): `status` = not-proposed / pending / accepted /
/// rejected, `reason` = the moderator's motive for a rejected proposal.
fn to_record_with_proposal(
    s: UserScore,
    status: Option<String>,
    reason: Option<String>,
) -> ScoreRecord {
    let m = s.meta;
    ScoreRecord {
        id: s.id,
        title: m.title,
        composer: m.composer,
        level: s.level,
        created_at: s.created_at,
        measure_count: m.measure_count,
        time_sig: m.time_sig,
        key_fifths: m.key_fifths,
        min_note_value: m.facets.min_note_value.map(i32::from),
        tempo_bpm: m.facets.tempo_bpm.map(i32::from),
        note_count: m.facets.note_count as i32,
        lowest_midi: m.facets.lowest_midi.map(i32::from),
        highest_midi: m.facets.highest_midi.map(i32::from),
        favorite: s.favorite,
        proposal_status: status,
        rejection_reason: reason,
    }
}

/// Current UTC date, for the public-profile age gate when resolving a contributor
/// credit (change: add-score-catalog-proposal).
fn today_utc() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

fn to_hit(h: CatalogHit) -> ProtoCatalogHit {
    ProtoCatalogHit {
        id: h.id,
        title: h.title,
        composer: h.composer,
        level: h.level,
        license: h.license,
        source: h.source,
        arranger: h.arranger,
        min_note_value: h.min_note_value,
        tempo_bpm: h.tempo_bpm,
        note_count: h.note_count,
        lowest_midi: h.lowest_midi,
        highest_midi: h.highest_midi,
        time_sig: h.time_sig,
        key_fifths: h.key_fifths,
        needs_review: h.needs_review,
        moderation_status: h.moderation_status,
        proposed_by: h.proposed_by,
        proposer_display_name: h.proposer_display_name,
        contributor_credit: h.contributor_credit,
        review_reason: h.review_reason,
        resubmission_note: h.resubmission_note,
    }
}

/// Map the domain curator standing to the wire type (change: add-curation-rewards),
/// building the FULL badge grid (earned + locked with milestone hints) from [`BADGES`].
fn to_proto_rewards(r: CuratorRewards) -> ProtoRewards {
    let earned: std::collections::HashSet<&str> =
        r.earned_badges.iter().map(String::as_str).collect();
    let badges = BADGES
        .iter()
        .map(|b| CuratorBadge {
            key: b.key.to_string(),
            metric: b.metric.as_str().to_string(),
            threshold: b.threshold,
            earned: earned.contains(b.key),
        })
        .collect();
    ProtoRewards {
        lifetime_points: r.lifetime_points,
        spendable_balance: r.spendable_balance,
        level: r.level,
        level_floor: r.level_floor,
        next_level_at: r.next_level_at,
        total_ratings: r.metrics.total_ratings,
        coverage_contribution: r.metrics.coverage_contribution,
        alignment_rate: r.metrics.alignment_rate(),
        badges,
        recent: r.recent.into_iter().map(to_proto_activity).collect(),
    }
}

/// Map one ledger entry to the wire activity feed row.
fn to_proto_activity(e: LedgerEntry) -> RewardActivity {
    RewardActivity {
        kind: e.kind.as_str().to_string(),
        amount: e.amount.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        catalog_id: e.catalog_score_id,
        reward_key: e.reward_key,
        source: e.source.map(|s| s.as_str().to_string()),
        created_at: e.created_at_ms,
    }
}

/// Map one reward-shop item to the wire type.
fn to_proto_shop_item(i: crate::curation_rewards::ShopItem) -> RewardShopItem {
    RewardShopItem {
        key: i.key,
        label: i.label,
        instrument: i.instrument,
        license: i.license,
        attribution: i.attribution.unwrap_or_default(),
        point_cost: i.point_cost.clamp(0, i32::MAX as i64) as i32,
        redeemable: i.redeemable,
        owned: i.owned,
    }
}

/// Map curator metrics to the back-office reliability wire type.
fn to_proto_reliability(m: CuratorMetrics) -> CuratorReliability {
    CuratorReliability {
        total_ratings: m.total_ratings,
        coverage_contribution: m.coverage_contribution,
        alignment_rate: m.alignment_rate(),
        settled_count: m.settled_count,
        aligned_count: m.aligned_count,
    }
}

#[tonic::async_trait]
impl ScoreService for ScoreGrpc {
    async fn upload_score(
        &self,
        req: Request<UploadScoreRequest>,
    ) -> Result<Response<ScoreRecord>, Status> {
        let owner_id = owner(&req)?;
        let r = req.into_inner();
        let rec = self
            .module
            .upload(
                &owner_id,
                UploadInput {
                    data: r.data,
                    filename: r.filename,
                    level: r.level,
                    rights_basis: r.rights_basis,
                    rights_ack: r.rights_ack,
                    fallback_title: r.fallback_title,
                    fallback_composer: r.fallback_composer,
                },
            )
            .await?;
        Ok(Response::new(to_record(rec)))
    }

    async fn list_my_scores(
        &self,
        req: Request<ListMyScoresRequest>,
    ) -> Result<Response<ListMyScoresResponse>, Status> {
        let owner_id = owner(&req)?;
        let scores = self.module.list_contributions(&owner_id).await?;
        Ok(Response::new(ListMyScoresResponse {
            scores: scores
                .into_iter()
                .map(|(s, status, reason)| to_record_with_proposal(s, status, reason))
                .collect(),
        }))
    }

    async fn propose_score(
        &self,
        req: Request<ProposeScoreRequest>,
    ) -> Result<Response<ProposeScoreResponse>, Status> {
        let id = identity(&req)?;
        let owner_id = id.user_id.clone();
        // The initial catalog status branches on the caller's role, never the body: a
        // music-scope admin proposal is auto-accepted, everyone else is pending.
        let is_admin = id.has_role_in_scope("music", "admin");
        let r = req.into_inner();
        self.module
            .propose(
                &owner_id,
                &r.score_id,
                &r.license,
                r.rights_ack,
                r.resubmission_note.as_deref(),
                is_admin,
            )
            .await?;
        Ok(Response::new(ProposeScoreResponse {}))
    }

    async fn delete_score(
        &self,
        req: Request<DeleteScoreRequest>,
    ) -> Result<Response<DeleteScoreResponse>, Status> {
        let owner_id = owner(&req)?;
        let id = req.into_inner().id;
        self.module.delete(&owner_id, &id).await?;
        Ok(Response::new(DeleteScoreResponse {}))
    }

    async fn get_score_bytes(
        &self,
        req: Request<GetScoreBytesRequest>,
    ) -> Result<Response<GetScoreBytesResponse>, Status> {
        let owner_id = owner(&req)?;
        let id = req.into_inner().id;
        let data = self.module.get_bytes(&owner_id, &id).await?;
        Ok(Response::new(GetScoreBytesResponse { data }))
    }

    /// Lists the server-owned SoundFont catalog (change: add-soundfont-catalog-db)
    /// so the client offers only fonts that actually exist. Authenticated; the bytes
    /// are fetched separately via the `GET /soundfonts/{id}` delivery route.
    async fn list_sound_fonts(
        &self,
        req: Request<ListSoundFontsRequest>,
    ) -> Result<Response<ListSoundFontsResponse>, Status> {
        identity(&req)?; // authenticated-only; the public catalog is the same for everyone
        let repo = self.soundfont_repo()?;
        // Public listing: only validated (`accepted`) fonts are offered. Unvalidated
        // fonts are visible only through the admin listing (change: add-soundfont-moderation).
        let fonts = repo
            .list_accepted()
            .await
            .map_err(|e| Status::internal(format!("list soundfonts: {e}")))?;
        // Opt-in public contributor credit (change: add-soundfont-uploader-attribution):
        // resolve every uploader id in ONE batched, fail-closed directory call — only a
        // `Public`, age-eligible profile comes back, so a private/unresolvable/seeded
        // uploader simply yields no credit. The raw `uploaded_by` id never leaves here.
        let mut credits: std::collections::HashMap<String, String> = Default::default();
        if let Some(user) = &self.user {
            let ids: Vec<String> = {
                let mut ids: Vec<String> =
                    fonts.iter().filter_map(|f| f.uploaded_by.clone()).collect();
                ids.sort();
                ids.dedup();
                ids
            };
            if !ids.is_empty()
                && let Ok(profiles) = user.listable_profiles(&ids, today_utc()).await
            {
                for p in profiles {
                    if let Some(name) = p.handle.or(p.display_name) {
                        credits.insert(p.user_id, name);
                    }
                }
            }
        }
        // `has_preview` lets the app grey a locked font's play control up front when no
        // preview clip exists yet (change: add-soundfont-entitlement-previews).
        let mut soundfonts = Vec::with_capacity(fonts.len());
        for f in fonts {
            let has_preview = match &self.soundfont_store {
                Some(s) => s
                    .size(&crate::soundfont_preview::preview_object_key(&f.id))
                    .await
                    .is_ok(),
                None => false,
            };
            let contributor_credit = f
                .uploaded_by
                .as_deref()
                .and_then(|id| credits.get(id).cloned())
                .unwrap_or_default();
            soundfonts.push(ProtoSoundFont {
                id: f.id,
                label: f.label,
                license: f.license,
                attribution: f.attribution.unwrap_or_default(),
                instrument: f.instrument,
                has_preview,
                contributor_credit,
            });
        }
        Ok(Response::new(ListSoundFontsResponse { soundfonts }))
    }

    /// Admin list of the SoundFont catalog (change:
    /// add-soundfont-back-office-management). Music-scope moderator/admin only; carries
    /// the storage-facing fields the user listing omits and reports object presence.
    async fn admin_list_sound_fonts(
        &self,
        req: Request<AdminListSoundFontsRequest>,
    ) -> Result<Response<AdminListSoundFontsResponse>, Status> {
        cymbra_platform::guard::require_moderator_or_admin(&identity(&req)?)?;
        let repo = self.soundfont_repo()?;
        let r = req.into_inner();
        // Paging (change: add-soundfont-moderation): clamp the page size, treat an
        // empty status filter as "all".
        const DEFAULT_LIMIT: i32 = 50;
        const MAX_LIMIT: i32 = 200;
        let limit = if r.limit <= 0 {
            DEFAULT_LIMIT
        } else {
            r.limit.min(MAX_LIMIT)
        };
        let offset = r.offset.max(0);
        let status = (!r.moderation_status.is_empty()).then_some(r.moderation_status.as_str());
        let (fonts, total) = repo
            .list_admin_page(status, limit as i64, offset as i64)
            .await
            .map_err(|e| Status::internal(format!("list soundfonts: {e}")))?;
        // Catalog-wide counts for the KPI cards (independent of the filter/page).
        let counts = repo
            .status_counts()
            .await
            .map_err(|e| Status::internal(format!("soundfont counts: {e}")))?;
        let page_len = fonts.len() as i32;
        // Privileged uploader attribution (change: add-soundfont-uploader-attribution):
        // resolve each page's `uploaded_by` ids (deduped) to the contributor's pseudo,
        // unconditionally (audit surface — independent of profile visibility). A seeded
        // font has no uploader and simply carries no pseudo; an unresolvable id degrades
        // to none rather than failing the listing.
        let mut pseudos: std::collections::HashMap<String, String> = Default::default();
        if let Some(user) = &self.user {
            let ids: Vec<&String> = {
                let mut ids: Vec<&String> = fonts
                    .iter()
                    .filter_map(|f| f.uploaded_by.as_ref())
                    .collect();
                ids.sort();
                ids.dedup();
                ids
            };
            for id in ids {
                if let Ok(acct) = user.get_account(id).await
                    && let Some(name) = acct.handle.or(acct.display_name)
                {
                    pseudos.insert(id.clone(), name);
                }
            }
        }
        let mut soundfonts = Vec::with_capacity(fonts.len());
        for f in fonts {
            let (has_object, has_preview) = match &self.soundfont_store {
                Some(s) => (
                    s.size(&f.object_key).await.is_ok(),
                    s.size(&crate::soundfont_preview::preview_object_key(&f.id))
                        .await
                        .is_ok(),
                ),
                None => (false, false),
            };
            let uploader_display_name = f
                .uploaded_by
                .as_deref()
                .and_then(|id| pseudos.get(id).cloned())
                .unwrap_or_default();
            soundfonts.push(AdminSoundFont {
                id: f.id,
                label: f.label,
                object_key: f.object_key,
                instrument: f.instrument,
                license: f.license,
                attribution: f.attribution.unwrap_or_default(),
                size_bytes: f.size_bytes.unwrap_or(0),
                has_object,
                moderation_status: f.moderation_status,
                reviewed_by: f.reviewed_by.unwrap_or_default(),
                reviewed_at: f.reviewed_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
                uploaded_by: f.uploaded_by.unwrap_or_default(),
                content_sha256: f.content_sha256.unwrap_or_default(),
                has_preview,
                uploader_display_name,
                resubmission_note: f.resubmission_note.unwrap_or_default(),
            });
        }
        Ok(Response::new(AdminListSoundFontsResponse {
            soundfonts,
            next_offset: offset + page_len,
            total: total as i32,
            pending_count: counts.pending as i32,
            accepted_count: counts.accepted as i32,
            rejected_count: counts.rejected as i32,
            total_count: counts.total as i32,
        }))
    }

    /// Edit a font's metadata (music-scope moderator/admin only). Id and object_key
    /// are immutable; bytes are replaced by re-upload through the HTTP route.
    async fn update_sound_font(
        &self,
        req: Request<UpdateSoundFontRequest>,
    ) -> Result<Response<UpdateSoundFontResponse>, Status> {
        cymbra_platform::guard::require_moderator_or_admin(&identity(&req)?)?;
        let repo = self.soundfont_repo()?;
        let r = req.into_inner();
        let attribution = (!r.attribution.is_empty()).then_some(r.attribution.as_str());
        let updated = repo
            .update_meta(&r.id, &r.label, &r.license, attribution)
            .await
            .map_err(|e| Status::internal(format!("update soundfont: {e}")))?;
        if !updated {
            return Err(Status::not_found("soundfont not found"));
        }
        Ok(Response::new(UpdateSoundFontResponse {}))
    }

    /// Remove a font (music-scope moderator/admin only): delete the catalog row, then
    /// best-effort delete the stored object. The row is removed even if object deletion
    /// fails, so the font immediately stops being offered.
    async fn delete_sound_font(
        &self,
        req: Request<DeleteSoundFontRequest>,
    ) -> Result<Response<DeleteSoundFontResponse>, Status> {
        cymbra_platform::guard::require_moderator_or_admin(&identity(&req)?)?;
        let repo = self.soundfont_repo()?;
        let font_id = req.into_inner().id;
        let entry = repo
            .lookup(&font_id)
            .await
            .map_err(|e| Status::internal(format!("lookup soundfont: {e}")))?;
        let Some(entry) = entry else {
            return Err(Status::not_found("soundfont not found"));
        };
        repo.delete(&font_id)
            .await
            .map_err(|e| Status::internal(format!("delete soundfont: {e}")))?;
        if let Some(store) = &self.soundfont_store
            && let Err(e) = store.delete(&entry.object_key).await
        {
            tracing::warn!("soundfont object {} not deleted: {e}", entry.object_key);
        }
        Ok(Response::new(DeleteSoundFontResponse {}))
    }

    /// Set a font's moderation status (change: add-soundfont-moderation). Music-scope
    /// moderator/admin only; the reviewer is the authenticated caller (never the body)
    /// and is stamped as `reviewed_by` alongside the status.
    async fn set_sound_font_moderation_status(
        &self,
        req: Request<SetSoundFontModerationStatusRequest>,
    ) -> Result<Response<SetSoundFontModerationStatusResponse>, Status> {
        let id = identity(&req)?;
        cymbra_platform::guard::require_moderator_or_admin(&id)?;
        let r = req.into_inner();
        if !matches!(r.status.as_str(), "pending" | "accepted" | "rejected") {
            return Err(Status::invalid_argument(
                "status must be pending, accepted, or rejected",
            ));
        }
        let repo = self.soundfont_repo()?;
        // Accepting a font publishes it as publicly auditionable, so a preview clip is
        // MANDATORY (change: add-soundfont-entitlement-previews): the moderator generates
        // it ("Generate sample"), auditions it, then accepts. Refuse acceptance until the
        // preview object exists. Only the `accepted` transition is gated (pending/rejected
        // need none); an unknown id still resolves to NotFound via `set_moderation_status`.
        if r.status == "accepted"
            && repo
                .lookup(&r.id)
                .await
                .map_err(|e| Status::internal(format!("lookup soundfont: {e}")))?
                .is_some()
        {
            let key = crate::soundfont_preview::preview_object_key(&r.id);
            let has_preview = match &self.soundfont_store {
                Some(store) => store.size(&key).await.is_ok(),
                None => false,
            };
            if !has_preview {
                return Err(Status::failed_precondition(
                    "a preview sample must be generated before accepting this soundfont",
                ));
            }
        }
        // The reason is the rejection motive (change: add-soundfont-uploader-
        // attribution): keep it only on `rejected`, so accepting or re-queuing clears
        // any stale reason (mirrors the score path).
        let reason = r
            .reason
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty() && r.status == "rejected");
        let matched = repo
            .set_moderation_status(&r.id, &r.status, &id.user_id, reason)
            .await
            .map_err(|e| Status::internal(format!("set soundfont moderation status: {e}")))?;
        if !matched {
            return Err(Status::not_found("soundfont not found"));
        }
        Ok(Response::new(SetSoundFontModerationStatusResponse {}))
    }

    async fn set_score_favorite(
        &self,
        req: Request<SetScoreFavoriteRequest>,
    ) -> Result<Response<SetScoreFavoriteResponse>, Status> {
        let owner_id = owner(&req)?;
        let r = req.into_inner();
        self.module
            .set_favorite(&owner_id, &r.id, r.favorite)
            .await?;
        Ok(Response::new(SetScoreFavoriteResponse {}))
    }

    // --- Score Hub (change: score-hub-search) -------------------------------
    // Every handler asserts an authenticated identity via `owner()` (the strict
    // interceptor already rejects unauthenticated calls; this is defense-in-depth
    // and the identity source for the owner-scoped ops). Search + bytes are
    // catalog-wide, so they require identity but do not scope by owner.

    async fn search_catalog(
        &self,
        req: Request<SearchCatalogRequest>,
    ) -> Result<Response<SearchCatalogResponse>, Status> {
        let id = identity(&req)?; // authenticated-only (catalog is public, not owner-scoped)
        self.guard_enumeration(&id).await?; // per-user browse cap (scrape guard)
        // The moderation-status filter and any moderation-oriented sort key are
        // privileged and back-office-only: when the caller uses either, they MUST be
        // authorised — reject with PERMISSION_DENIED and run no query otherwise.
        // #1 restricted this to `admin`; this change (add-moderation-back-office)
        // widens it to admin-or-(music) moderator.
        let uses_moderation = req.get_ref().moderation_status.is_some()
            || req.get_ref().review_queue.unwrap_or(false)
            || req.get_ref().all_statuses.unwrap_or(false)
            || req
                .get_ref()
                .sort
                .iter()
                .any(|k| is_moderation_sort_field(&k.field));
        if uses_moderation {
            cymbra_platform::guard::require_moderator_or_admin(&id)?;
        }
        let r = req.into_inner();
        let offset = r.offset;
        let query = CatalogQuery {
            query: r.query,
            author: r.author,
            level: r.level,
            facets: crate::catalog_search::FacetFilters {
                is_piano: r.is_piano,
                max_note_value: r.max_note_value.map(|v| v.clamp(0, i16::MAX as i32) as i16),
                has_chords: r.has_chords,
                has_tuplets: r.has_tuplets,
                has_dotted: r.has_dotted,
                max_ambitus_semitones: r
                    .max_ambitus_semitones
                    .map(|v| v.clamp(0, i16::MAX as i32) as i16),
                staff_count: r.staff_count.map(|v| v.clamp(0, i16::MAX as i32) as i16),
                min_bpm: r.min_bpm,
                max_bpm: r.max_bpm,
            },
            moderation_status: r.moderation_status,
            review_queue: r.review_queue.unwrap_or(false),
            all_statuses: r.all_statuses.unwrap_or(false),
            source: r.source,
            sort: r
                .sort
                .into_iter()
                .map(|k| SortKey {
                    field: k.field,
                    descending: k.descending,
                })
                .collect(),
            limit: r.limit as i64,
            offset: r.offset as i64,
        };
        let privileged = uses_moderation || query.moderation_status.is_some() || query.review_queue;
        let (mut hits, total) = self.module.search_catalog(query).await?;
        // Proposer attribution (change: add-score-catalog-proposal): a privileged read
        // resolves the proposer pseudo for the review queue; a normal read is sanitised
        // (privileged fields stripped) and gets only the opt-in public credit.
        if privileged {
            self.module.attach_review_attribution(&mut hits).await;
        } else {
            self.module
                .attach_public_credit(&mut hits, today_utc())
                .await;
        }
        let next_offset = offset.max(0) + hits.len() as i32;
        Ok(Response::new(SearchCatalogResponse {
            hits: hits.into_iter().map(to_hit).collect(),
            next_offset,
            total: total.clamp(0, i32::MAX as i64) as i32,
        }))
    }

    async fn save_catalog_score(
        &self,
        req: Request<SaveCatalogScoreRequest>,
    ) -> Result<Response<SaveCatalogScoreResponse>, Status> {
        let owner_id = owner(&req)?;
        let catalog_id = req.into_inner().catalog_id;
        self.module
            .save_catalog_score(&owner_id, &catalog_id)
            .await?;
        Ok(Response::new(SaveCatalogScoreResponse {}))
    }

    async fn remove_saved_catalog_score(
        &self,
        req: Request<RemoveSavedCatalogScoreRequest>,
    ) -> Result<Response<RemoveSavedCatalogScoreResponse>, Status> {
        let owner_id = owner(&req)?;
        let catalog_id = req.into_inner().catalog_id;
        self.module
            .remove_saved_catalog_score(&owner_id, &catalog_id)
            .await?;
        Ok(Response::new(RemoveSavedCatalogScoreResponse {}))
    }

    async fn list_saved_catalog_scores(
        &self,
        req: Request<ListSavedCatalogScoresRequest>,
    ) -> Result<Response<ListSavedCatalogScoresResponse>, Status> {
        let owner_id = owner(&req)?;
        let mut hits = self.module.list_saved_catalog_scores(&owner_id).await?;
        // Public read: sanitise privileged proposer fields, add any opt-in credit.
        self.module
            .attach_public_credit(&mut hits, today_utc())
            .await;
        Ok(Response::new(ListSavedCatalogScoresResponse {
            hits: hits.into_iter().map(to_hit).collect(),
        }))
    }

    async fn get_catalog_score_bytes(
        &self,
        req: Request<GetCatalogScoreBytesRequest>,
    ) -> Result<Response<GetCatalogScoreBytesResponse>, Status> {
        // Authenticated-only. A moderator/admin may open a score in any moderation
        // status (to review it); a normal caller is served only `accepted` bytes and
        // gets not-found otherwise (change: add-score-moderation-gating, widened to
        // moderator by add-moderation-back-office).
        let id = identity(&req)?;
        self.guard_download(&id).await?; // per-user download guardrail (scrape guard)
        let allow_unvalidated = id.is_admin() || id.has_role("moderator");
        let catalog_id = req.into_inner().catalog_id;
        let data = self
            .module
            .get_catalog_bytes(&catalog_id, allow_unvalidated)
            .await?;
        Ok(Response::new(GetCatalogScoreBytesResponse { data }))
    }

    async fn get_rating_preview_bytes(
        &self,
        req: Request<GetRatingPreviewBytesRequest>,
    ) -> Result<Response<GetRatingPreviewBytesResponse>, Status> {
        // Authenticated-only (any signed-in rater). Serves a `pending` or `accepted`
        // score's bytes for the deck's read-only preview; a `rejected`/unknown id is
        // not-found. The player-open bytes path and library save stay accepted-only
        // (change: rate-pending-scores).
        let id = identity(&req)?;
        self.guard_download(&id).await?; // shares the per-user download guardrail
        let catalog_id = req.into_inner().catalog_id;
        let data = self
            .module
            .rating_preview_bytes(&id.user_id, &catalog_id)
            .await?;
        Ok(Response::new(GetRatingPreviewBytesResponse { data }))
    }

    async fn get_curator_rewards(
        &self,
        req: Request<GetCuratorRewardsRequest>,
    ) -> Result<Response<ProtoRewards>, Status> {
        // The signed-in curator's own standing (change: add-curation-rewards).
        let user_id = owner(&req)?;
        let rewards = self.rewards()?.get_rewards(&user_id).await?;
        Ok(Response::new(to_proto_rewards(rewards)))
    }

    async fn list_reward_shop(
        &self,
        req: Request<ListRewardShopRequest>,
    ) -> Result<Response<ListRewardShopResponse>, Status> {
        let user_id = owner(&req)?;
        let items = self.rewards()?.list_shop(&user_id).await?;
        Ok(Response::new(ListRewardShopResponse {
            items: items.into_iter().map(to_proto_shop_item).collect(),
        }))
    }

    async fn redeem_reward(
        &self,
        req: Request<RedeemRewardRequest>,
    ) -> Result<Response<RedeemRewardResponse>, Status> {
        let user_id = owner(&req)?;
        let key = req.into_inner().reward_key;
        let res = self.rewards()?.redeem(&user_id, &key).await?;
        Ok(Response::new(RedeemRewardResponse {
            owned: res.owned,
            new_balance: res.new_balance,
        }))
    }

    async fn get_curator_reliability(
        &self,
        req: Request<GetCuratorReliabilityRequest>,
    ) -> Result<Response<CuratorReliability>, Status> {
        // Read-only per-user reliability indicator (design D7): MODERATOR/ADMIN only,
        // informs manual promotion — never auto-promotes. A non-moderator/admin caller
        // is refused before any read.
        let id = identity(&req)?;
        cymbra_platform::guard::require_moderator_or_admin(&id)?;
        let user_id = req.into_inner().user_id;
        let m = self.rewards()?.metrics(&user_id).await?;
        Ok(Response::new(to_proto_reliability(m)))
    }

    async fn get_catalog_score(
        &self,
        req: Request<GetCatalogScoreRequest>,
    ) -> Result<Response<ProtoCatalogHit>, Status> {
        // Same gate as fetch-bytes: a moderator/admin resolves any status; a normal
        // caller only `accepted` (non-`accepted` id → not-found). Lets a detail view
        // load by id without depending on a prior list.
        let id = identity(&req)?;
        self.guard_enumeration(&id).await?; // per-user browse cap (scrape guard)
        let privileged = id.is_admin() || id.has_role("moderator");
        let catalog_id = req.into_inner().catalog_id;
        let hit = self.module.get_catalog_hit(&catalog_id, privileged).await?;
        let mut hits = vec![hit];
        if privileged {
            self.module.attach_review_attribution(&mut hits).await;
        } else {
            self.module
                .attach_public_credit(&mut hits, today_utc())
                .await;
        }
        Ok(Response::new(to_hit(hits.pop().expect("one hit"))))
    }

    async fn set_moderation_status(
        &self,
        req: Request<SetModerationStatusRequest>,
    ) -> Result<Response<SetModerationStatusResponse>, Status> {
        // Evaluate action (change: add-moderation-back-office): moderator/admin only.
        // The reviewer id is the authenticated caller — never the body — and is
        // stamped as `reviewed_by` alongside the status.
        let id = identity(&req)?;
        cymbra_platform::guard::require_moderator_or_admin(&id)?;
        let r = req.into_inner();
        self.module
            .set_moderation_status(&id.user_id, &r.score_id, &r.status, r.reason.as_deref())
            .await?;
        Ok(Response::new(SetModerationStatusResponse {}))
    }

    async fn update_catalog_score(
        &self,
        req: Request<UpdateCatalogScoreRequest>,
    ) -> Result<Response<UpdateCatalogScoreResponse>, Status> {
        // Curatorial metadata edit (change: add-catalog-metadata-editing): moderator/
        // admin only. The editor is the authenticated caller, never the body; only the
        // descriptive fields are editable (the request shape can't carry derived facts).
        let id = identity(&req)?;
        cymbra_platform::guard::require_moderator_or_admin(&id)?;
        let r = req.into_inner();
        let changes = MetadataChanges {
            title: r.title,
            composer: r.composer,
            arranger: r.arranger,
            level: r.level,
        };
        self.module
            .update_catalog_score(&id.user_id, &r.score_id, changes)
            .await?;
        Ok(Response::new(UpdateCatalogScoreResponse {}))
    }

    async fn submit_score_rating(
        &self,
        req: Request<SubmitScoreRatingRequest>,
    ) -> Result<Response<SubmitScoreRatingResponse>, Status> {
        // Any signed-in user may rate (change: add-app-score-rating). The rater is
        // the authenticated caller — never the body. The module validates the
        // verdict/stars and rejects a non-`accepted`/unknown target.
        let user_id = owner(&req)?;
        let r = req.into_inner();
        let (agg, points) = self
            .module
            .submit_rating(&user_id, &r.catalog_id, &r.verdict, r.stars)
            .await?;
        Ok(Response::new(SubmitScoreRatingResponse {
            rating_avg: agg.avg_effective,
            rating_count: agg.count.clamp(0, i32::MAX as i64) as i32,
            dislike_count: agg.dislike.clamp(0, i32::MAX as i64) as i32,
            like_count: agg.like.clamp(0, i32::MAX as i64) as i32,
            love_count: agg.love.clamp(0, i32::MAX as i64) as i32,
            points_awarded: points.clamp(0, i32::MAX as i64) as i32,
        }))
    }

    async fn list_rating_deck(
        &self,
        req: Request<ListRatingDeckRequest>,
    ) -> Result<Response<ListRatingDeckResponse>, Status> {
        // Authenticated-only; the un-rated exclusion is per caller (change:
        // improve-rating-deck-sourcing).
        let id = identity(&req)?;
        self.guard_enumeration(&id).await?; // per-user browse cap (scrape guard)
        let user_id = id.user_id;
        let r = req.into_inner();
        let offset = r.offset;
        let mut hits = self
            .module
            .list_rating_deck(&user_id, r.limit as i64, r.offset as i64)
            .await?;
        // Deck is a normal-caller read: sanitise privileged proposer fields.
        self.module
            .attach_public_credit(&mut hits, today_utc())
            .await;
        let next_offset = offset.max(0) + hits.len() as i32;
        Ok(Response::new(ListRatingDeckResponse {
            hits: hits.into_iter().map(to_hit).collect(),
            next_offset,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_storage::{FakeStore, ObjectStorage};

    use crate::catalog_search::{FakeCatalogRow, FakeCatalogSearchRepo};
    use crate::score_rating::FakeScoreRatingRepo;
    use crate::user_library::FakeUserLibraryRepo;
    use crate::user_scores::FakeUserScoreRepo;

    const DEBUSSY: &str = "11111111-1111-7111-8111-111111111111";
    const SATIE: &str = "22222222-2222-7222-8222-222222222222";
    const PENDING: &str = "44444444-4444-7444-8444-444444444444";
    const REJECTED: &str = "55555555-5555-7555-8555-555555555555";

    /// A `ScoreGrpc` over a seeded catalog + an object store holding the catalog
    /// scores' bytes, so byte fetches resolve. The catalog carries two `accepted`
    /// scores, one `pending` score, and one `rejected` score (change:
    /// add-score-moderation-gating / rate-pending-scores), so the moderation gate and
    /// the rating/preview paths can be exercised at the handler layer.
    async fn grpc() -> ScoreGrpc {
        let store = Arc::new(FakeStore::default());
        for id in [DEBUSSY, SATIE, PENDING, REJECTED] {
            store
                .put(&format!("safe/pdmx/{id}.mxl"), b"<score/>".to_vec())
                .await
                .unwrap();
        }
        let catalog = Arc::new(FakeCatalogSearchRepo::with(vec![
            FakeCatalogRow::new(DEBUSSY, "Clair de Lune", "Claude Debussy", Some("advanced"))
                .piano(),
            FakeCatalogRow::new(SATIE, "Gymnopédie", "Erik Satie", Some("beginner")).piano(),
            FakeCatalogRow::new(PENDING, "Pending Piece", "Anon", Some("beginner"))
                .with_moderation_status("pending"),
            FakeCatalogRow::new(REJECTED, "Rejected Piece", "Anon", Some("beginner"))
                .with_moderation_status("rejected"),
        ]));
        let module = Arc::new(ScoreModule::new(
            Arc::new(FakeUserScoreRepo::default()),
            catalog,
            Arc::new(FakeUserLibraryRepo::default()),
            Arc::new(FakeScoreRatingRepo::default()),
            store,
            5,
            7,
            8 * 1024 * 1024,
        ));
        ScoreGrpc::new(module)
    }

    /// Like [`grpc`] but with the per-user access limiter enabled at tiny thresholds
    /// (download floor 2, burst 3, enum 2) so the guardrail trips quickly in tests
    /// (change: add-catalog-access-limits).
    async fn grpc_limited() -> ScoreGrpc {
        use crate::catalog_limits::CatalogAccessLimiter;
        use cymbra_platform::cache::FakeCache;
        use cymbra_platform::config::CatalogLimitsConfig;
        let cfg = CatalogLimitsConfig {
            enabled: true,
            download_burst_max: 3,
            download_burst_window: std::time::Duration::from_secs(60),
            volume_window: std::time::Duration::from_secs(24 * 3600),
            volume_base_floor: 2,
            volume_per_engagement: 2,
            volume_hard_ceiling: 20,
            enum_max: 2,
            enum_window: std::time::Duration::from_secs(60),
        };
        let limiter = Arc::new(CatalogAccessLimiter::new(
            Arc::new(FakeCache::default()),
            Arc::new(crate::play::FakePlayRepo::default()),
            Arc::new(crate::score_rating::FakeScoreRatingRepo::default()),
            cfg,
        ));
        grpc().await.with_limiter(limiter)
    }

    /// A test SoundFont catalog entry (accepted / publicly visible by default).
    fn font(id: &str, attr: Option<&str>) -> crate::soundfont::FontEntry {
        font_status(id, attr, "accepted")
    }

    /// A test SoundFont catalog entry with an explicit moderation status.
    fn font_status(id: &str, attr: Option<&str>, status: &str) -> crate::soundfont::FontEntry {
        crate::soundfont::FontEntry {
            id: id.into(),
            label: format!("{id} label"),
            object_key: format!("{id}.sf2"),
            instrument: "piano".into(),
            license: "CC0-1.0".into(),
            attribution: attr.map(Into::into),
            size_bytes: None,
            moderation_status: status.into(),
            reviewed_by: None,
            reviewed_at: None,
            uploaded_by: None,
            content_sha256: Some(crate::soundfont::sha256_hex(id.as_bytes())),
            point_cost: 0,
            redeemable: true,
            review_reason: None,
            resubmission_note: None,
        }
    }

    #[tokio::test]
    async fn list_soundfonts_returns_the_catalog_for_an_authed_caller() {
        use crate::soundfont::FakeSoundFontRepo;
        let svc = grpc()
            .await
            .with_soundfonts(Arc::new(FakeSoundFontRepo::with(vec![
                font("upright-piano-kw", None),
                font("ydp-grand", Some("Roberto / Zenph Studios")),
            ])));
        let resp = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.soundfonts.len(), 2);
        let ydp = resp
            .soundfonts
            .iter()
            .find(|f| f.id == "ydp-grand")
            .unwrap();
        assert_eq!(ydp.attribution, "Roberto / Zenph Studios");
        assert_eq!(ydp.instrument, "piano");
        // A missing attribution maps to an empty string on the wire.
        let up = resp
            .soundfonts
            .iter()
            .find(|f| f.id == "upright-piano-kw")
            .unwrap();
        assert_eq!(up.attribution, "");
        assert_eq!(up.instrument, "piano");
    }

    #[tokio::test]
    async fn list_soundfonts_requires_auth() {
        use crate::soundfont::FakeSoundFontRepo;
        let svc = grpc()
            .await
            .with_soundfonts(Arc::new(FakeSoundFontRepo::default()));
        let err = svc
            .list_sound_fonts(Request::new(ListSoundFontsRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn list_soundfonts_unavailable_without_a_repo() {
        let svc = grpc().await; // no catalog wired
        let err = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unavailable);
    }

    /// A ScoreGrpc with the SoundFont catalog + a store seeded with the font object.
    async fn grpc_soundfont_admin() -> (
        ScoreGrpc,
        std::sync::Arc<crate::soundfont::FakeSoundFontRepo>,
        std::sync::Arc<FakeStore>,
    ) {
        use crate::soundfont::FakeSoundFontRepo;
        let repo = Arc::new(FakeSoundFontRepo::with(vec![font(
            "ydp-grand",
            Some("Roberto"),
        )]));
        let store = Arc::new(FakeStore::default());
        store.put("ydp-grand.sf2", b"SF2".to_vec()).await.unwrap();
        let svc = grpc()
            .await
            .with_soundfonts(repo.clone())
            .with_soundfont_store(store.clone());
        (svc, repo, store)
    }

    #[tokio::test]
    async fn admin_soundfont_ops_require_moderator_or_admin() {
        let (svc, _repo, _store) = grpc_soundfont_admin().await;
        // A plain user (no moderator/admin) is refused on every admin op.
        let list = svc
            .admin_list_sound_fonts(authed(AdminListSoundFontsRequest::default(), "u"))
            .await
            .unwrap_err();
        assert_eq!(list.code(), tonic::Code::PermissionDenied);
        let upd = svc
            .update_sound_font(authed(
                UpdateSoundFontRequest {
                    id: "ydp-grand".into(),
                    label: "x".into(),
                    license: "x".into(),
                    attribution: String::new(),
                },
                "u",
            ))
            .await
            .unwrap_err();
        assert_eq!(upd.code(), tonic::Code::PermissionDenied);
        let del = svc
            .delete_sound_font(authed(
                DeleteSoundFontRequest {
                    id: "ydp-grand".into(),
                },
                "u",
            ))
            .await
            .unwrap_err();
        assert_eq!(del.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn admin_list_reports_has_object_and_has_preview() {
        let (svc, _repo, store) = grpc_soundfont_admin().await;
        let req = || {
            authed_admin(
                AdminListSoundFontsRequest {
                    limit: 50,
                    offset: 0,
                    moderation_status: String::new(),
                },
                "admin-1",
            )
        };
        // The font object is present; no preview clip has been rendered yet.
        let listed = svc
            .admin_list_sound_fonts(req())
            .await
            .unwrap()
            .into_inner();
        let row = listed
            .soundfonts
            .iter()
            .find(|f| f.id == "ydp-grand")
            .unwrap();
        assert!(row.has_object);
        assert!(!row.has_preview);

        // Once the preview object exists, has_preview flips true (drives the BO's
        // play / "Generate sample" control).
        store
            .put("ydp-grand.preview.wav", b"RIFF....WAVE".to_vec())
            .await
            .unwrap();
        let listed = svc
            .admin_list_sound_fonts(req())
            .await
            .unwrap()
            .into_inner();
        let row = listed
            .soundfonts
            .iter()
            .find(|f| f.id == "ydp-grand")
            .unwrap();
        assert!(row.has_preview);
    }

    #[tokio::test]
    async fn list_sound_fonts_reports_has_preview() {
        let (svc, _repo, store) = grpc_soundfont_admin().await;
        // No preview clip yet.
        let listed = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap()
            .into_inner();
        let row = listed
            .soundfonts
            .iter()
            .find(|f| f.id == "ydp-grand")
            .unwrap();
        assert!(!row.has_preview);

        // Once the preview object exists, the public listing reports it (drives the
        // app's up-front greying of a locked font's play control).
        store
            .put("ydp-grand.preview.wav", b"RIFF....WAVE".to_vec())
            .await
            .unwrap();
        let listed = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap()
            .into_inner();
        let row = listed
            .soundfonts
            .iter()
            .find(|f| f.id == "ydp-grand")
            .unwrap();
        assert!(row.has_preview);
    }

    #[tokio::test]
    async fn admin_updates_metadata() {
        let (svc, repo, _store) = grpc_soundfont_admin().await;
        svc.update_sound_font(authed_admin(
            UpdateSoundFontRequest {
                id: "ydp-grand".into(),
                label: "YDP Grand (edited)".into(),
                license: "CC-BY 3.0".into(),
                attribution: "Roberto / Zenph".into(),
            },
            "admin-1",
        ))
        .await
        .unwrap();
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.label, "YDP Grand (edited)");
        assert_eq!(e.attribution.as_deref(), Some("Roberto / Zenph"));
        // Instrument is immutable across a metadata edit.
        assert_eq!(e.instrument, "piano");

        // An unknown id is not-found.
        let err = svc
            .update_sound_font(authed_admin(
                UpdateSoundFontRequest {
                    id: "nope".into(),
                    label: "x".into(),
                    license: "x".into(),
                    attribution: String::new(),
                },
                "admin-1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn admin_delete_removes_row_and_object() {
        let (svc, repo, store) = grpc_soundfont_admin().await;
        svc.delete_sound_font(authed_admin(
            DeleteSoundFontRequest {
                id: "ydp-grand".into(),
            },
            "admin-1",
        ))
        .await
        .unwrap();
        assert!(repo.list().await.unwrap().is_empty());
        // The stored object is gone too.
        assert!(store.size("ydp-grand.sf2").await.is_err());

        // Deleting an unknown id is not-found.
        let err = svc
            .delete_sound_font(authed_admin(
                DeleteSoundFontRequest {
                    id: "ydp-grand".into(),
                },
                "admin-1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn admin_list_paginates_and_filters_by_status() {
        use crate::soundfont::FakeSoundFontRepo;
        // 3 accepted + 2 pending; label order a..e for deterministic paging.
        let repo = Arc::new(FakeSoundFontRepo::with(vec![
            font_status("a", None, "accepted"),
            font_status("b", None, "pending"),
            font_status("c", None, "accepted"),
            font_status("d", None, "pending"),
            font_status("e", None, "accepted"),
        ]));
        let svc = grpc().await.with_soundfonts(repo);

        // Page 1 of all: limit 2, offset 0 → 2 rows, total 5, next_offset 2.
        let p1 = svc
            .admin_list_sound_fonts(authed_admin(
                AdminListSoundFontsRequest {
                    limit: 2,
                    offset: 0,
                    moderation_status: String::new(),
                },
                "admin-1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(p1.total, 5);
        assert_eq!(p1.next_offset, 2);
        assert_eq!(
            p1.soundfonts
                .iter()
                .map(|f| f.id.clone())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        // KPI counts are catalog-wide, independent of the filter/page.
        assert_eq!(p1.total_count, 5);
        assert_eq!(p1.accepted_count, 3);
        assert_eq!(p1.pending_count, 2);
        assert_eq!(p1.rejected_count, 0);

        // Filter to pending: total 2 regardless of paging.
        let pending = svc
            .admin_list_sound_fonts(authed_admin(
                AdminListSoundFontsRequest {
                    limit: 50,
                    offset: 0,
                    moderation_status: "pending".into(),
                },
                "admin-1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(pending.total, 2);
        assert_eq!(
            pending
                .soundfonts
                .iter()
                .map(|f| f.id.clone())
                .collect::<Vec<_>>(),
            ["b", "d"]
        );
    }

    #[tokio::test]
    async fn list_soundfonts_hides_unvalidated() {
        use crate::soundfont::FakeSoundFontRepo;
        let svc = grpc()
            .await
            .with_soundfonts(Arc::new(FakeSoundFontRepo::with(vec![
                font_status("accepted-one", None, "accepted"),
                font_status("pending-one", None, "pending"),
                font_status("rejected-one", None, "rejected"),
            ])));
        let resp = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap()
            .into_inner();
        // Only the accepted font is offered publicly.
        assert_eq!(resp.soundfonts.len(), 1);
        assert_eq!(resp.soundfonts[0].id, "accepted-one");
    }

    #[tokio::test]
    async fn set_soundfont_moderation_status_gates_and_transitions() {
        use crate::soundfont::FakeSoundFontRepo;
        let repo = Arc::new(FakeSoundFontRepo::with(vec![font_status(
            "ydp-grand",
            None,
            "pending",
        )]));
        // Accepting requires the font's preview object to exist (change:
        // add-soundfont-entitlement-previews) — seed it so the accept transition passes.
        let store = Arc::new(FakeStore::default());
        store
            .put("ydp-grand.preview.wav", b"RIFF....WAVE".to_vec())
            .await
            .unwrap();
        let svc = grpc()
            .await
            .with_soundfonts(repo.clone())
            .with_soundfont_store(store.clone());

        // A plain user is refused.
        let denied = svc
            .set_sound_font_moderation_status(authed(
                SetSoundFontModerationStatusRequest {
                    id: "ydp-grand".into(),
                    status: "accepted".into(),
                    reason: None,
                },
                "u",
            ))
            .await
            .unwrap_err();
        assert_eq!(denied.code(), tonic::Code::PermissionDenied);

        // An invalid status value is rejected.
        let bad = svc
            .set_sound_font_moderation_status(authed_moderator(
                SetSoundFontModerationStatusRequest {
                    id: "ydp-grand".into(),
                    status: "banana".into(),
                    reason: None,
                },
                "mod-1",
            ))
            .await
            .unwrap_err();
        assert_eq!(bad.code(), tonic::Code::InvalidArgument);

        // A moderator accepts it → it becomes publicly visible, stamped with the reviewer.
        // The reviewer id is a UUID (as the real AuthIdentity.user_id is).
        let mod_uuid = "11111111-1111-1111-1111-111111111111";
        svc.set_sound_font_moderation_status(authed_moderator(
            SetSoundFontModerationStatusRequest {
                id: "ydp-grand".into(),
                status: "accepted".into(),
                reason: None,
            },
            mod_uuid,
        ))
        .await
        .unwrap();
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.moderation_status, "accepted");
        assert_eq!(e.reviewed_by.as_deref(), Some(mod_uuid));
        assert_eq!(repo.list_accepted().await.unwrap().len(), 1);

        // An unknown id is not-found.
        let missing = svc
            .set_sound_font_moderation_status(authed_moderator(
                SetSoundFontModerationStatusRequest {
                    id: "nope".into(),
                    status: "accepted".into(),
                    reason: None,
                },
                mod_uuid,
            ))
            .await
            .unwrap_err();
        assert_eq!(missing.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn accepting_a_soundfont_requires_a_preview_sample() {
        use crate::soundfont::FakeSoundFontRepo;
        let repo = Arc::new(FakeSoundFontRepo::with(vec![font_status(
            "no-preview",
            None,
            "pending",
        )]));
        // A store WITHOUT the font's preview object.
        let store = Arc::new(FakeStore::default());
        let svc = grpc()
            .await
            .with_soundfonts(repo.clone())
            .with_soundfont_store(store.clone());
        let mod_uuid = "11111111-1111-1111-1111-111111111111";

        // Accepting is refused while no preview exists — and the status is unchanged.
        let refused = svc
            .set_sound_font_moderation_status(authed_moderator(
                SetSoundFontModerationStatusRequest {
                    id: "no-preview".into(),
                    status: "accepted".into(),
                    reason: None,
                },
                mod_uuid,
            ))
            .await
            .unwrap_err();
        assert_eq!(refused.code(), tonic::Code::FailedPrecondition);
        assert_eq!(
            repo.lookup("no-preview")
                .await
                .unwrap()
                .unwrap()
                .moderation_status,
            "pending"
        );

        // Rejecting needs no preview — it still works.
        svc.set_sound_font_moderation_status(authed_moderator(
            SetSoundFontModerationStatusRequest {
                id: "no-preview".into(),
                status: "rejected".into(),
                reason: None,
            },
            mod_uuid,
        ))
        .await
        .unwrap();
        assert_eq!(
            repo.lookup("no-preview")
                .await
                .unwrap()
                .unwrap()
                .moderation_status,
            "rejected"
        );

        // Once the preview exists, acceptance goes through.
        store
            .put("no-preview.preview.wav", b"RIFF....WAVE".to_vec())
            .await
            .unwrap();
        svc.set_sound_font_moderation_status(authed_moderator(
            SetSoundFontModerationStatusRequest {
                id: "no-preview".into(),
                status: "accepted".into(),
                reason: None,
            },
            mod_uuid,
        ))
        .await
        .unwrap();
        assert_eq!(
            repo.lookup("no-preview")
                .await
                .unwrap()
                .unwrap()
                .moderation_status,
            "accepted"
        );
    }

    // --- Uploader attribution (change: add-soundfont-uploader-attribution) ---

    /// The uploader id used by the attribution tests (a UUID, as real ids are).
    const UPLOADER: &str = "33333333-3333-7333-8333-333333333333";

    #[tokio::test]
    async fn admin_list_resolves_uploader_pseudo_and_resubmission_note() {
        use crate::soundfont::FakeSoundFontRepo;
        use cymbra_user_port::{Account, MockUserPort};
        // A user-contributed (reopened) font and a seeded font with no uploader.
        let mut proposed = font_status("proposed-font", None, "pending");
        proposed.uploaded_by = Some(UPLOADER.into());
        proposed.resubmission_note = Some("fixed the licence".into());
        let mut user = MockUserPort::new();
        user.expect_get_account().returning(|uid| {
            Ok(Account {
                user_id: uid.to_string(),
                display_name: Some("Alice".into()),
                preferences: "{}".into(),
                version: 1,
                updated_at: 0,
                handle: Some("alice".into()),
                locale: None,
            })
        });
        let svc = grpc()
            .await
            .with_soundfonts(Arc::new(FakeSoundFontRepo::with(vec![
                proposed,
                font("upright-piano-kw", None),
            ])))
            .with_user_port(Arc::new(user));
        let resp = svc
            .admin_list_sound_fonts(authed_moderator(
                AdminListSoundFontsRequest::default(),
                "mod-1",
            ))
            .await
            .unwrap()
            .into_inner();
        let p = resp
            .soundfonts
            .iter()
            .find(|f| f.id == "proposed-font")
            .unwrap();
        // The pseudo is resolved unconditionally (privileged audit surface) and the
        // resubmission justification rides along for re-review.
        assert_eq!(p.uploader_display_name, "alice");
        assert_eq!(p.resubmission_note, "fixed the licence");
        // A seeded font carries neither an uploader id nor a pseudo.
        let s = resp
            .soundfonts
            .iter()
            .find(|f| f.id == "upright-piano-kw")
            .unwrap();
        assert_eq!(s.uploaded_by, "");
        assert_eq!(s.uploader_display_name, "");
        assert_eq!(s.resubmission_note, "");
    }

    #[tokio::test]
    async fn public_listing_credits_only_a_public_uploader() {
        use crate::soundfont::FakeSoundFontRepo;
        use cymbra_user_port::{MockUserPort, PlayerProfile, Visibility};
        const PRIVATE_UPLOADER: &str = "66666666-6666-7666-8666-666666666666";
        let mut credited = font("credited", Some("Sample Author"));
        credited.uploaded_by = Some(UPLOADER.into());
        let mut uncredited = font("uncredited", None);
        uncredited.uploaded_by = Some(PRIVATE_UPLOADER.into());
        // The directory returns ONLY the public, age-eligible uploader (fail-closed):
        // the private one is simply absent from the batch result.
        let mut user = MockUserPort::new();
        user.expect_listable_profiles().returning(|ids, _| {
            assert!(ids.contains(&UPLOADER.to_string()));
            Ok(vec![PlayerProfile {
                user_id: UPLOADER.to_string(),
                handle: Some("alice".into()),
                display_name: Some("Alice".into()),
                visibility: Visibility::Public,
            }])
        });
        let svc = grpc()
            .await
            .with_soundfonts(Arc::new(FakeSoundFontRepo::with(vec![
                credited,
                uncredited,
                font("upright-piano-kw", Some("K. W.")),
            ])))
            .with_user_port(Arc::new(user));
        let resp = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap()
            .into_inner();
        let by_id = |id: &str| resp.soundfonts.iter().find(|f| f.id == id).unwrap();
        // Public uploader → credit, alongside (not replacing) the licence attribution.
        assert_eq!(by_id("credited").contributor_credit, "alice");
        assert_eq!(by_id("credited").attribution, "Sample Author");
        // Private/absent uploader and seeded font → no credit; licence attribution stays.
        assert_eq!(by_id("uncredited").contributor_credit, "");
        assert_eq!(by_id("upright-piano-kw").contributor_credit, "");
        assert_eq!(by_id("upright-piano-kw").attribution, "K. W.");
    }

    #[tokio::test]
    async fn public_credit_degrades_to_absent_when_the_directory_fails() {
        use crate::soundfont::FakeSoundFontRepo;
        use cymbra_user_port::MockUserPort;
        let mut credited = font("credited", None);
        credited.uploaded_by = Some(UPLOADER.into());
        let mut user = MockUserPort::new();
        user.expect_listable_profiles()
            .returning(|_, _| Err(cymbra_platform::AppError::Internal(anyhow::anyhow!("down"))));
        let svc = grpc()
            .await
            .with_soundfonts(Arc::new(FakeSoundFontRepo::with(vec![credited])))
            .with_user_port(Arc::new(user));
        let resp = svc
            .list_sound_fonts(authed(ListSoundFontsRequest {}, "u"))
            .await
            .unwrap()
            .into_inner();
        // The listing still serves; the credit is simply omitted (fail-closed).
        assert_eq!(resp.soundfonts.len(), 1);
        assert_eq!(resp.soundfonts[0].contributor_credit, "");
    }

    #[tokio::test]
    async fn rejecting_with_a_reason_stores_it_and_other_decisions_clear_it() {
        use crate::soundfont::FakeSoundFontRepo;
        let repo = Arc::new(FakeSoundFontRepo::with(vec![font_status(
            "ydp-grand",
            None,
            "pending",
        )]));
        let store = Arc::new(FakeStore::default());
        store
            .put("ydp-grand.preview.wav", b"RIFF....WAVE".to_vec())
            .await
            .unwrap();
        let svc = grpc()
            .await
            .with_soundfonts(repo.clone())
            .with_soundfont_store(store);
        let mod_uuid = "11111111-1111-1111-1111-111111111111";
        // Rejecting with a reason stores the trimmed motive.
        svc.set_sound_font_moderation_status(authed_moderator(
            SetSoundFontModerationStatusRequest {
                id: "ydp-grand".into(),
                status: "rejected".into(),
                reason: Some("  too noisy  ".into()),
            },
            mod_uuid,
        ))
        .await
        .unwrap();
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.review_reason.as_deref(), Some("too noisy"));
        // A non-`rejected` decision ignores any carried reason and clears the stale one.
        svc.set_sound_font_moderation_status(authed_moderator(
            SetSoundFontModerationStatusRequest {
                id: "ydp-grand".into(),
                status: "accepted".into(),
                reason: Some("ignored".into()),
            },
            mod_uuid,
        ))
        .await
        .unwrap();
        let e = repo.lookup("ydp-grand").await.unwrap().unwrap();
        assert_eq!(e.moderation_status, "accepted");
        assert!(e.review_reason.is_none());
    }

    /// Attach an authenticated identity to a request (as the interceptor would).
    fn authed<T>(msg: T, user_id: &str) -> Request<T> {
        authed_with(msg, user_id, &["user"])
    }

    /// Attach an authenticated identity carrying the `admin` role.
    fn authed_admin<T>(msg: T, user_id: &str) -> Request<T> {
        authed_with(msg, user_id, &["user", "admin"])
    }

    /// Attach an authenticated identity carrying the `moderator` role.
    fn authed_moderator<T>(msg: T, user_id: &str) -> Request<T> {
        authed_with(msg, user_id, &["user", "moderator"])
    }

    fn authed_with<T>(msg: T, user_id: &str, roles: &[&str]) -> Request<T> {
        let mut req = Request::new(msg);
        req.extensions_mut().insert(AuthIdentity {
            user_id: user_id.into(),
            audience: "music".into(),
            roles: roles.iter().map(|r| (*r).into()).collect(),
            ..Default::default()
        });
        req
    }

    fn search(query: &str, author: Option<&str>, level: Option<&str>) -> SearchCatalogRequest {
        SearchCatalogRequest {
            query: query.into(),
            author: author.map(Into::into),
            level: level.map(Into::into),
            limit: 50,
            offset: 0,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn unauthenticated_requests_are_rejected() {
        let g = grpc().await;
        // No AuthIdentity in the extensions → unauthenticated on every hub RPC.
        let err = g
            .search_catalog(Request::new(search("", None, None)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        let err = g
            .save_catalog_score(Request::new(SaveCatalogScoreRequest {
                catalog_id: DEBUSSY.into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        let err = g
            .list_saved_catalog_scores(Request::new(ListSavedCatalogScoresRequest {}))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn access_limits_reject_download_and_enumeration_floods() {
        let g = grpc_limited().await;
        // Download floor is 2 (no plays) → the 3rd catalog-bytes fetch is rejected
        // with RESOURCE_EXHAUSTED (before any storage read).
        for _ in 0..2 {
            g.get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: DEBUSSY.into(),
                },
                "u1",
            ))
            .await
            .expect("within download floor");
        }
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: DEBUSSY.into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::ResourceExhausted);

        // Rating-preview bytes share the same per-user download counter, so u1 is
        // already over budget on that path too.
        let err = g
            .get_rating_preview_bytes(authed(
                GetRatingPreviewBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::ResourceExhausted);

        // Enumeration cap is 2 → the 3rd search by a fresh user is throttled while
        // the page-size clamp is unaffected.
        for _ in 0..2 {
            g.search_catalog(authed(search("", None, None), "u2"))
                .await
                .expect("within enum cap");
        }
        let err = g
            .search_catalog(authed(search("", None, None), "u2"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::ResourceExhausted);
    }

    #[tokio::test]
    async fn search_composes_query_author_and_level() {
        let g = grpc().await;
        // Author + level filter narrows to the one advanced Debussy work.
        let resp = g
            .search_catalog(authed(search("", Some("Debussy"), Some("advanced")), "u1"))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, [DEBUSSY]);
        assert_eq!(resp.next_offset, 1);
        assert_eq!(resp.total, 1); // full match count for the filter, on the response
        assert_eq!(resp.hits[0].license, "CC-BY-4.0");
    }

    #[tokio::test]
    async fn save_list_remove_round_trip_reflects_across_calls() {
        let g = grpc().await;
        // Save two, list returns them newest-first for the SAME owner (the sync
        // source of truth — a later list reflects earlier writes).
        g.save_catalog_score(authed(
            SaveCatalogScoreRequest {
                catalog_id: SATIE.into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        g.save_catalog_score(authed(
            SaveCatalogScoreRequest {
                catalog_id: DEBUSSY.into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        let listed = g
            .list_saved_catalog_scores(authed(ListSavedCatalogScoresRequest {}, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            listed
                .hits
                .iter()
                .map(|h| h.id.as_str())
                .collect::<Vec<_>>(),
            [DEBUSSY, SATIE]
        );
        // Another owner sees none of u1's saves (isolation).
        let other = g
            .list_saved_catalog_scores(authed(ListSavedCatalogScoresRequest {}, "u2"))
            .await
            .unwrap()
            .into_inner();
        assert!(other.hits.is_empty());
        // Remove one; a subsequent list reflects it.
        g.remove_saved_catalog_score(authed(
            RemoveSavedCatalogScoreRequest {
                catalog_id: SATIE.into(),
            },
            "u1",
        ))
        .await
        .unwrap();
        let listed = g
            .list_saved_catalog_scores(authed(ListSavedCatalogScoresRequest {}, "u1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            listed
                .hits
                .iter()
                .map(|h| h.id.as_str())
                .collect::<Vec<_>>(),
            [DEBUSSY]
        );
    }

    #[tokio::test]
    async fn bytes_for_known_and_unknown_id() {
        let g = grpc().await;
        let resp = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: DEBUSSY.into(),
                },
                "u1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.data, b"<score/>");
        // Unknown id → NotFound.
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: "99999999-9999-7999-8999-999999999999".into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    // --- moderation gating (change: add-score-moderation-gating) -------------

    fn search_status(status: &str) -> SearchCatalogRequest {
        SearchCatalogRequest {
            limit: 50,
            offset: 0,
            moderation_status: Some(status.into()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn normal_search_hides_pending_scores() {
        let g = grpc().await;
        // A normal caller browsing (no status filter) sees only the accepted rows,
        // never the pending one.
        let resp = g
            .search_catalog(authed(search("", None, None), "u1"))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, [DEBUSSY, SATIE]); // title_norm order; PENDING excluded
        assert!(!ids.contains(&PENDING));
    }

    #[tokio::test]
    async fn non_admin_supplying_status_filter_is_permission_denied() {
        let g = grpc().await;
        // A normal (non-admin) caller that sets the privileged filter is rejected,
        // and no query runs (the request never reaches the search path).
        let err = g
            .search_catalog(authed(search_status("pending"), "u1"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn admin_supplying_status_filter_is_honoured() {
        let g = grpc().await;
        // An admin caller's `pending` filter returns exactly the pending score.
        let resp = g
            .search_catalog(authed_admin(search_status("pending"), "admin1"))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, [PENDING]);
    }

    #[tokio::test]
    async fn bytes_of_pending_score_gated_by_role() {
        let g = grpc().await;
        // A normal caller cannot open the pending score's bytes (not found)…
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "u1",
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        // …but an admin reviewer is served them.
        let resp = g
            .get_catalog_score_bytes(authed_admin(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "admin1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.data, b"<score/>");
    }

    // --- moderator role widening + evaluate + sort (add-moderation-back-office) --

    #[tokio::test]
    async fn moderator_may_use_status_filter_and_open_pending_bytes() {
        let g = grpc().await;
        // A moderator (not admin) can now use the privileged status filter…
        let resp = g
            .search_catalog(authed_moderator(search_status("pending"), "mod1"))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            resp.hits.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
            [PENDING]
        );
        // …and open a pending score's bytes to review it.
        let resp = g
            .get_catalog_score_bytes(authed_moderator(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                "mod1",
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.data, b"<score/>");
    }

    #[tokio::test]
    async fn moderation_sort_key_is_privileged() {
        let g = grpc().await;
        let sorted = || SearchCatalogRequest {
            limit: 50,
            sort: vec![crate::proto::SortKey {
                field: "status_rank".into(),
                descending: true,
            }],
            ..Default::default()
        };
        // A normal caller sorting by a moderation-oriented key is denied…
        let err = g.search_catalog(authed(sorted(), "u1")).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // …while a moderator may (a substance-only sort stays open to anyone, so
        // this asserts the privileged path specifically).
        assert!(
            g.search_catalog(authed_moderator(sorted(), "mod1"))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn substance_sort_key_is_open_to_normal_callers() {
        let g = grpc().await;
        let req = SearchCatalogRequest {
            limit: 50,
            sort: vec![crate::proto::SortKey {
                field: "note_count".into(),
                descending: true,
            }],
            ..Default::default()
        };
        // No moderation key → no privilege required.
        assert!(g.search_catalog(authed(req, "u1")).await.is_ok());
    }

    #[tokio::test]
    async fn set_moderation_status_requires_moderator_or_admin() {
        let g = grpc().await;
        let req = || SetModerationStatusRequest {
            score_id: PENDING.into(),
            status: "accepted".into(),
            reason: None,
        };
        // A normal caller is denied and nothing changes.
        let err = g
            .set_moderation_status(authed(req(), "u1"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // A moderator succeeds; the score is now visible to a normal search.
        g.set_moderation_status(authed_moderator(req(), "mod1"))
            .await
            .unwrap();
        let resp = g
            .search_catalog(authed(search("", None, None), "u1"))
            .await
            .unwrap()
            .into_inner();
        assert!(resp.hits.iter().any(|h| h.id == PENDING));
    }

    #[tokio::test]
    async fn update_catalog_score_requires_moderator_or_admin() {
        let g = grpc().await;
        let req = || UpdateCatalogScoreRequest {
            score_id: PENDING.into(),
            title: Some("Corrected Title".into()),
            composer: None,
            arranger: None,
            level: None,
        };
        // A normal caller is denied and nothing changes.
        let err = g
            .update_catalog_score(authed(req(), "u1"))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
        // A moderator succeeds.
        g.update_catalog_score(authed_moderator(
            req(),
            "77777777-7777-7777-8777-777777777777",
        ))
        .await
        .unwrap();
    }

    // --- score ratings (change: add-app-score-rating) ------------------------

    const RATER: &str = "66666666-6666-7666-8666-666666666666";

    fn rating(catalog_id: &str, verdict: &str, stars: Option<i32>) -> SubmitScoreRatingRequest {
        SubmitScoreRatingRequest {
            catalog_id: catalog_id.into(),
            verdict: verdict.into(),
            stars,
        }
    }

    #[tokio::test]
    async fn submit_rating_requires_authentication() {
        let g = grpc().await;
        // No identity in the extensions → unauthenticated.
        let err = g
            .submit_score_rating(Request::new(rating(DEBUSSY, "like", None)))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn submit_rating_records_and_returns_the_aggregate() {
        let g = grpc().await;
        // A signed-in user rates an accepted score with explicit stars.
        let resp = g
            .submit_score_rating(authed(rating(DEBUSSY, "love", Some(5)), RATER))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.rating_count, 1);
        assert_eq!(resp.love_count, 1);
        assert!((resp.rating_avg - 5.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn submit_rating_on_a_pending_score_succeeds() {
        let g = grpc().await;
        // change: rate-pending-scores — a signed-in user CAN rate a pending candidate
        // (the community helps moderate); the rating is recorded.
        let resp = g
            .submit_score_rating(authed(rating(PENDING, "like", None), RATER))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(resp.rating_count, 1);
        assert_eq!(resp.like_count, 1);
    }

    #[tokio::test]
    async fn submit_rating_on_a_rejected_score_is_rejected() {
        let g = grpc().await;
        // A `rejected` score is never rateable.
        let err = g
            .submit_score_rating(authed(rating(REJECTED, "like", None), RATER))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn rating_preview_bytes_serves_pending_and_accepted_refuses_rejected() {
        let g = grpc().await;
        // A signed-in rater previews both an accepted score and a pending candidate.
        for id in [DEBUSSY, PENDING] {
            let resp = g
                .get_rating_preview_bytes(authed(
                    GetRatingPreviewBytesRequest {
                        catalog_id: id.into(),
                    },
                    RATER,
                ))
                .await
                .unwrap()
                .into_inner();
            assert!(!resp.data.is_empty());
        }
        // A rejected score is never previewable.
        let err = g
            .get_rating_preview_bytes(authed(
                GetRatingPreviewBytesRequest {
                    catalog_id: REJECTED.into(),
                },
                RATER,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        // Unauthenticated → rejected.
        let err = g
            .get_rating_preview_bytes(Request::new(GetRatingPreviewBytesRequest {
                catalog_id: DEBUSSY.into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }

    #[tokio::test]
    async fn player_open_bytes_stays_accepted_only_for_a_normal_caller() {
        let g = grpc().await;
        // The player-open path is unchanged by rate-pending-scores: a normal caller
        // still cannot open a pending score there…
        let err = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: PENDING.into(),
                },
                RATER,
            ))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
        // …but the accepted score opens fine.
        let resp = g
            .get_catalog_score_bytes(authed(
                GetCatalogScoreBytesRequest {
                    catalog_id: DEBUSSY.into(),
                },
                RATER,
            ))
            .await
            .unwrap()
            .into_inner();
        assert!(!resp.data.is_empty());
    }

    #[tokio::test]
    async fn list_rating_deck_requires_auth() {
        let g = grpc().await;
        // Unauthenticated → rejected.
        let err = g
            .list_rating_deck(Request::new(ListRatingDeckRequest {
                limit: 50,
                offset: 0,
            }))
            .await
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
        // A signed-in caller gets the un-rated piano scores. The pending/rejected rows
        // in this fixture are non-piano, so they are excluded by the deck's piano gate
        // regardless of status (pending sourcing is covered by the module test).
        let resp = g
            .list_rating_deck(authed(
                ListRatingDeckRequest {
                    limit: 50,
                    offset: 0,
                },
                RATER,
            ))
            .await
            .unwrap()
            .into_inner();
        let ids: Vec<&str> = resp.hits.iter().map(|h| h.id.as_str()).collect();
        assert!(ids.contains(&DEBUSSY) && ids.contains(&SATIE));
        assert!(!ids.contains(&PENDING));
    }
}
