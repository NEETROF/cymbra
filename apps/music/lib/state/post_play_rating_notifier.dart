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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import '../services/rating_service.dart';
import 'player_notifier.dart';
import 'post_play_rating_core.dart';
import 'score_catalog.dart';
import 'session_notifier.dart';

part 'post_play_rating_notifier.g.dart';

/// Everything the post-play rating prompt needs to know about the score currently
/// open in the player (change: add-post-play-rating-prompt).
class PostPlayRatingData {
  const PostPlayRatingData({
    this.catalogId,
    this.rated = RatedState.unknown,
    this.offered = const [],
    this.submittedStars,
    this.failed = false,
  });

  /// The played score's catalog id — null for a bundled or user-contributed
  /// score, neither of which is rateable.
  final String? catalogId;

  /// Whether the caller has already rated it (server truth), or `unknown` while
  /// the read is in flight, offline, or failed.
  final RatedState rated;

  /// Catalog ids already offered for rating on this device, oldest first.
  final List<String> offered;

  /// The star value just submitted from a prompt, so the surface can switch to
  /// its thanks state. Null until the user rates.
  final int? submittedStars;

  /// A submission failed. The surface shows a localized message — never the
  /// underlying transport error.
  final bool failed;

  /// Whether this score has already been offered (so it must never be again).
  bool get alreadyOffered => catalogId != null && offered.contains(catalogId);

  PostPlayRatingData copyWith({
    RatedState? rated,
    List<String>? offered,
    int? submittedStars,
    bool? failed,
  }) => PostPlayRatingData(
    catalogId: catalogId,
    rated: rated ?? this.rated,
    offered: offered ?? this.offered,
    submittedStars: submittedStars ?? this.submittedStars,
    failed: failed ?? this.failed,
  );
}

/// Owns the post-play prompt's state for the score currently open in the player:
/// the caller's existing rating (resolved once per open), the per-score
/// "already offered" memory, and the submission.
///
/// The build is async and nothing awaits it on the play path — an unresolved read
/// simply leaves `rated` at [RatedState.unknown], which suppresses the prompt.
/// Rebuilds when the selected score changes, so opening another piece re-resolves.
@Riverpod(keepAlive: true)
class PostPlayRating extends _$PostPlayRating {
  /// Preferences key holding the comma-separated catalog ids already offered.
  static const String prefsKey = 'rating_prompt_offered';

  @override
  Future<PostPlayRatingData> build() async {
    final catalogId = ref.watch(selectedScoreProvider)?.catalogId;
    final online = ref.watch(canUseOnlineServicesProvider);
    final offered = await _readOffered();
    // A guest, or a score that isn't in the public catalog, can never be rated —
    // don't spend a round-trip asking.
    if (!online || catalogId == null) {
      return PostPlayRatingData(catalogId: catalogId, offered: offered);
    }
    RatedState rated;
    try {
      final mine = await ref
          .read(ratingServiceProvider)
          .myRating(catalogId: catalogId);
      rated = mine.rated ? RatedState.rated : RatedState.notRated;
    } catch (_) {
      // Offline or a failed read → unknown, which suppresses the prompt rather
      // than risking a doomed submission or a second prompt for a score already
      // rated on another device.
      rated = RatedState.unknown;
    }
    return PostPlayRatingData(
      catalogId: catalogId,
      rated: rated,
      offered: offered,
    );
  }

  /// Record that the prompt was **shown** for [catalogId], retiring that score
  /// from any future prompt. Called when the surface appears, not when it is
  /// answered, so a dismissal and a rating retire it alike. Persisted best-effort:
  /// losing the write only risks one extra prompt on a later run.
  Future<void> markOffered(String catalogId) async {
    final data = state.valueOrNull;
    if (data == null || data.alreadyOffered) return;
    final next = rememberOffered(data.offered, catalogId);
    state = AsyncData(data.copyWith(offered: next));
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, next.join(','));
    } catch (_) {
      // Best-effort: the in-memory value still applies for this session.
    }
  }

  /// Submit a 1–5 star rating for the played score, deriving the verdict the same
  /// way the deck does so stars and swipes fold onto one scale.
  ///
  /// Optimistic: the thanks state shows immediately. A failure flips [failed] so
  /// the surface can say so in the user's language — it never throws, because the
  /// caller may be on its way out of the player and must not be blocked.
  Future<void> submit(int stars) async {
    final data = state.valueOrNull;
    final id = data?.catalogId;
    if (data == null || id == null) return;
    state = AsyncData(data.copyWith(submittedStars: stars, failed: false));
    try {
      await ref
          .read(ratingServiceProvider)
          .submit(catalogId: id, verdict: verdictForStars(stars), stars: stars);
      // The score is now rated for good — belt and braces with the server truth
      // resolved on the next open.
      state = AsyncData(
        (state.valueOrNull ?? data).copyWith(rated: RatedState.rated),
      );
    } catch (_) {
      state = AsyncData((state.valueOrNull ?? data).copyWith(failed: true));
    }
  }

  Future<List<String>> _readOffered() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      if (raw == null || raw.isEmpty) return const [];
      return raw.split(',').where((s) => s.isNotEmpty).toList();
    } catch (_) {
      return const []; // storage unavailable → nothing offered yet
    }
  }
}

/// Whether to offer the rating prompt right now.
///
/// [reachedEnd] distinguishes the two surfaces: the summary modal passes true (a
/// finished run is engagement whatever the note count), the early-exit sheet
/// passes false and is gated on having heard [kRatingPromptMinPlayedFraction] of
/// the piece's notes.
///
/// Watching the played fraction (not the raw playhead) keeps this from rebuilding
/// every frame: it only changes when the playhead passes another note.
@riverpod
bool postPlayRatingEligible(Ref ref, {required bool reachedEnd}) {
  final data = ref.watch(postPlayRatingProvider).valueOrNull;
  if (data == null) return false;
  bool decide({required double playedFraction, required bool reachedEnd}) =>
      shouldPromptRating(
        signedIn: ref.watch(canUseOnlineServicesProvider),
        catalogId: data.catalogId,
        rated: data.rated,
        alreadyOffered: data.alreadyOffered,
        playedFraction: playedFraction,
        reachedEnd: reachedEnd,
      );
  // Settle every cheap term first, with the playback term forced true. If one of
  // them says no — a guest, a bundled score, already rated, already offered —
  // there is no reason to look at the player at all, so this provider never
  // depends on it in those cases (and the summary modal, which passes
  // `reachedEnd`, never does either).
  if (!decide(playedFraction: 1, reachedEnd: true)) return false;
  if (reachedEnd) return true;
  return decide(
    playedFraction: ref.watch(
      playerProvider.select(
        (s) => playedNoteFraction(s.notes, s.furthestElapsedMs),
      ),
    ),
    reachedEnd: false,
  );
}
