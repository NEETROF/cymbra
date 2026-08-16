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
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/plans.pbgrpc.dart' as plans;
import 'app_platform.dart';
import 'grpc_client.dart';

part 'plan_service.freezed.dart';
part 'plan_service.g.dart';

// --- Domain view models ------------------------------------------------------
//
// Freezed models mapped from the wire types (change: add-premium-subscription),
// so the notifiers, widgets and their tests never touch the generated proto.

/// A purchase / management channel.
enum PlanChannel { apple, google, web }

/// One active beta membership.
@freezed
abstract class BetaMembershipView with _$BetaMembershipView {
  const factory BetaMembershipView({
    required String campaignKey,
    required String campaignName,

    /// `premium_trial` | `feature`.
    required String kind,
    required DateTime joinedAt,
    DateTime? endsAt,
  }) = _BetaMembershipView;
}

/// The caller's plan as the server decided it (design D10): the plan, its
/// source and end, the running trial, the betas, and what THIS platform may do.
@freezed
abstract class PlanSnapshotView with _$PlanSnapshotView {
  const PlanSnapshotView._();

  const factory PlanSnapshotView({
    /// `free` | `premium`.
    required String plan,

    /// `apple` | `google` | `web` | `code` | `admin` — absent when free.
    String? source,
    DateTime? endsAt,

    /// The plan ends without renewal (trial, cancelled, comp): show the
    /// "rights end on [date]" line.
    @Default(false) bool endsWithoutRenewal,
    String? trialCampaignKey,
    String? trialCampaignName,
    DateTime? trialEndsAt,
    @Default([]) List<BetaMembershipView> betas,

    /// Where the subscription is managed (absent for free / trial / admin).
    PlanChannel? managedOn,

    /// A purchase may be started from this platform right now.
    @Default(false) bool canPurchaseHere,

    /// The channel this platform buys through (set with [canPurchaseHere]).
    PlanChannel? purchaseChannel,

    /// Store / MoR product ids to offer (prices come from the store).
    @Default([]) List<String> products,

    /// Unlock keys the effective plan grants (`catalog.unlimited`, …).
    @Default([]) List<String> unlocks,
  }) = _PlanSnapshotView;

  /// The free plan, no memberships (signed-out / plans disabled).
  static const free = PlanSnapshotView(plan: 'free');

  bool get isPremium => plan == 'premium';

  /// Whether the effective plan grants [unlock] (e.g. `offline.cache`).
  bool grants(String unlock) => unlocks.contains(unlock);

  /// Premium via a running trial (a `code` row with a trial campaign).
  bool get isTrial => isPremium && trialCampaignKey != null && source == 'code';
}

/// Outcome of a store purchase report / restore: the refreshed plan.
@freezed
abstract class PurchaseReportView with _$PurchaseReportView {
  const factory PurchaseReportView({required PlanSnapshotView plan}) =
      _PurchaseReportView;
}

// --- Conversion --------------------------------------------------------------

DateTime? _date(String s) => s.isEmpty ? null : DateTime.tryParse(s)?.toUtc();

PlanChannel? _channel(plans.Channel c) => switch (c) {
  plans.Channel.CHANNEL_APPLE => PlanChannel.apple,
  plans.Channel.CHANNEL_GOOGLE => PlanChannel.google,
  plans.Channel.CHANNEL_WEB => PlanChannel.web,
  _ => null,
};

plans.Channel _toChannel(PlanChannel c) => switch (c) {
  PlanChannel.apple => plans.Channel.CHANNEL_APPLE,
  PlanChannel.google => plans.Channel.CHANNEL_GOOGLE,
  PlanChannel.web => plans.Channel.CHANNEL_WEB,
};

