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
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../painters/staff_painter.dart';
import '../services/audio_service.dart';
import '../services/notation_engine.dart';
import '../services/score_upload_service.dart';
import '../src/rust/api/musicxml.dart'
    show InstrumentKind, ScoreDocument, ScoreSummary;
import '../state/drums_access.dart';
import '../state/notation_playback.dart';
import '../state/score_font.dart';
import '../widgets/score_font_listener.dart';
import '../state/note_density_core.dart';
import '../state/contributed_scores.dart';
import '../state/player_data.dart' show TimedNote;
import '../state/score_catalog.dart';
import '../state/score_upload_notifier.dart';
import '../widgets/score_propose_sheet.dart';

/// The three-step contribution wizard (design 7). Reached via `Navigator.push`
/// from the library, only when signed in. Step gating is enforced by
/// [ScoreUploadNotifier]; this screen just renders the current step.
class ScoreUploadScreen extends ConsumerWidget {
  const ScoreUploadScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(scoreUploadNotifierProvider);
    final notifier = ref.read(scoreUploadNotifierProvider.notifier);
    final step = state.step;

    void quit() {
      Navigator.of(context).maybePop();
      notifier.reset();
    }

    // The wizard is a dense multi-step form. On a phone the Material base sizes
    // crowd the screen, so shrink text ~15% there; on a tablet/desktop the extra
    // room makes that feel tiny, so keep the platform base. The device Dynamic
    // Type factor is respected proportionally either way (accessibility scaling
    // keeps working).
    final isPhone = MediaQuery.of(context).size.shortestSide < 600;
    final titleSize = isPhone ? 18.0 : 22.0;
    return MediaQuery(
      data: MediaQuery.of(context).copyWith(
        textScaler: TextScaler.linear(
          MediaQuery.textScalerOf(context).scale(1) * (isPhone ? 0.85 : 1.0),
        ),
      ),
      child: Scaffold(
        appBar: AppBar(
          // Back = previous step (or quit at the first step / after success). Reset
          // on quit so the next visit starts clean (user action → mutation allowed).
          leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            tooltip: step == UploadStep.upload || state.isDone
                ? l10n.uploadCloseTooltip
                : l10n.uploadPreviousStepTooltip,
            onPressed: () {
              if (state.isDone) {
                quit();
              } else {
                switch (step) {
                  case UploadStep.upload:
                    quit();
                  case UploadStep.verify:
                    notifier.backToUpload();
                  case UploadStep.confirm:
                    notifier.backToVerify();
                }
              }
            },
          ),
          // Style on the Text (not AppBar.titleTextStyle) so it merges with — and
          // keeps — the theme's title colour, only overriding the size.
          title: Text(
            l10n.uploadTitle,
            style: TextStyle(fontSize: titleSize, fontWeight: FontWeight.w600),
          ),
          actions: [
            if (!state.isDone) _ForwardAction(state: state, notifier: notifier),
            const SizedBox(width: 8),
          ],
          bottom: PreferredSize(
            preferredSize: const Size.fromHeight(58),
            child: _WizardStepper(current: step, done: state.isDone),
          ),
        ),
        // Centre + plafonne la largeur : carte centrée sur desktop, plein écran
        // sur mobile (le contenu est plus lisible qu'étiré sur toute la largeur).
        body: SafeArea(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 746),
              child: switch (step) {
                UploadStep.upload => const _UploadStepView(),
                UploadStep.verify => const _VerifyStepView(),
                UploadStep.confirm => const _ConfirmStepView(),
              },
            ),
          ),
        ),
      ),
    );
  }
}

/// The step-appropriate forward action, shown in the AppBar: Vérifier → Continuer
/// → Envoyer, disabled until the current step's gate is met.
class _ForwardAction extends StatelessWidget {
  const _ForwardAction({required this.state, required this.notifier});
  final ScoreUploadState state;
  final ScoreUploadNotifier notifier;

