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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../layout/device_class.dart';
import '../painters/notation_palette.dart';
import '../painters/partition_painter.dart';
import '../state/notation_notifier.dart';
import '../state/player_notifier.dart';
import '../state/player_preferences.dart';
import '../state/practice_settings_store.dart';
import '../state/score_catalog.dart';
import '../theme/cymbra_theme.dart';

/// Opens the dedicated measure-selection mode over the player (change:
/// add-in-game-measure-selection, D3): pauses playback — the only session
/// mutation before a choice is confirmed — then pushes the full-screen route.
/// A no-op when the piece has no engraved notation or no measure table (the
/// demo score), where there is nothing to pick on.
void openMeasureSelect(BuildContext context, WidgetRef ref) {
  if (!ref.read(notationProvider).hasDocument) return;
  if (ref.read(playerProvider).measureStartMs.isEmpty) return;
  ref.read(playerProvider.notifier).setPlaying(false);
  Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => const MeasureSelectScreen()));
}

/// Full-screen measure-selection mode (change: add-in-game-measure-selection):
/// the engraved score scrolling vertically — no keyboard, no playhead — under
/// its own title bar. Tap a first and a last measure to draft a range; the
/// draft is **local** to this screen and the live session is only touched by
/// the title-bar actions: Confirm applies it as the active practice range,
/// Whole-piece clears back to a full run, Cancel/back leaves everything as on
/// entry. Pushed over the player screen, which stays mounted underneath (so
/// the auto-dispose player state, MIDI and audio session all stay alive).
class MeasureSelectScreen extends ConsumerStatefulWidget {
  const MeasureSelectScreen({super.key});

  @override
  ConsumerState<MeasureSelectScreen> createState() =>
      _MeasureSelectScreenState();
}

class _MeasureSelectScreenState extends ConsumerState<MeasureSelectScreen> {
  final ScrollController _scroll = ScrollController();

  /// Per-measure hit rectangles, refilled by the painter every frame, so a tap
  /// maps to the measure actually engraved under it.
  final List<MeasureHit> _hits = [];

  /// The drafted range (0-based, inclusive), complete once both are set.
  int? _start;
  int? _end;

  /// First measure of an in-progress two-tap pick; null when none.
  int? _pendingStart;

  /// Whether the user tapped at all — an async saved-settings pre-fill must
  /// never override a pick already under way.
  bool _touched = false;

  @override
  void initState() {
    super.initState();
    // Draft pre-fill (D5): the active range when the run is already selective,
    // else this score's saved practice settings, else empty.
    final data = ref.read(playerProvider);
    if (data.isSelectiveRun) {
      _start = data.practiceStartMeasure;
      _end = data.practiceEndMeasure;
    } else {
      unawaited(_prefillFromSaved());
    }
  }

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  /// Pre-fills the draft from the per-score saved practice settings, clamped to
  /// the piece's current measure count — the pre-fill the setup modal used to
  /// do, relocated here (D6). Skipped when the user has started picking.
  Future<void> _prefillFromSaved() async {
    final scoreKey = pieceIdentityOf(
      ref.read(selectedScoreProvider),
      ref.read(playerProvider).title,
    );
    final saved = await ref.read(practiceSettingsStoreProvider).load(scoreKey);
    if (!mounted || _touched) return;
    final applied = saved?.clampedTo(
      ref.read(playerProvider).practiceMeasureCount,
    );
    if (applied == null) return;
    setState(() {
      _start = applied.startMeasure;
      _end = applied.endMeasure;
    });
  }

  /// Tap-to-draft: the first tap picks the start, the second the end
  /// (order-normalized); tapping again begins a new draft from the tapped
  /// measure. A tap that misses every measure (header, gutter) is ignored.
  void _onScoreTap(TapUpDetails details) {
    final measure = measureAtPosition(_hits, details.localPosition);
    if (measure == null) return;
    _touched = true;
    final pending = _pendingStart;
    setState(() {
      if (pending == null) {
        _pendingStart = measure;
        _start = null;
        _end = null;
      } else {
        _pendingStart = null;
        _start = pending <= measure ? pending : measure;
        _end = pending <= measure ? measure : pending;
      }
    });
  }

  /// The range currently tinted on the score: the in-progress first pick alone
  /// (so it reads back immediately), else the complete draft.
  ({int start, int end})? get _highlight {
    final pending = _pendingStart;
    if (pending != null) return (start: pending, end: pending);
    final start = _start;
    final end = _end;
    return start != null && end != null ? (start: start, end: end) : null;
  }

