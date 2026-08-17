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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../services/app_platform.dart';
import '../services/legal_links.dart';
import '../services/plan_service.dart';
import '../services/store_client.dart';
import '../state/plan_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/plan_listener.dart';

/// Push the plan screen (plan status + paywall) from any locked surface or the
/// account menu.
void openPlanScreen(BuildContext context) => Navigator.of(
  context,
).push(MaterialPageRoute<void>(builder: (_) => const PlanScreen()));

/// Apple / Google subscription-management deep links (store rows).
Uri manageUriFor(PlanChannel channel) => switch (channel) {
  PlanChannel.apple => Uri.parse(
    'https://apps.apple.com/account/subscriptions',
  ),
  PlanChannel.google => Uri.parse(
    'https://play.google.com/store/account/subscriptions',
  ),
  // The web portal URL is provider-hosted and fetched at request time; until
  // that call is wired the site's account page is the stable entry.
  PlanChannel.web => Uri.parse('https://cymbra.app/account'),
};

/// The plan screen (change: add-premium-subscription, spec `music-premium-
/// paywall`): the current plan (source, end / renewal, "rights end on [date]"
/// when the plan will not renew, betas), the channel-aware purchase entry for
/// THIS platform (App Store / Play flow, or the web checkout in the browser),
/// restore on store builds, and a manage action opening the right portal.
/// Store builds carry no external link, no code field and no discount copy —
/// everything shown is decided server-side in the plan snapshot.
class PlanScreen extends ConsumerWidget {
  const PlanScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final plan = ref.watch(planProvider);
    final flow = ref.watch(purchaseFlowProvider);
    final platform = ref.watch(appPlatformProvider);
    return PlanListener(
      child: Scaffold(
        backgroundColor: CymbraColors.background,
        // The AppBar only pads the TOP inset; in landscape the sensor housing sits
        // on the side and would cover the back button — pad the sides too. Same for
        // the body: the ListView has its own padding, so it would not apply the
        // MediaQuery insets by itself.
        appBar: PreferredSize(
          preferredSize: const Size.fromHeight(kToolbarHeight),
          child: SafeArea(
            top: false,
            bottom: false,
            child: AppBar(title: Text(l10n.planTitle)),
          ),
        ),
        body: SafeArea(
          top: false,
          child: RefreshIndicator(
            onRefresh: () => ref.read(planProvider.notifier).refresh(),
            child: ListView(
              key: const Key('plan-screen'),
              padding: const EdgeInsets.all(16),
              children: [
                switch (plan) {
                  AsyncData(:final value) => _StatusCard(
                    snapshot: value,
                    platform: platform,
                  ),
                  AsyncError() => _StatusCard(
                    snapshot: PlanSnapshotView.free,
                    platform: platform,
                  ),
                  _ => const Center(
                    child: Padding(
                      padding: EdgeInsets.all(24),
                      child: CircularProgressIndicator(),
                    ),
                  ),
                },
                const SizedBox(height: 16),
                if (plan.valueOrNull case final snap?) ...[
                  if (snap.betas.isNotEmpty) ...[
                    _BetasCard(betas: snap.betas),
                    const SizedBox(height: 16),
                  ],
                  _BenefitsCard(),
                  const SizedBox(height: 16),
                  if (snap.canPurchaseHere)
                    _PurchaseCard(
                      snapshot: snap,
                      platform: platform,
                      busy: flow.busy,
                    )
                  else if (snap.managedOn != null)
                    _ManagedElsewhereCard(channel: snap.managedOn!),
                  if (platform.isStoreBuild) ...[
                    const SizedBox(height: 8),
                    TextButton(
                      key: const Key('plan-restore'),
                      onPressed: flow.busy
                          ? null
                          : () => ref
                                .read(purchaseFlowProvider.notifier)
                                .restore(),
                      child: Text(l10n.planRestore),
                    ),
                  ],
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

String _fmtDate(BuildContext context, DateTime d) =>
    MaterialLocalizations.of(context).formatMediumDate(d.toLocal());

String _channelLabel(AppLocalizations l10n, PlanChannel c) => switch (c) {
  PlanChannel.apple => l10n.planChannelApple,
  PlanChannel.google => l10n.planChannelGoogle,
  PlanChannel.web => l10n.planChannelWeb,
};

class _StatusCard extends ConsumerWidget {
  const _StatusCard({required this.snapshot, required this.platform});

  final PlanSnapshotView snapshot;
  final AppPlatform platform;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final s = snapshot;
    final title = !s.isPremium
        ? l10n.planFree
        : s.isTrial
        ? l10n.planPremiumTrial
        : l10n.planPremium;
    final lines = <String>[];
    if (s.isPremium) {
      if (s.isTrial && s.trialEndsAt != null) {
        lines.add(l10n.planTrialUntil(_fmtDate(context, s.trialEndsAt!)));
      } else if (s.source != null) {
        lines.add(l10n.planSource(_sourceLabel(l10n, s.source!)));
      }
      final ends = s.endsAt;
      if (ends != null) {
        lines.add(
          s.endsWithoutRenewal
              ? l10n.planRightsEndOn(_fmtDate(context, ends))
              : l10n.planRenewsOn(_fmtDate(context, ends)),
        );
        if (s.endsWithoutRenewal) lines.add(l10n.planWithdrawalWarning);
      }
    } else {
      lines.add(l10n.planFreeBody);
    }
    return Card(
      key: const Key('plan-status'),
      color: CymbraColors.surfaceContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  s.isPremium ? Icons.workspace_premium : Icons.person_outline,
                  color: CymbraColors.primary,
                ),
                const SizedBox(width: 8),
                Text(
                  title,
                  key: const Key('plan-status-title'),
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ],
            ),
            for (final line in lines) ...[
              const SizedBox(height: 6),
              Text(
                line,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            ],
            if (s.managedOn case final channel?) ...[
              const SizedBox(height: 12),
              OutlinedButton.icon(
                key: const Key('plan-manage'),
                onPressed: () => ref
                    .read(legalLinkLauncherProvider)
                    .open(manageUriFor(channel)),
                icon: const Icon(Icons.open_in_new),
                label: Text(l10n.planManageOn(_channelLabel(l10n, channel))),
              ),
            ],
          ],
        ),
      ),
    );
  }

  String _sourceLabel(AppLocalizations l10n, String source) => switch (source) {
    'apple' => l10n.planChannelApple,
    'google' => l10n.planChannelGoogle,
    'web' => l10n.planChannelWeb,
    'code' => l10n.planSourceCode,
    _ => l10n.planSourceAdmin,
  };
}

class _BetasCard extends StatelessWidget {
  const _BetasCard({required this.betas});

  final List<BetaMembershipView> betas;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Card(
      key: const Key('plan-betas'),
      color: CymbraColors.surfaceContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.planBetasTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            for (final b in betas)
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(
                  b.kind == 'premium_trial'
                      ? Icons.card_giftcard
                      : Icons.science_outlined,
                  color: CymbraColors.primary,
                ),
                title: Text(b.campaignName),
                subtitle: Text(
                  b.endsAt != null
                      ? l10n.planBetaJoinedUntil(
                          _fmtDate(context, b.joinedAt),
                          _fmtDate(context, b.endsAt!),
                        )
                      : l10n.planBetaJoined(_fmtDate(context, b.joinedAt)),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _BenefitsCard extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final items = [
      (Icons.library_music, l10n.planBenefitCatalog),
      (Icons.piano, l10n.planBenefitSoundfonts),
      (Icons.offline_pin_outlined, l10n.planBenefitOffline),
      (Icons.cloud_upload_outlined, l10n.planBenefitQuotas),
    ];
    return Card(
      key: const Key('plan-benefits'),
      color: CymbraColors.surfaceContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.planBenefitsTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            for (final (icon, label) in items)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 4),
                child: Row(
                  children: [
                    Icon(icon, size: 20, color: CymbraColors.primary),
                    const SizedBox(width: 10),
                    Expanded(child: Text(label)),
                  ],
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _PurchaseCard extends ConsumerWidget {
  const _PurchaseCard({
    required this.snapshot,
    required this.platform,
    required this.busy,
  });

  final PlanSnapshotView snapshot;
  final AppPlatform platform;
  final bool busy;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final products = snapshot.products;
    return Card(
      key: const Key('plan-purchase'),
      color: CymbraColors.surfaceContainerHigh,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              snapshot.isTrial
                  ? l10n.planKeepPremiumTitle
                  : l10n.planSubscribeTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            if (platform.usesWebCheckout) ...[
              // Desktop / web: the hosted checkout opens in the browser; the
              // store-localised price lives on that page.
              for (final id in products)
                Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: FilledButton.icon(
                    key: Key('plan-buy-$id'),
                    onPressed: busy
                        ? null
                        : () => ref.read(purchaseFlowProvider.notifier).buy(id),
                    icon: const Icon(Icons.open_in_new),
                    label: Text(l10n.planSubscribeWeb(_productLabel(l10n, id))),
                  ),
                ),
              const SizedBox(height: 4),
              TextButton.icon(
                key: const Key('plan-refresh'),
                onPressed: () => ref.read(planProvider.notifier).refresh(),
                icon: const Icon(Icons.refresh),
                label: Text(l10n.planIvePaidRefresh),
              ),
            ] else
              _StoreProducts(products: products, busy: busy),
          ],
        ),
      ),
    );
  }
}

String _productLabel(AppLocalizations l10n, String id) =>
    id.contains('year') ? l10n.planProductYearly : l10n.planProductMonthly;

/// Store builds: the products as the store lists them (localized prices).
class _StoreProducts extends ConsumerWidget {
  const _StoreProducts({required this.products, required this.busy});

  final List<String> products;
  final bool busy;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    // Keyed by a STRING (value equality) — a fresh Set per build would spin up
    // a fresh family member every rebuild and never settle.
    final listing = ref.watch(_storeListingProvider(products.join(',')));
    return switch (listing) {
      AsyncData(:final value) when value.isNotEmpty => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final p in value)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: FilledButton(
                key: Key('plan-buy-${p.id}'),
                onPressed: busy
                    ? null
                    : () => ref.read(purchaseFlowProvider.notifier).buy(p.id),
                child: Text('${_productLabel(l10n, p.id)} · ${p.price}'),
              ),
            ),
        ],
      ),
      AsyncData() => Text(
        l10n.planStoreUnavailable,
        key: const Key('plan-store-unavailable'),
      ),
      AsyncError() => Text(
        l10n.planStoreUnavailable,
        key: const Key('plan-store-unavailable'),
      ),
      _ => const Center(child: CircularProgressIndicator()),
    };
  }
}

final _storeListingProvider = FutureProvider.autoDispose
    .family<List<StoreProduct>, String>((ref, joinedIds) async {
      final store = ref.watch(storeClientProvider);
      if (!await store.isAvailable()) return const [];
      final ids = joinedIds.split(',').where((s) => s.isNotEmpty).toSet();
      if (ids.isEmpty) return const [];
      return store.products(ids);
    });

class _ManagedElsewhereCard extends StatelessWidget {
  const _ManagedElsewhereCard({required this.channel});

  final PlanChannel channel;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Card(
      key: const Key('plan-managed-elsewhere'),
      color: CymbraColors.surfaceContainer,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Text(l10n.planManagedElsewhere(_channelLabel(l10n, channel))),
      ),
    );
  }
}