  @override
  Widget build(BuildContext context) {
    if (state.submitting) {
      return const Padding(
        padding: EdgeInsets.symmetric(horizontal: 16),
        child: Center(
          child: SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        ),
      );
    }
    final l10n = AppLocalizations.of(context);
    final (String label, VoidCallback? onPressed) = switch (state.step) {
      UploadStep.upload => (
        l10n.uploadActionVerify,
        state.canLeaveUpload ? notifier.goToVerify : null,
      ),
      UploadStep.verify => (l10n.uploadActionContinue, notifier.goToConfirm),
      UploadStep.confirm => (
        l10n.uploadActionSubmit,
        state.canFinalize ? notifier.submit : null,
      ),
    };
    return TextButton(onPressed: onPressed, child: Text(label));
  }
}

/// A compact 3-step header (Import → Vérification → Confirmation) with a
/// completed / current / pending state per step.
class _WizardStepper extends StatelessWidget {
  const _WizardStepper({required this.current, required this.done});
  final UploadStep current;
  final bool done;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final labels = [
      l10n.uploadStepImport,
      l10n.uploadStepVerification,
      l10n.uploadStepConfirmation,
    ];
    // After success, all steps read as completed.
    final currentIndex = done ? labels.length : current.index;
    final scheme = Theme.of(context).colorScheme;

    Color lineColor(bool active) =>
        active ? scheme.primary : scheme.outlineVariant;

    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 4, 8, 10),
      child: Row(
        children: [
          for (var i = 0; i < labels.length; i++)
            Expanded(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: i == 0
                            ? const SizedBox()
                            : Container(
                                height: 2,
                                color: lineColor(i <= currentIndex),
                              ),
                      ),
                      _Dot(index: i, currentIndex: currentIndex),
                      Expanded(
                        child: i == labels.length - 1
                            ? const SizedBox()
                            : Container(
                                height: 2,
                                color: lineColor(i < currentIndex),
                              ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 3),
                  Text(
                    labels[i],
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: i == currentIndex
                          ? FontWeight.w600
                          : FontWeight.w400,
                      color: i <= currentIndex
                          ? scheme.onSurface
                          : scheme.onSurfaceVariant,
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

class _Dot extends StatelessWidget {
  const _Dot({required this.index, required this.currentIndex});
  final int index;
  final int currentIndex;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final done = index < currentIndex;
    final current = index == currentIndex;
    return Container(
      width: 20,
      height: 20,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: done || current ? scheme.primary : Colors.transparent,
        border: Border.all(
          color: done || current ? scheme.primary : scheme.outlineVariant,
          width: 2,
        ),
      ),
      alignment: Alignment.center,
      child: done
          ? Icon(Icons.check, size: 13, color: scheme.onPrimary)
          : Text(
              '${index + 1}',
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w700,
                color: current ? scheme.onPrimary : scheme.onSurfaceVariant,
              ),
            ),
    );
  }
}

String _rejectMessage(AppLocalizations l10n, String code) => switch (code) {
  'too_large' => l10n.uploadRejectTooLarge,
  'undecodable' => l10n.uploadRejectUndecodable,
  'unparseable' => l10n.uploadRejectUnparseable,
  'no_notes' => l10n.uploadRejectNoNotes,
  // The drum gate (change: add-drums-access) — the file is fine, the feature
  // is not available to this caller. Local and backend refusals share it.
  kDrumsNotAvailableCode => l10n.uploadRejectDrumsNotAvailable,
  _ => l10n.uploadRejectGeneric,
};

// --- Step 1: Upload ---------------------------------------------------------

class _UploadStepView extends ConsumerWidget {
  const _UploadStepView();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(scoreUploadNotifierProvider);
    final notifier = ref.read(scoreUploadNotifierProvider.notifier);

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        FilledButton.icon(
          onPressed: state.validating ? null : notifier.pickAndValidate,
          icon: const Icon(Icons.upload_file),
          label: Text(
            state.file == null
                ? l10n.uploadPickFile
                : l10n.uploadPickAnotherFile,
          ),
        ),
        const SizedBox(height: 8),
        Text(l10n.uploadAcceptedFormats, style: const TextStyle(fontSize: 12)),
        const SizedBox(height: 16),
        if (state.validating) const Center(child: CircularProgressIndicator()),
        if (state.file != null && !state.validating) ...[
          if (state.rejectCode != null)
            _Banner(
              icon: Icons.error_outline,
              color: Theme.of(context).colorScheme.error,
              text: _rejectMessage(l10n, state.rejectCode!),
            )
          else if (state.isValidated) ...[
            _Banner(
              icon: Icons.check_circle_outline,
              color: Colors.green,
              text: l10n.uploadFileValid(state.file!.name),
            ),
            const SizedBox(height: 16),
            Text(l10n.uploadRightsQuestion),
            RadioGroup<RightsBasis>(
              groupValue: state.rightsBasis,
              onChanged: (v) {
                if (v != null) notifier.setRightsBasis(v);
              },
              child: Column(
                children: [
                  RadioListTile<RightsBasis>(
                    value: RightsBasis.author,
                    title: Text(l10n.uploadRightsAuthor),
                  ),
                  RadioListTile<RightsBasis>(
                    value: RightsBasis.publicDomain,
                    title: Text(l10n.uploadRightsPublicDomain),
                  ),
                ],
              ),
            ),
            CheckboxListTile(
              value: state.rightsAck,
              onChanged: (v) => notifier.setRightsAck(v ?? false),
              controlAffinity: ListTileControlAffinity.leading,
              title: Text(
                l10n.uploadRightsAck,
                style: const TextStyle(fontSize: 13),
              ),
            ),
          ],
        ],
      ],
    );
  }
}

