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

/// The caller's freemium daily-access state for the current SERVER day (change:
/// add-score-daily-access-rewards, design D3) — carried by the catalog bytes
/// fetch and by the dedicated state read. Immutable wire-shaped value (like
/// `CatalogHit`); the notifiers hold it.
class CatalogAccessState {
  /// The gate is on for this caller. `false` = no quota (off, exempt, subscriber,
  /// contributor): every catalog open serves and no chip is shown.
  final bool enabled;

  /// This open was refused by the daily quota (bytes fetch only).
  final bool locked;

  /// Distinct free opens per day.
  final int freeQuota;

  /// Consumed today (paid slots are not counted).
  final int freeUsed;

  /// Next server-day rollover, epoch ms — what the chip counts down to.
  final int resetsAtMs;

  /// Points for one extra piece today.
  final int daySlotCost;

  /// The caller's spendable points balance.
  final int spendableBalance;

  /// `has_active_subscription` (billing seam — false today).
  final bool subscriber;

  /// The moment the future subscription offer would be shown.
  final bool upsell;

  /// Pieces open for the caller today (free + paid).
  final List<String> openedToday;

  /// The subset bought with points.
  final List<String> paidToday;

  const CatalogAccessState({
    required this.enabled,
    required this.locked,
    required this.freeQuota,
    required this.freeUsed,
    required this.resetsAtMs,
    required this.daySlotCost,
    required this.spendableBalance,
    required this.subscriber,
    required this.upsell,
    this.openedToday = const [],
    this.paidToday = const [],
  });

  /// Free opens still available today (never negative).
  int get freeLeft => (freeQuota - freeUsed).clamp(0, freeQuota);

  /// Whether the caller can afford a day-slot right now.
  bool get canAffordDaySlot => spendableBalance >= daySlotCost;

  /// Whether [catalogId] is already open for the caller today.
  bool isOpenToday(String catalogId) => openedToday.contains(catalogId);

  /// Time left until the quota resets, measured from [now] (never negative).
  Duration untilReset(DateTime now) {
    final reset = DateTime.fromMillisecondsSinceEpoch(resetsAtMs, isUtc: true);
    final d = reset.difference(now.toUtc());
    return d.isNegative ? Duration.zero : d;
  }

  CatalogAccessState copyWith({bool? locked}) => CatalogAccessState(
    enabled: enabled,
    locked: locked ?? this.locked,
    freeQuota: freeQuota,
    freeUsed: freeUsed,
    resetsAtMs: resetsAtMs,
    daySlotCost: daySlotCost,
    spendableBalance: spendableBalance,
    subscriber: subscriber,
    upsell: upsell,
    openedToday: openedToday,
    paidToday: paidToday,
  );

  @override
  bool operator ==(Object other) =>
      other is CatalogAccessState &&
      other.enabled == enabled &&
      other.locked == locked &&
      other.freeQuota == freeQuota &&
      other.freeUsed == freeUsed &&
      other.resetsAtMs == resetsAtMs &&
      other.daySlotCost == daySlotCost &&
      other.spendableBalance == spendableBalance &&
      other.subscriber == subscriber &&
      other.upsell == upsell &&
      _sameList(other.openedToday, openedToday) &&
      _sameList(other.paidToday, paidToday);

  @override
  int get hashCode => Object.hash(
    enabled,
    locked,
    freeQuota,
    freeUsed,
    resetsAtMs,
    daySlotCost,
    spendableBalance,
    subscriber,
    upsell,
    Object.hashAll(openedToday),
    Object.hashAll(paidToday),
  );

  static bool _sameList(List<String> a, List<String> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}