  /// Title-bar label for the current draft state (1-based bar numbers).
  String _title(AppLocalizations l10n) {
    final pending = _pendingStart;
    if (pending != null) return l10n.measureSelectPending(pending + 1);
    final start = _start;
    final end = _end;
    if (start != null && end != null) {
      return l10n.measureSelectRange(start + 1, end + 1);
    }
    return l10n.measureSelectTitle;
  }

  void _confirm() {
    final start = _start;
    final end = _end;
    if (start == null || end == null) return;
    ref.read(playerProvider.notifier).setPracticeRange(start, end);
    Navigator.of(context).pop();
  }

  void _wholePiece() {
    ref.read(playerProvider.notifier).clearPracticeRange();
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final notation = ref.watch(notationProvider);
    final data = ref.watch(playerProvider);
    final complete = _start != null && _end != null;

    final appBar = AppBar(
      backgroundColor: CymbraColors.surfaceContainerLow,
      foregroundColor: CymbraColors.onSurface,
      title: Text(_title(l10n), key: const Key('measure-select-title')),
      actions: [
        TextButton(
          key: const Key('measure-select-whole'),
          onPressed: _wholePiece,
          child: Text(l10n.measureSelectWholePiece),
        ),
        Padding(
          padding: const EdgeInsets.only(left: 4, right: 8),
          child: FilledButton(
            key: const Key('measure-select-confirm'),
            onPressed: complete ? _confirm : null,
            child: Text(l10n.measureSelectConfirm),
          ),
        ),
      ],
    );

    // Guarded at the entry point; degrade to an empty body if the score is
    // swapped out from under the open route.
    if (!notation.hasDocument || data.measureStartMs.isEmpty) {
      return Scaffold(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        appBar: appBar,
        body: const SizedBox.shrink(),
      );
    }

    final sizeFactor = resolveScoreSize(
      ref.watch(playerPreferencesProvider.select((p) => p.scoreSize)),
      isPhone: context.isPhoneLayout,
    ).factor;
    final palette = NotationPalette.of(
      ref.watch(playerPreferencesProvider.select((p) => p.notationTheme)),
    );
    return Scaffold(
      backgroundColor: palette.background,
      appBar: appBar,
      body: LayoutBuilder(
        builder: (context, constraints) {
          // Same engraving contract as the Partition view: systems are laid
          // out against the width divided by the score-size factor, painted at
          // the real width with a scaled staff space. With no keyboard below,
          // the engraving gets the full viewport height — what makes this mode
          // workable on landscape-locked phones.
          final engraveWidth = constraints.maxWidth;
          final staffSpace = 12.0 * sizeFactor;
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (!mounted) return;
            ref
                .read(notationProvider.notifier)
                .setAvailableWidth(engraveWidth / sizeFactor);
          });
          final measurer = PartitionPainter(
            document: notation.document!,
            systems: notation.systems,
            // Before the first measure → no playhead cursor, no dimming: this
            // is a picker, not a playback view.
            elapsedMs: -1,
            measureStartMs: data.measureStartMs,
            songEndMs: data.songEndMs,
            activeNotes: const {},
            selectedHands: data.selectedHands,
            staffSpace: staffSpace,
            palette: palette,
          );
          return SingleChildScrollView(
            controller: _scroll,
            // Rebuild as the view scrolls so the painter culls to the visible
            // systems (same large-score fix as the Partition view).
            child: ListenableBuilder(
              listenable: _scroll,
              builder: (context, _) {
                final pos = _scroll.hasClients ? _scroll.position : null;
                final viewTop = pos != null && pos.hasPixels ? pos.pixels : 0.0;
                final viewHeight = pos != null && pos.hasViewportDimension
                    ? pos.viewportDimension
                    : constraints.maxHeight;
                return GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTapUp: _onScoreTap,
                  child: CustomPaint(
                    key: const Key('measure-select-canvas'),
                    painter: PartitionPainter(
                      document: notation.document!,
                      systems: notation.systems,
                      elapsedMs: -1,
                      measureStartMs: data.measureStartMs,
                      songEndMs: data.songEndMs,
                      activeNotes: const {},
                      selectedHands: data.selectedHands,
                      viewTop: viewTop,
                      viewBottom: viewTop + viewHeight,
                      staffSpace: staffSpace,
                      palette: palette,
                      practiceRange: _highlight,
                      hitRects: _hits,
                    ),
                    size: Size(engraveWidth, measurer.heightFor(engraveWidth)),
                  ),
                );
              },
            ),
          );
        },
      ),
    );
  }
}
