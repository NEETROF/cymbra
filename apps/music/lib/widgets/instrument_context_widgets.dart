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
import '../state/drums_access.dart';
import '../state/instrument_context.dart';
import '../theme/cymbra_theme.dart';

/// The permanent context switcher in the home header (change:
/// add-instrument-context): present ONLY while drums are visible — everyone
/// else sees today's home, with no new control and no question. A sticky
/// context is only safe because correcting it is always one tap away, in
/// view, never behind a settings screen.
class InstrumentSwitcher extends ConsumerWidget {
  const InstrumentSwitcher({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final drumsVisible = ref.watch(drumsEnabledProvider);
    if (!drumsVisible) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    final current = ref.watch(effectiveInstrumentContextProvider);
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: SegmentedButton<AppInstrument>(
        key: const Key('instrument-switcher'),
        style: ButtonStyle(
          visualDensity: VisualDensity.compact,
          backgroundColor: WidgetStateProperty.resolveWith(
            (s) => s.contains(WidgetState.selected)
                ? CymbraColors.primaryContainer
                : CymbraColors.surfaceContainerHigh,
          ),
          // Explicit: the surrounding MaterialApp theme is light, so the
          // default label colour disappears on these dark surfaces.
          foregroundColor: WidgetStateProperty.resolveWith(
            (s) => s.contains(WidgetState.selected)
                ? CymbraColors.onSurface
                : CymbraColors.onSurfaceVariant,
          ),
        ),
        segments: [
          ButtonSegment(
            value: AppInstrument.keyboard,
            label: Text(l10n.instrumentKeyboard),
            icon: const Icon(Icons.piano, size: 18),
          ),
          ButtonSegment(
            value: AppInstrument.drums,
            label: Text(l10n.instrumentDrums),
            icon: const Icon(Icons.album_outlined, size: 18),
          ),
        ],
        selected: {current},
        onSelectionChanged: (s) =>
            ref.read(instrumentContextProvider.notifier).select(s.first),
        showSelectedIcon: false,
      ),
    );
  }
}

/// Offers the one-time instrument choice, on the home only (change:
/// add-instrument-context): the next time the user is on — or arrives at —
/// the home while drums are visible. One rule for both phases: after sign-in
/// while beta-scoped, at first launch once the flag is global. A visibility
/// flip landing mid-play defers to the next arrival here — never a dialog
/// over the player.
///
/// A dedicated listener widget per the layering rules: the side effect
/// (showing a dialog) lives here, not in a build method.
class InstrumentChoiceListener extends ConsumerStatefulWidget {
  const InstrumentChoiceListener({required this.child, super.key});

  final Widget child;

  @override
  ConsumerState<InstrumentChoiceListener> createState() =>
      _InstrumentChoiceListenerState();
}