plans.Platform _toPlatform(AppPlatform p) => switch (p) {
  AppPlatform.ios => plans.Platform.PLATFORM_IOS,
  AppPlatform.macos => plans.Platform.PLATFORM_MACOS,
  AppPlatform.android => plans.Platform.PLATFORM_ANDROID,
  AppPlatform.linux => plans.Platform.PLATFORM_LINUX,
  AppPlatform.windows => plans.Platform.PLATFORM_WINDOWS,
  AppPlatform.web => plans.Platform.PLATFORM_WEB,
};

PlanSnapshotView toPlanView(plans.GetMyPlanResponse r) => PlanSnapshotView(
  plan: r.plan,
  source: r.hasSource() ? r.source : null,
  endsAt: r.hasEndsAt() ? _date(r.endsAt) : null,
  endsWithoutRenewal: r.endsWithoutRenewal,
  trialCampaignKey: r.hasTrialCampaignKey() ? r.trialCampaignKey : null,
  trialCampaignName: r.hasTrialCampaignName() ? r.trialCampaignName : null,
  trialEndsAt: r.hasTrialEndsAt() ? _date(r.trialEndsAt) : null,
  betas: [
    for (final b in r.betas)
      BetaMembershipView(
        campaignKey: b.campaignKey,
        campaignName: b.campaignName,
        kind: b.kind,
        joinedAt: _date(b.joinedAt) ?? DateTime.now().toUtc(),
        endsAt: b.hasEndsAt() ? _date(b.endsAt) : null,
      ),
  ],
  managedOn: _channel(r.managedOn),
  canPurchaseHere: r.canPurchaseHere,
  purchaseChannel: r.canPurchaseHere ? _channel(r.purchaseChannel) : null,
  products: List.unmodifiable(r.products),
  unlocks: List.unmodifiable(r.unlocks),
);

// --- Service seam ------------------------------------------------------------

/// Seam over the backend `PlanService` (change: add-premium-subscription): the
/// caller's plan for this platform, store purchase reports (also restore), and
/// the web checkout URL. Every call is bearer-authenticated; the production impl
/// refreshes transparently on `UNAUTHENTICATED`. Tests override the provider
/// with a mockito mock. Failures throw `AuthException`.
abstract class PlanService {
  /// The caller's plan + what [platform] may purchase.
  Future<PlanSnapshotView> getMyPlan(AppPlatform platform);

  /// Report a store purchase for server-side verification ([payload] = Apple
  /// signed transaction JWS | Google purchase token). Returns the refreshed plan.
  Future<PurchaseReportView> reportStorePurchase({
    required PlanChannel channel,
    required String payload,
    required String productId,
  });

  /// A hosted merchant-of-record checkout URL for [productId] (desktop / web).
  Future<Uri> createWebCheckout(String productId);
}

/// Production [PlanService] over the generated `PlanServiceClient`.
class GrpcPlanService implements PlanService {
  GrpcPlanService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = plans.PlanServiceClient(channel),
       _authed = authed;

  final plans.PlanServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<PlanSnapshotView> getMyPlan(AppPlatform platform) => _authed(
    (bearer) async => toPlanView(
      await _client.getMyPlan(
        plans.GetMyPlanRequest(platform: _toPlatform(platform)),
        options: bearerOptions(bearer),
      ),
    ),
  );

  @override
  Future<PurchaseReportView> reportStorePurchase({
    required PlanChannel channel,
    required String payload,
    required String productId,
  }) => _authed((bearer) async {
    final resp = await _client.reportStorePurchase(
      plans.ReportStorePurchaseRequest(
        channel: _toChannel(channel),
        payload: payload,
        productId: productId,
      ),
      options: bearerOptions(bearer),
    );
    return PurchaseReportView(plan: toPlanView(resp.plan));
  });

  @override
  Future<Uri> createWebCheckout(String productId) => _authed((bearer) async {
    final resp = await _client.createWebCheckout(
      plans.CreateWebCheckoutRequest(productId: productId),
      options: bearerOptions(bearer),
    );
    return Uri.parse(resp.checkoutUrl);
  });
}

/// Production plan-service provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
PlanService planService(Ref ref) => GrpcPlanService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
