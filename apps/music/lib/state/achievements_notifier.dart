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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/achievements_service.dart';
import '../services/preferences_service.dart';
import 'app_locale.dart';

part 'achievements_notifier.freezed.dart';
part 'achievements_notifier.g.dart';

// --- Presentation models -----------------------------------------------------

/// One tile in the grid. A graduated series collapses to a SINGLE tile (design
/// D5): the tile wears the highest tier the user has earned and its progress
/// indicator points at the next one. The full [ladder] travels with it so the
/// detail sheet can show every rung without another read.
@freezed
abstract class AchievementTileView with _$AchievementTileView {
  const AchievementTileView._();

  const factory AchievementTileView({
    /// The tier whose identity the tile shows: the highest EARNED tier, or the
    /// lowest tier when none is earned yet.
    required AchievementBadgeView badge,

    /// The next tier still to earn — what the progress indicator measures. Null
    /// at the top of a completed ladder.
    AchievementBadgeView? next,

    /// Every tier in ascending threshold order. A standalone badge has exactly
    /// one entry, so the detail sheet needs no special case.
    @Default(<AchievementBadgeView>[]) List<AchievementBadgeView> ladder,

    /// Earned since the user last opened the section.
    @Default(false) bool isNew,
  }) = _AchievementTileView;

  /// Whether anything on this ladder has been earned.
  bool get earned => badge.earned;

  /// The badge the progress indicator reflects: the next locked tier when there
  /// is one, else the (completed) displayed badge.
  AchievementBadgeView get target => next ?? badge;

  /// The track key, or the badge key for a standalone badge — a stable identity
  /// for the tile itself.
  String get id => badge.isTracked ? badge.track : badge.key;
}

/// One family section of the grid.
@freezed
abstract class AchievementFamilyView with _$AchievementFamilyView {
  const AchievementFamilyView._();

  const factory AchievementFamilyView({
    required String family,
    required List<AchievementTileView> tiles,
  }) = _AchievementFamilyView;

  int get earnedCount => tiles.where((t) => t.earned).length;
  int get totalCount => tiles.length;
}

/// The whole Achievements surface: families in registry order, each with its
/// tiles earned-first. [languageCode] is the display language the labels were
/// resolved against, so a widget renders without re-resolving.
@freezed
abstract class AchievementsView with _$AchievementsView {
  const AchievementsView._();

  const factory AchievementsView({
    @Default(<AchievementFamilyView>[]) List<AchievementFamilyView> families,
    @Default('en') String languageCode,

    /// The most recent moment any badge was earned, or null when none has been.
    /// The "mark seen" listener records this so the new markers clear on the
    /// next visit.
    DateTime? newestEarnedAt,
  }) = _AchievementsView;

  bool get isEmpty => families.isEmpty;
}

// --- Grouping ----------------------------------------------------------------

/// Group a flat registry projection into the grid the profile renders: families
/// in the order the server sent them (registry order — the app holds no list of
/// its own), tracks collapsed to one tile, earned tiles before locked ones.
///
/// A badge earned strictly after [lastSeen] is marked **new**. When [lastSeen] is
/// null the user has never opened the section, and NOTHING is marked: the first
/// visit establishes the baseline rather than flagging a lifetime of badges at
/// once.
AchievementsView groupAchievements(
  List<AchievementBadgeView> badges, {
  required String languageCode,
  DateTime? lastSeen,
}) {
  // Family order follows first appearance, so the server decides it.
  final order = <String>[];
  final byFamily = <String, List<AchievementBadgeView>>{};
  for (final b in badges) {
    if (!byFamily.containsKey(b.family)) order.add(b.family);
    (byFamily[b.family] ??= <AchievementBadgeView>[]).add(b);
  }

  final families = <AchievementFamilyView>[];
  for (final family in order) {
    final tiles = _tilesFor(byFamily[family]!, lastSeen);
    // A family with no badges never renders — which is what lets a family be
    // declared server-side ahead of the counter that will feed it.
    if (tiles.isEmpty) continue;
    // Earned first; within each group the server's (registry) order is kept, so
    // a track's position does not jump around as tiers are earned.
    final earned = tiles.where((t) => t.earned).toList();
    final locked = tiles.where((t) => !t.earned).toList();
    families.add(
      AchievementFamilyView(family: family, tiles: [...earned, ...locked]),
    );
  }

  final earnedMoments = badges
      .map((b) => b.earnedAt)
      .whereType<DateTime>()
      .toList();
  return AchievementsView(
    families: families,
    languageCode: languageCode,
    newestEarnedAt: earnedMoments.isEmpty
        ? null
        : earnedMoments.reduce((a, b) => a.isAfter(b) ? a : b),
  );
}