// --- Step 2: Verify (horizontal, tempo-locked preview) ----------------------

class _VerifyStepView extends ConsumerStatefulWidget {
  const _VerifyStepView();
  @override
  ConsumerState<_VerifyStepView> createState() => _VerifyStepViewState();
}

class _VerifyStepViewState extends ConsumerState<_VerifyStepView>
    with SingleTickerProviderStateMixin {
  late final Ticker _ticker;
  DerivedPlayback? _playback;
  ScoreDocument? _doc;
  double _elapsedMs = 0;
  Duration _lastTick = Duration.zero;
  bool _playing = false;
  String? _error;

  // Audio playback: notes sorted by onset + a cursor, and the currently-sounding
  // voices so we can note-off at each note's end (the piano synth).
  List<TimedNote> _sorted = const [];
  int _nextNote = 0;
  final List<({int pitch, double endMs})> _sounding = [];

  // Cached in initState: `dispose()` must not touch `ref` (it's already disposed).
  late final AudioService _audio;

  @override
  void initState() {
    super.initState();
    _audio = ref.read(audioServiceProvider);
    _ticker = createTicker(_onTick);
    // Fire-and-forget: load the SoundFont so playback has sound (guarded natively
    // until ready). Same synth the player uses.
    _audio.init();
    _load();
  }

  Future<void> _load() async {
    final bytes = ref.read(scoreUploadNotifierProvider).file?.bytes;
    if (bytes == null) return;
    try {
      final doc = await ref.read(notationEngineProvider).parse(bytes);
      if (!mounted) return;
      final playback = notationToTimedNotes(doc);
      setState(() {
        _doc = doc;
        _playback = playback;
        _sorted = [...playback.notes]
          ..sort((a, b) => a.startMs.compareTo(b.startMs));
      });
    } catch (e) {
      if (mounted) setState(() => _error = '$e');
    }
  }

  /// Trigger note-ons/offs for the window crossed up to [nowMs] (drives sound).
  ///
  /// A percussion score sounds through the **drum** verbs, on the drum channel
  /// where a kit font's bank-128 presets live (change: add-drum-audio-channel).
  /// Sounding them melodically is what made an uploaded groove audition as
  /// piano pitches. It waits for the kit to be installed, exactly as the
  /// player does — before that the preview is honestly visual-only.
  void _fireAudio(double nowMs) {
    final percussion = _playback?.isPercussion ?? false;
    // A kit that has not finished installing means SILENT, never melodic and
    // never permanently muted: the playhead still consumes onsets (so the
    // notes already gone by do not burst out when the kit lands), and every
    // onset after it sounds normally.
    final ready =
        !percussion || ref.read(scoreFontProvider) == KitFontStatus.ready;
    _sounding.removeWhere((s) {
      if (s.endMs <= nowMs) {
        if (percussion) {
          _audio.drumOff(s.pitch);
        } else {
          _audio.noteOff(s.pitch);
        }
        return true;
      }
      // (a sounding entry only exists when it was actually sounded)
      return false;
    });
    while (_nextNote < _sorted.length && _sorted[_nextNote].startMs <= nowMs) {
      final n = _sorted[_nextNote];
      if (ready) {
        if (percussion) {
          _audio.drumOn(n.pitch);
        } else {
          _audio.noteOn(n.pitch);
        }
        _sounding.add((
          pitch: n.pitch,
          endMs: (n.startMs + n.durationMs).toDouble(),
        ));
      }
      _nextNote++;
    }
  }

  void _onTick(Duration elapsed) {
    final playback = _playback;
    if (playback == null) return;
    final dt = (elapsed - _lastTick).inMicroseconds / 1000.0;
    _lastTick = elapsed;
    final next = _elapsedMs + dt; // real time == score tempo (notes are in ms)
    _fireAudio(next);
    setState(() {
      _elapsedMs = next;
      if (_elapsedMs >= playback.songEndMs) {
        _elapsedMs = playback.songEndMs;
        _stop();
      }
    });
  }

  void _togglePlay() {
    setState(() {
      if (_playing) {
        _stop();
      } else {
        if (_elapsedMs >= (_playback?.songEndMs ?? 0)) {
          _elapsedMs = 0;
          _nextNote = 0;
          _sounding.clear();
        }
        _lastTick = Duration.zero;
        _ticker.start();
        _playing = true;
      }
    });
  }

  void _stop() {
    if (_ticker.isActive) _ticker.stop();
    _audio.allNotesOff();
    _sounding.clear();
    _playing = false;
  }

  @override
  void dispose() {
    _audio.allNotesOff();
    _ticker.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Read (not watch): the summary is fixed once we reach the Verify step, and a
    // lingering select-listener on a disposed instance can fire markNeedsBuild on
    // a defunct element when the notifier mutates elsewhere.
    final l10n = AppLocalizations.of(context);
    final summary = ref.read(scoreUploadNotifierProvider).summary;
    final playback = _playback;

    // The preview sounds the score it just parsed, so it owns the font swap
    // for its own family — the player is not mounted here (change:
    // add-drum-audio-channel): an uploaded drum groove installs the kit and is
    // restored to the piano when this step is left.
    return ScoreFontListener(
      percussion: playback?.isPercussion ?? false,
      child: Column(
        children: [
          // Scrollable content: on a short screen the metadata + preview scroll
          // instead of squeezing the preview to zero height (bottom overflow).
          Expanded(
            child: SingleChildScrollView(
              child: Column(
                children: [
                  if (summary != null) _MetadataCard(summary: summary),
                  // Fixed-height preview so it is always visible. StaffPainter is a
                  // scrolling synchronized view (playhead at 25%; notes scroll past
                  // as elapsedMs advances) — it just needs a normal viewport.
                  Container(
                    height: 220,
                    margin: const EdgeInsets.fromLTRB(8, 0, 8, 8),
                    clipBehavior: Clip.hardEdge,
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(8),
                      color: Colors.black.withValues(alpha: 0.15),
                    ),
                    child: _error != null
                        ? Center(
                            child: Text(l10n.uploadPreviewUnavailable(_error!)),
                          )
                        : playback == null
                        ? const Center(child: CircularProgressIndicator())
                        : CustomPaint(
                            size: Size.infinite,
                            painter: StaffPainter(
                              notes: playback.notes,
                              rests: playback.rests,
                              tieContinuations: playback.tieContinuations,
                              elapsedMs: _elapsedMs,
                              activeNotes: const <int>{},
                              bpm: playback.bpm,
                              songEndMs: playback.songEndMs,
                              keyFifths: _doc?.attributes.keyFifths ?? 0,
                              measureKeyFifths: playback.measureKeyFifths,
                              beats: _doc?.attributes.time.beats ?? 4,
                              beatType: _doc?.attributes.time.beatType ?? 4,
                              measureStartMs: playback.measureStartMs,
                              // Same readability cap as the player: this preview
                              // is how the uploader checks their score parsed
                              // right, so a dense one must not arrive cramped.
                              onsetGapMs: cachedOnsetGapMs(playback.notes),
                              measureMs: medianMeasureMs(
                                playback.measureStartMs,
                                songEndMs: playback.songEndMs,
                              ),
                            ),
                          ),
                  ),
                ],
              ),
            ),
          ),
          // Play controls pinned below the scroll area.
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                IconButton.filled(
                  onPressed: playback == null ? null : _togglePlay,
                  icon: Icon(_playing ? Icons.pause : Icons.play_arrow),
                  tooltip: _playing
                      ? l10n.uploadPauseTooltip
                      : l10n.uploadPlayTooltip,
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    l10n.uploadPlaybackHint,
                    style: const TextStyle(fontSize: 12),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// --- Step 3: Confirm --------------------------------------------------------

class _ConfirmStepView extends ConsumerWidget {
  const _ConfirmStepView();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final state = ref.watch(scoreUploadNotifierProvider);
    final notifier = ref.read(scoreUploadNotifierProvider.notifier);

    if (state.isDone) {
      final uploaded = state.result!;
      void closeWizard() {
        Navigator.of(context).pop();
        ref.read(scoreUploadNotifierProvider.notifier).reset();
      }

      // Opt-in proposal step (change: add-score-catalog-proposal): after a successful
      // upload the score is PRIVATE; the user may explicitly propose it to the public
      // catalog. Declining (Not now) leaves it private — never a pre-ticked default.
      Future<void> propose() async {
        final r = await showScoreProposeDialog(context);
        if (r == null) return;
        // Fire the action on the contributions notifier; the list reacts to state and
        // the library listener announces the outcome (the hub below stays mounted),
        // so nothing is claimed here before the server has answered.
        ref
            .read(myUploadsProvider.notifier)
            .proposeToPublicCatalog(
              uploaded.id,
              license: r.license,
              attestation: true,
              attribution: r.attribution,
            );
        closeWizard();
      }

      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_circle, color: Colors.green, size: 64),
            const SizedBox(height: 12),
            Text(l10n.uploadDoneMessage),
            const SizedBox(height: 24),
            Text(
              l10n.scoreProposeWizardPrompt,
              textAlign: TextAlign.center,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 12),
            FilledButton.icon(
              icon: const Icon(Icons.public),
              label: Text(l10n.scoreProposeAction),
              onPressed: propose,
            ),
            const SizedBox(height: 4),
            TextButton(
              onPressed: closeWizard,
              child: Text(l10n.scoreProposeSkip),
            ),
          ],
        ),
      );
    }

    // Actionable inputs FIRST (visible without scrolling on a phone); the
    // read-only recap goes at the bottom.
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          l10n.uploadDifficultyLabel,
          style: const TextStyle(fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        SegmentedButton<PracticeLevel>(
          style: const ButtonStyle(
            visualDensity: VisualDensity.compact,
            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            padding: WidgetStatePropertyAll(
              EdgeInsets.symmetric(horizontal: 8, vertical: 6),
            ),
            textStyle: WidgetStatePropertyAll(TextStyle(fontSize: 13)),
          ),
          segments: [
            for (final l in PracticeLevel.values)
              ButtonSegment(
                value: l,
                label: Text(l.localizedLabel(AppLocalizations.of(context))),
              ),
          ],
          selected: state.level == null ? const {} : {state.level!},
          emptySelectionAllowed: true,
          onSelectionChanged: (s) =>
              s.isEmpty ? null : notifier.setLevel(s.first),
        ),
        // Fallback inputs shown only when the file itself carries no title /
        // composer (a parsed value always wins server-side — design 2b).
        if (state.summary?.title == null)
          _FallbackField(
            label: l10n.uploadFallbackTitleLabel,
            onChanged: notifier.setFallbackTitle,
          ),
        if (state.summary?.composer == null)
          _FallbackField(
            label: l10n.uploadFallbackComposerLabel,
            onChanged: notifier.setFallbackComposer,
          ),
        if (state.submitErrorCode != null || state.submitError != null) ...[
          const SizedBox(height: 12),
          _Banner(
            icon: Icons.error_outline,
            color: Theme.of(context).colorScheme.error,
            // A typed refusal code (e.g. the drum gate) is localized here; it
            // wins over the pre-baked message so no raw string ever shows.
            text: state.submitErrorCode != null
                ? _rejectMessage(l10n, state.submitErrorCode!)
                : state.submitError!,
          ),
        ],
        const SizedBox(height: 20),
        if (state.summary != null) _MetadataCard(summary: state.summary!),
      ],
    );
  }
}

