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
import 'package:grpc/grpc.dart';

import '../analytics/usage_actions.dart';
import '../l10n/gen/app_localizations.dart';
import '../services/profile_service.dart';
import '../state/coaching_notifier.dart';
import '../state/play_activity_notifier.dart';
import '../state/profile_notifier.dart';
import '../state/session_notifier.dart';
import '../state/usage_consent.dart';
import '../state/usage_tracking_notifier.dart';
import '../widgets/coach_mark.dart';
import '../widgets/curator_rewards_section.dart';
import '../widgets/play_heatmap.dart';

/// A player's profile (change: add-play-activity-profile). Reuses the play
/// heatmap and shows only the allow-listed public fields (handle/display name,
/// activity, songs played). Passing no [userId] shows the signed-in user's own
/// profile, which additionally exposes the visibility control + age gate.
class ProfileScreen extends ConsumerWidget {
  const ProfileScreen({super.key, this.userId});

  /// The player to show; `null` = the signed-in user's own profile.
  final String? userId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final selfId = ref.watch(currentUserIdProvider);
    final targetId = userId ?? selfId;
    final isSelf = targetId != null && targetId == selfId;

    return Scaffold(
      appBar: AppBar(title: Text(l10n.profileTitle)),
      // Respect the display cutouts (notch/camera, esp. the side inset in
      // landscape and the home indicator), like the other screens.
      body: SafeArea(
        child: targetId == null
            ? Center(child: Text(l10n.profileUnavailable))
            : _ProfileBody(targetId: targetId, isSelf: isSelf),
      ),
    );
  }
}

class _ProfileBody extends ConsumerWidget {
  const _ProfileBody({required this.targetId, required this.isSelf});

  final String targetId;
  final bool isSelf;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    // Usage telemetry (change: add-feature-usage-analytics): record a view of
    // ANOTHER player's profile once it loads (never a self-view). A listener, not
    // a build-body service call — the notifier gates emission on consent.
    if (!isSelf) {
      ref.listen(playerProfileProvider(targetId), (_, next) {
        if (next is AsyncData) {
          ref
              .read(usageTrackingNotifierProvider.notifier)
              .record(UsageActions.profileView, subjectId: targetId);
        }
      });
    }
    final profile = ref.watch(playerProfileProvider(targetId));

    return profile.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      // Fail-closed on the server: a private/ineligible target reads as an error.
      error: (_, _) => Center(child: Text(l10n.profileUnavailable)),
      data: (p) => ListView(
        padding: const EdgeInsets.all(16),
        children: [
          // The visibility control lives in the header now (change: profile-ux),
          // to the right of the pseudo; the self-only side-effect listener sits
          // alongside it.
          if (isSelf) _VisibilityResultListener(targetId: targetId),
          _Header(profile: p, isSelf: isSelf),
          if (isSelf) ...[
            // One-time hint explaining what making a profile public means, next
            // to the control that does it (change: add-welcome-onboarding, D4).
            // The age gate itself stays a required step, not a dismissible hint.
            const SizedBox(height: 12),
            const CoachHintCallout(
              hint: CoachHint.goingPublic,
              icon: Icons.public,
            ),
            // Usage-analytics consent grouped with visibility: both are
            // profile-level privacy settings (moved off the account menu).
            const _UsageConsentToggle(),
          ],
          const SizedBox(height: 24),
          _ActivitySection(targetId: targetId),
          if (isSelf) ...[
            // Curator rewards live here now (change: add-curation-rewards) — the
            // signed-in user's own standing, integrated into their profile rather
            // than a separate screen. The GetCuratorRewards RPC is caller-scoped,
            // so it is shown only on the OWN profile.
            const SizedBox(height: 24),
            // First visit to the rewards surface: what points, badges and the
            // shop are — through the same one-time coaching mechanism.
            const CoachHintCallout(
              hint: CoachHint.rewards,
              icon: Icons.emoji_events_outlined,
            ),
            const CuratorRewardsSection(),
          ],
        ],
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.profile, this.isSelf = false});

  final PlayerProfile profile;
  final bool isSelf;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final name = profile.displayName ?? profile.handle;
    return Row(
      children: [
        const CircleAvatar(radius: 28, child: Icon(Icons.person)),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (name != null) Text(name, style: theme.textTheme.titleLarge),
              if (profile.handle != null)
                Text('@${profile.handle}', style: theme.textTheme.bodyMedium),
            ],
          ),
        ),
        // Compact public/private toggle to the right of the pseudo (self-only).
        if (isSelf) ...[
          const SizedBox(width: 12),
          _VisibilityToggle(current: profile.visibility),
        ],
      ],
    );
  }
}

