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
    this.declined = const [],
    this.submittedStars,
    this.failed = false,
  });

  /// The played score's catalog id — null for a bundled or user-contributed
  /// score, neither of which is rateable.
  final String? catalogId;

  /// Whether the caller has already rated it (server truth), or `unknown` while
  /// the read is in flight, offline, or failed.
  final RatedState rated;

  /// Catalog ids the user has **explicitly refused** to rate on this device,
  /// oldest first. Being shown the prompt does not put a score here — only a
  /// deliberate "not this one" does.
  final List<String> declined;

  /// The star value just submitted from a prompt, so the surface can switch to
  /// its thanks state. Null until the user rates.
  final int? submittedStars;

  /// A submission failed. The surface shows a localized message — never the
  /// underlying transport error.
  final bool failed;

  /// Whether the user has explicitly refused to rate this score.
  bool get isDeclined => catalogId != null && declined.contains(catalogId);

  PostPlayRatingData copyWith({
    RatedState? rated,
    List<String>? declined,
    int? submittedStars,
    bool? failed,
  }) => PostPlayRatingData(
    catalogId: catalogId,
    rated: rated ?? this.rated,
    declined: declined ?? this.declined,
    submittedStars: submittedStars ?? this.submittedStars,
    failed: failed ?? this.failed,
  );
}

/// Owns the post-play prompt's state for the score currently open in the player:
/// the caller's existing rating (resolved once per open), the per-score
/// explicit-refusal memory, and the submission.
///
/// The build is async and nothing awaits it on the play path — an unresolved read
/// simply leaves `rated` at [RatedState.unknown], which suppresses the prompt.
/// Rebuilds when the selected score changes, so opening another piece re-resolves.
@Riverpod(keepAlive: true)
class PostPlayRating extends _$PostPlayRating {
  /// Preferences key holding the comma-separated catalog ids the user explicitly
  /// refused to rate.
  static const String prefsKey = 'rating_prompt_declined';

  @override
  Future<PostPlayRatingData> build() async {
    final catalogId = ref.watch(selectedScoreProvider)?.catalogId;
    final online = ref.watch(canUseOnlineServicesProvider);
    final declined = await _readDeclined();
    // A guest, or a score that isn't in the public catalog, can never be rated —
    // don't spend a round-trip asking.
    if (!online || catalogId == null) {
      return PostPlayRatingData(catalogId: catalogId, declined: declined);
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
      declined: declined,
    );
  }

  /// Record that the user **explicitly refused** to rate [catalogId], retiring it
  /// from any future prompt on this device.
  ///
  /// Only a deliberate "not this one" calls this — never the mere fact of having
  /// been shown the prompt. Closing a summary to leave the player says nothing
  /// about the piece, so the offer stands and returns on the next run; the user's
  /// escape hatch is this refusal, which is why both surfaces expose it.
  ///
  /// Persisted best-effort: losing the write only risks one more prompt later.
  Future<void> decline(String catalogId) async {
    final data = state.valueOrNull;
    if (data == null || data.isDeclined) return;
    final next = rememberDeclined(data.declined, catalogId);
    state = AsyncData(data.copyWith(declined: next));
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
  ///
  /// A rating needs no entry in the declined list: the score is now rated, which
  /// the server reports on the next open and which [RatedState.rated] reflects
  /// straight away.
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

  Future<List<String>> _readDeclined() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      if (raw == null || raw.isEmpty) return const [];
      return raw.split(',').where((s) => s.isNotEmpty).toList();
    } catch (_) {
      return const []; // storage unavailable → nothing declined yet
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
        declined: data.isDeclined,
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