class _InstrumentChoiceListenerState
    extends ConsumerState<InstrumentChoiceListener> {
  bool _showing = false;
  bool _rearmed = false;

  void _maybeOffer() {
    _rearmed = false;
    if (!mounted || _showing) return;
    final record = ref.read(instrumentContextProvider);
    // Never decide before the stored record is read back: pre-hydration the
    // defaults claim "never offered" and every launch would re-prompt. The
    // hydration listener in build re-checks the moment it lands.
    if (!record.hydrated) return;
    if (record.choiceOffered || !ref.read(drumsEnabledProvider)) return;
    // Only while the home route is the current one: a flip landing while the
    // player (or any pushed screen) is on top waits for the return here.
    // Nothing rebuilds this widget when that route pops, so re-arm a
    // post-frame check — it requests no frame itself (free while idle) and
    // the return transition guarantees frames to fire on.
    if (!(ModalRoute.of(context)?.isCurrent ?? true)) {
      _rearm();
      return;
    }

    _showing = true;
    // Offered at most once per installation, whatever is answered — the
    // switcher stays permanently in view, so a dismissed modal loses nothing.
    ref.read(instrumentContextProvider.notifier).markChoiceOffered();
    final l10n = AppLocalizations.of(context);
    showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        key: const Key('instrument-choice-modal'),
        backgroundColor: CymbraColors.surfaceContainerLow,
        // Explicit colours: the app theme is light, so the dialog defaults
        // would put near-black text on this dark surface.
        title: Text(
          l10n.instrumentChoiceTitle,
          style: const TextStyle(
            color: CymbraColors.onSurface,
            fontWeight: FontWeight.w700,
          ),
        ),
        content: Text(
          l10n.instrumentChoiceBody,
          style: const TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
        actions: [
          TextButton(
            key: const Key('instrument-choice-keyboard'),
            onPressed: () {
              ref
                  .read(instrumentContextProvider.notifier)
                  .select(AppInstrument.keyboard);
              Navigator.of(dialogContext).pop();
            },
            child: Text(l10n.instrumentKeyboard),
          ),
          FilledButton(
            key: const Key('instrument-choice-drums'),
            onPressed: () {
              ref
                  .read(instrumentContextProvider.notifier)
                  .select(AppInstrument.drums);
              Navigator.of(dialogContext).pop();
            },
            child: Text(l10n.instrumentDrums),
          ),
        ],
      ),
    ).whenComplete(() => _showing = false);
  }

  void _rearm() {
    if (_rearmed) return;
    _rearmed = true;
    WidgetsBinding.instance.addPostFrameCallback((_) => _maybeOffer());
  }

  @override
  Widget build(BuildContext context) {
    // React to visibility arriving (the flag snapshot resolves async), and
    // check on every (re)build — which is what fires on returning to the
    // home after a deferred flip.
    ref.listen(drumsEnabledProvider, (_, visible) {
      if (visible) {
        WidgetsBinding.instance.addPostFrameCallback((_) => _maybeOffer());
      }
    });
    // Hydration landing is the other async arrival the offer waits on.
    ref.listen(instrumentContextProvider.select((s) => s.hydrated), (
      _,
      hydrated,
    ) {
      if (hydrated) {
        WidgetsBinding.instance.addPostFrameCallback((_) => _maybeOffer());
      }
    });
    WidgetsBinding.instance.addPostFrameCallback((_) => _maybeOffer());
    return widget.child;
  }
}

/// The drums-context empty state (change: add-instrument-context): the
/// context is in force but nothing is reachable for it — an explicit
/// invitation naming the cause and offering to switch, never a bare empty
/// screen. The courses surface under drums reuses it too.
class DrumsEmptyInvitation extends ConsumerWidget {
  const DrumsEmptyInvitation({super.key, required this.onBrowseCatalog});

  /// Opens the score hub. Required rather than optional: this state is where a
  /// drummer with nothing saved lands, so an invitation whose only action is
  /// "give up on drums" would be the common case, not the edge one. The screen
  /// owns the navigation (and the signed-out sign-in detour); the widget only
  /// offers it.
  final VoidCallback onBrowseCatalog;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(
              Icons.album_outlined,
              size: 56,
              color: CymbraColors.outline,
            ),
            const SizedBox(height: 16),
            Text(
              l10n.drumsEmptyTitle,
              textAlign: TextAlign.center,
              style: const TextStyle(
                color: CymbraColors.onSurface,
                fontSize: 18,
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              l10n.drumsEmptyBody,
              textAlign: TextAlign.center,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
            const SizedBox(height: 20),
            // The catalog leads. Switching back to the keyboard stays offered
            // but reads as the fallback it is.
            FilledButton.icon(
              key: const Key('drums-empty-browse'),
              icon: const Icon(Icons.search),
              label: Text(l10n.drumsEmptyBrowse),
              onPressed: onBrowseCatalog,
            ),
            const SizedBox(height: 4),
            TextButton.icon(
              key: const Key('drums-empty-switch'),
              icon: const Icon(Icons.piano),
              label: Text(l10n.drumsEmptySwitch),
              onPressed: () => ref
                  .read(instrumentContextProvider.notifier)
                  .select(AppInstrument.keyboard),
            ),
          ],
        ),
      ),
    );
  }
}