// --- Shared bits ------------------------------------------------------------

/// Localized label for a detected [InstrumentKind]; `null` (no row) when the
/// parse could not determine one — never an "unknown" placeholder.
String? _instrumentLabel(AppLocalizations l10n, InstrumentKind kind) =>
    switch (kind) {
      InstrumentKind.keyboard => l10n.instrumentKeyboard,
      InstrumentKind.percussion => l10n.instrumentDrums,
      InstrumentKind.unknown => null,
    };

class _MetadataCard extends StatelessWidget {
  const _MetadataCard({required this.summary});
  final ScoreSummary summary;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    Widget row(String k, String v) => Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 110,
            child: Text(k, style: const TextStyle(color: Colors.grey)),
          ),
          Expanded(child: Text(v)),
        ],
      ),
    );
    return Card(
      margin: const EdgeInsets.all(12),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.uploadDetectedInfo,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            row(l10n.uploadFieldTitle, summary.title ?? '—'),
            row(l10n.uploadFieldComposer, summary.composer ?? '—'),
            // The DETECTED instrument (change: add-drums-access): display only,
            // never a control — the facet is derived, like the key or the time
            // signature. An unknown instrument shows nothing at all.
            if (_instrumentLabel(l10n, summary.instrument) case final label?)
              row(l10n.uploadFieldInstrument, label),
            row(l10n.uploadFieldKey, l10n.uploadKeyValue(summary.keyFifths)),
            row(l10n.uploadFieldTimeSig, summary.timeSig),
            row(l10n.uploadFieldMeasureCount, '${summary.measureCount}'),
          ],
        ),
      ),
    );
  }
}

