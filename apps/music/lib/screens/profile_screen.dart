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

import '../l10n/gen/app_localizations.dart';
import '../services/profile_service.dart';
import '../state/play_activity_notifier.dart';
import '../state/profile_notifier.dart';
import '../state/session_notifier.dart';
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
    final profile = ref.watch(playerProfileProvider(targetId));

    return profile.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      // Fail-closed on the server: a private/ineligible target reads as an error.
      error: (_, _) => Center(child: Text(l10n.profileUnavailable)),
      data: (p) => ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _Header(profile: p),
          const SizedBox(height: 24),
          _ActivitySection(targetId: targetId),
          if (isSelf) ...[
            const SizedBox(height: 24),
            _VisibilityResultListener(targetId: targetId),
            _VisibilityControl(current: p.visibility),
          ],
        ],
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.profile});

  final PlayerProfile profile;

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

/// The self-only visibility control + neutral age gate.
class _VisibilityControl extends ConsumerWidget {
  const _VisibilityControl({required this.current});

  final String current;

  // Two effective states today: Private (hidden from others) and Public (other
  // signed-in players can see the profile). A "limited"/followers-only tier is
  // reserved for later and intentionally not surfaced until it has real behavior.
  static const _options = ['private', 'public'];

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final isPublic = current == 'public';
    String label(String v) => v == 'private'
        ? l10n.profileVisibilityPrivate
        : l10n.profileVisibilityPublic;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.profileVisibility,
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        SegmentedButton<String>(
          key: const Key('profile-visibility'),
          segments: [
            for (final v in _options)
              ButtonSegment(value: v, label: Text(label(v))),
          ],
          // A stored "limited" (legacy) falls back to the Private control.
          selected: {isPublic ? 'public' : 'private'},
          onSelectionChanged: (sel) => _select(context, ref, sel.first, l10n),
        ),
        const SizedBox(height: 8),
        // Explain the current state either way (the feedback: it wasn't clear).
        Text(
          isPublic ? l10n.profilePublicHint : l10n.profileGoPublicHint,
          style: Theme.of(context).textTheme.bodySmall,
        ),
      ],
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