class _ActivitySection extends ConsumerWidget {
  const _ActivitySection({required this.targetId});

  final String targetId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final activity = ref.watch(playActivityProvider(targetId));
    return activity.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => const SizedBox.shrink(),
      data: (a) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          PlayHeatmap(activity: a),
          const SizedBox(height: 12),
          Text(
            l10n.profileSongsPlayed(a.totalSessions),
            style: Theme.of(context).textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }
}

/// A compact self-only public/private visibility toggle, shown to the right of
/// the pseudo (change: profile-ux). Tapping it flips the state — going public
/// runs the neutral age gate first. Its tooltip explains the current state, so
/// no separate hint line is needed (keeps the header uncluttered).
class _VisibilityToggle extends ConsumerWidget {
  const _VisibilityToggle({required this.current});

  final String current;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final isPublic = current == 'public';
    return Tooltip(
      message: isPublic ? l10n.profilePublicHint : l10n.profileGoPublicHint,
      child: OutlinedButton.icon(
        key: const Key('profile-visibility'),
        icon: Icon(isPublic ? Icons.public : Icons.lock_outline, size: 18),
        label: Text(
          isPublic
              ? l10n.profileVisibilityPublic
              : l10n.profileVisibilityPrivate,
        ),
        onPressed: () =>
            _select(context, ref, isPublic ? 'private' : 'public', l10n),
      ),
    );
  }

  Future<void> _select(
    BuildContext context,
    WidgetRef ref,
    String value,
    AppLocalizations l10n,
  ) async {
    final controller = ref.read(profileVisibilityControllerProvider.notifier);
    if (value != 'public') {
      await controller.setVisibility(value);
      return;
    }
    // Going public: run the neutral age gate (asks the date of birth, sent once).
    final dob = await _ageGate(context, l10n);
    if (dob == null) return; // cancelled
    await controller.setVisibility('public', dateOfBirth: dob);
  }

  /// Neutral age gate: explain, then pick a date of birth. Returns null if the
  /// user cancels either step.
  Future<DateTime?> _ageGate(
    BuildContext context,
    AppLocalizations l10n,
  ) async {
    final proceed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text(l10n.profileAgeGateTitle),
        content: Text(l10n.profileAgeGateBody),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: Text(l10n.cancel),
          ),
          TextButton(
            key: const Key('age-gate-continue'),
            onPressed: () => Navigator.of(dialogContext).pop(true),
            child: Text(l10n.profileAgeGateContinue),
          ),
        ],
      ),
    );
    if (proceed != true || !context.mounted) return null;
    final now = DateTime.now();
    return showDatePicker(
      context: context,
      // A neutral adult default — NOT the exact minimum-age boundary, so a
      // careless confirm doesn't land one day short of eligibility.
      initialDate: DateTime(now.year - 25, now.month, now.day),
      firstDate: DateTime(1900),
      lastDate: now,
      helpText: l10n.profileAgeGateTitle,
    );
  }
}

/// Self-only usage-analytics consent (change: add-feature-usage-analytics),
/// grouped here with the visibility control since both are profile-level
/// privacy settings (moved off the account menu).
class _UsageConsentToggle extends ConsumerWidget {
  const _UsageConsentToggle();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final enabled = ref.watch(usageConsentProvider);
    return SwitchListTile.adaptive(
      key: const Key('profile-usage-consent'),
      contentPadding: EdgeInsets.zero,
      title: Text(l10n.usageAnalyticsSetting),
      value: enabled,
      onChanged: (value) => ref.read(usageConsentProvider.notifier).set(value),
    );
  }
}

/// Isolates the visibility-action side effects (CLAUDE.md rule): on success it
/// refreshes the profile; on failure it shows a friendly, localized message —
/// the raw gRPC error is never surfaced (a `failedPrecondition` means under-age).
class _VisibilityResultListener extends ConsumerWidget {
  const _VisibilityResultListener({required this.targetId});

  final String targetId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    ref.listen(profileVisibilityControllerProvider, (_, next) {
      next.whenOrNull(
        data: (v) {
          if (v == null) return; // initial state
          ref.invalidate(playerProfileProvider(targetId));
        },
        error: (error, _) {
          final tooYoung =
              error is GrpcError && error.code == StatusCode.failedPrecondition;
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(
                tooYoung ? l10n.profileTooYoung : l10n.profileVisibilityError,
              ),
            ),
          );
        },
      );
    });
    return const SizedBox.shrink();
  }
}