/// A fallback title/composer input. A `StatefulWidget` with its own controller so
/// it keeps focus and text even though the confirm view rebuilds on each change.
class _FallbackField extends StatefulWidget {
  const _FallbackField({required this.label, required this.onChanged});
  final String label;
  final ValueChanged<String> onChanged;

  @override
  State<_FallbackField> createState() => _FallbackFieldState();
}

class _FallbackFieldState extends State<_FallbackField> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 8),
    child: TextField(
      controller: _controller,
      onChanged: widget.onChanged,
      textInputAction: TextInputAction.done,
      style: const TextStyle(fontSize: 14),
      decoration: InputDecoration(
        labelText: widget.label,
        isDense: true,
        contentPadding: const EdgeInsets.symmetric(
          horizontal: 12,
          vertical: 12,
        ),
        border: const OutlineInputBorder(),
      ),
    ),
  );
}

class _Banner extends StatelessWidget {
  const _Banner({required this.icon, required this.color, required this.text});
  final IconData icon;
  final Color color;
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.all(12),
    decoration: BoxDecoration(
      color: color.withValues(alpha: 0.1),
      borderRadius: BorderRadius.circular(8),
    ),
    child: Row(
      children: [
        Icon(icon, color: color),
        const SizedBox(width: 8),
        Expanded(child: Text(text)),
      ],
    ),
  );
}