/// Collapse one family's badges into tiles: one per track, one per standalone
/// badge.
List<AchievementTileView> _tilesFor(
  List<AchievementBadgeView> badges,
  DateTime? lastSeen,
) {
  final order = <String>[];
  final groups = <String, List<AchievementBadgeView>>{};
  for (final b in badges) {
    final id = b.isTracked ? b.track : b.key;
    if (!groups.containsKey(id)) order.add(id);
    (groups[id] ??= <AchievementBadgeView>[]).add(b);
  }
  return [for (final id in order) _tile(groups[id]!, lastSeen)];
}

AchievementTileView _tile(
  List<AchievementBadgeView> ladder,
  DateTime? lastSeen,
) {
  final sorted = [...ladder]
    ..sort((a, b) => a.threshold.compareTo(b.threshold));
  final earned = sorted.where((b) => b.earned).toList();
  // Highest earned tier if any, else the entry rung — so a locked track shows
  // what it takes to get started rather than its unreachable top.
  final display = earned.isNotEmpty ? earned.last : sorted.first;
  final next = sorted.where((b) => !b.earned).firstOrNull;
  final isNew =
      lastSeen != null &&
      earned.any((b) => b.earnedAt != null && b.earnedAt!.isAfter(lastSeen));
  return AchievementTileView(
    badge: display,
    next: next,
    ladder: sorted,
    isNew: isNew,
  );
}

// --- Providers ---------------------------------------------------------------

/// Persisted "last seen achievements" timestamp. Its **own** key, not the curator
/// activity one: badges and the points activity feed are independent surfaces
/// now, and seeing one must not silently clear the other's markers.
@Riverpod(keepAlive: true)
class AchievementsSeen extends _$AchievementsSeen {
  /// Preferences key holding the epoch-millis of the newest earned badge the
  /// user has seen (by opening the Achievements section).
  static const String prefsKey = 'achievements_seen';

  @override
  Future<DateTime?> build() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      final ms = int.tryParse(raw ?? '');
      return ms == null ? null : DateTime.fromMillisecondsSinceEpoch(ms);
    } catch (_) {
      return null; // storage unavailable → treat as "never seen"
    }
  }

  /// Mark badges earned up to [at] as seen (best-effort persisted). Never moves
  /// the mark backwards, so an out-of-order call cannot resurrect old markers.
  Future<void> markSeen(DateTime at) async {
    final current = state.valueOrNull;
    if (current != null && !at.isAfter(current)) return;
    state = AsyncData(at);
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(prefsKey, at.millisecondsSinceEpoch.toString());
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }
}

/// The signed-in user's Achievements surface (change: add-achievement-badges),
/// grouped by family with tracks collapsed. An AsyncNotifier over the injectable
/// [achievementsServiceProvider], so it is testable without a live backend.
///
/// The display language is **watched**: switching language re-groups with the
/// labels resolved afresh. The seen timestamp is **read** once, so marking the
/// section seen does not tear down the grid the user is currently looking at —
/// the markers clear on the next visit, which is what "since your last visit"
/// means.
@riverpod
class Achievements extends _$Achievements {
  @override
  Future<AchievementsView> build() async {
    final languageCode = ref.watch(appLocaleProvider).languageCode;
    final lastSeen = await ref.read(achievementsSeenProvider.future);
    final badges = await ref
        .read(achievementsServiceProvider)
        .getAchievements();
    return groupAchievements(
      badges,
      languageCode: languageCode,
      lastSeen: lastSeen,
    );
  }

  /// Reload the grid (pull-to-refresh / retry after an error). A failure lands in
  /// the state (`AsyncValue.guard`), never thrown.
  Future<void> refresh() async {
    final languageCode = ref.read(appLocaleProvider).languageCode;
    state = const AsyncLoading();
    state = await AsyncValue.guard(() async {
      final lastSeen = await ref.read(achievementsSeenProvider.future);
      final badges = await ref
          .read(achievementsServiceProvider)
          .getAchievements();
      return groupAchievements(
        badges,
        languageCode: languageCode,
        lastSeen: lastSeen,
      );
    });
  }
}
