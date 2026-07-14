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

import 'package:flutter/scheduler.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../painters/staff_painter.dart';
import '../services/audio_service.dart';
import '../services/notation_engine.dart';
import '../services/score_upload_service.dart';
import '../src/rust/api/musicxml.dart' show ScoreDocument, ScoreSummary;
import '../state/notation_playback.dart';
import '../state/player_data.dart' show TimedNote;
import '../state/score_catalog.dart';
import '../state/score_upload_notifier.dart';

/// The three-step contribution wizard (design 7). Reached via `Navigator.push`
/// from the library, only when signed in. Step gating is enforced by
/// [ScoreUploadNotifier]; this screen just renders the current step.
class ScoreUploadScreen extends ConsumerWidget {
  const ScoreUploadScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
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
          tooltip: step == UploadStep.upload || state.isDone ? 'Fermer' : 'Étape précédente',
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
          'Contribuer une partition',
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
    final (String label, VoidCallback? onPressed) = switch (state.step) {
      UploadStep.upload => (
        'Vérifier',
        state.canLeaveUpload ? notifier.goToVerify : null,
      ),
      UploadStep.verify => ('Continuer', notifier.goToConfirm),
      UploadStep.confirm => (
        'Envoyer',
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

  static const _labels = ['Import', 'Vérification', 'Confirmation'];

  @override
  Widget build(BuildContext context) {
    // After success, all steps read as completed.
    final currentIndex = done ? _labels.length : current.index;
    final scheme = Theme.of(context).colorScheme;

    Color lineColor(bool active) =>
        active ? scheme.primary : scheme.outlineVariant;

    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 4, 8, 10),
      child: Row(
        children: [
          for (var i = 0; i < _labels.length; i++)
            Expanded(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: i == 0
                            ? const SizedBox()
                            : Container(height: 2, color: lineColor(i <= currentIndex)),
                      ),
                      _Dot(index: i, currentIndex: currentIndex),
                      Expanded(
                        child: i == _labels.length - 1
                            ? const SizedBox()
                            : Container(height: 2, color: lineColor(i < currentIndex)),
                      ),
                    ],
                  ),
                  const SizedBox(height: 3),
                  Text(
                    _labels[i],
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

String _rejectMessage(String code) => switch (code) {
  'too_large' => 'Le fichier est trop volumineux.',
  'undecodable' => 'Le conteneur .mxl n\'a pas pu être décodé.',
  'unparseable' => 'Ce n\'est pas un fichier MusicXML valide.',
  'no_notes' => 'La partition ne contient aucune note jouable.',
  _ => 'Fichier invalide.',
};

// --- Step 1: Upload ---------------------------------------------------------

class _UploadStepView extends ConsumerWidget {
  const _UploadStepView();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(scoreUploadNotifierProvider);
    final notifier = ref.read(scoreUploadNotifierProvider.notifier);

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        FilledButton.icon(
          onPressed: state.validating ? null : notifier.pickAndValidate,
          icon: const Icon(Icons.upload_file),
          label: Text(
            state.file == null ? 'Choisir un fichier' : 'Choisir un autre fichier',
          ),
        ),
        const SizedBox(height: 8),
        const Text(
          'Formats acceptés : .musicxml, .xml ou .mxl (MusicXML compressé).',
          style: TextStyle(fontSize: 12),
        ),
        const SizedBox(height: 16),
        if (state.validating) const Center(child: CircularProgressIndicator()),
        if (state.file != null && !state.validating) ...[
          if (state.rejectCode != null)
            _Banner(
              icon: Icons.error_outline,
              color: Theme.of(context).colorScheme.error,
              text: _rejectMessage(state.rejectCode!),
            )
          else if (state.isValidated) ...[
            _Banner(
              icon: Icons.check_circle_outline,
              color: Colors.green,
              text: '« ${state.file!.name} » est valide.',
            ),
            const SizedBox(height: 16),
            const Text('Sur quelle base contribuez-vous ce contenu ?'),
            RadioGroup<RightsBasis>(
              groupValue: state.rightsBasis,
              onChanged: (v) {
                if (v != null) notifier.setRightsBasis(v);
              },
              child: const Column(
                children: [
                  RadioListTile<RightsBasis>(
                    value: RightsBasis.author,
                    title: Text('J\'en suis l\'auteur'),
                  ),
                  RadioListTile<RightsBasis>(
                    value: RightsBasis.publicDomain,
                    title: Text('Domaine public / licence libre'),
                  ),
                ],
              ),
            ),
            CheckboxListTile(
              value: state.rightsAck,
              onChanged: (v) => notifier.setRightsAck(v ?? false),
              controlAffinity: ListTileControlAffinity.leading,
              title: const Text(
                'Je certifie que cette déclaration est exacte et que je dispose '
                'des droits nécessaires pour mettre cette partition à disposition.',
                style: TextStyle(fontSize: 13),
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
  void _fireAudio(double nowMs) {
    _sounding.removeWhere((s) {
      if (s.endMs <= nowMs) {
        _audio.noteOff(s.pitch);
        return true;
      }
      return false;
    });
    while (_nextNote < _sorted.length && _sorted[_nextNote].startMs <= nowMs) {
      final n = _sorted[_nextNote];
      _audio.noteOn(n.pitch);
      _sounding.add((pitch: n.pitch, endMs: (n.startMs + n.durationMs).toDouble()));
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
    final summary = ref.read(scoreUploadNotifierProvider).summary;
    final playback = _playback;

    return Column(
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
                      ? Center(child: Text('Aperçu indisponible : $_error'))
                      : playback == null
                          ? const Center(child: CircularProgressIndicator())
                          : CustomPaint(
                              size: Size.infinite,
                              painter: StaffPainter(
                                notes: playback.notes,
                                elapsedMs: _elapsedMs,
                                activeNotes: const <int>{},
                                bpm: playback.bpm,
                                songEndMs: playback.songEndMs,
                                keyFifths: _doc?.attributes.keyFifths ?? 0,
                                beats: _doc?.attributes.time.beats ?? 4,
                                beatType: _doc?.attributes.time.beatType ?? 4,
                                measureStartMs: playback.measureStartMs,
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
                tooltip: _playing ? 'Pause' : 'Lecture (au tempo de la partition)',
              ),
              const SizedBox(width: 8),
              const Expanded(
                child: Text(
                  'Lecture au tempo de la partition, sans réglages.',
                  style: TextStyle(fontSize: 12),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// --- Step 3: Confirm --------------------------------------------------------

class _ConfirmStepView extends ConsumerWidget {
  const _ConfirmStepView();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(scoreUploadNotifierProvider);
    final notifier = ref.read(scoreUploadNotifierProvider.notifier);

    if (state.isDone) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.check_circle, color: Colors.green, size: 64),
            const SizedBox(height: 12),
            const Text('Partition ajoutée à vos contributions.'),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: () {
                Navigator.of(context).pop();
                ref.read(scoreUploadNotifierProvider.notifier).reset();
              },
              child: const Text('Terminé'),
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
        const Text(
          'Niveau de difficulté',
          style: TextStyle(fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        SegmentedButton<PracticeLevel>(
          style: ButtonStyle(
            visualDensity: VisualDensity.compact,
            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            padding: const WidgetStatePropertyAll(
              EdgeInsets.symmetric(horizontal: 8, vertical: 6),
            ),
            textStyle: const WidgetStatePropertyAll(TextStyle(fontSize: 13)),
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
            label: 'Titre (ce fichier n\'en contient pas)',
            onChanged: notifier.setFallbackTitle,
          ),
        if (state.summary?.composer == null)
          _FallbackField(
            label: 'Compositeur (optionnel)',
            onChanged: notifier.setFallbackComposer,
          ),
        if (state.submitError != null) ...[
          const SizedBox(height: 12),
          _Banner(
            icon: Icons.error_outline,
            color: Theme.of(context).colorScheme.error,
            text: state.submitError!,
          ),
        ],
        const SizedBox(height: 20),
        if (state.summary != null) _MetadataCard(summary: state.summary!),
      ],
    );
  }
}

// --- Shared bits ------------------------------------------------------------

class _MetadataCard extends StatelessWidget {
  const _MetadataCard({required this.summary});
  final ScoreSummary summary;

  @override
  Widget build(BuildContext context) {
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
            const Text(
              'Informations détectées (lecture seule)',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            row('Titre', summary.title ?? '—'),
            row('Compositeur', summary.composer ?? '—'),
            row('Tonalité', '${summary.keyFifths} altération(s)'),
            row('Mesure', summary.timeSig),
            row('Nombre de mesures', '${summary.measureCount}'),
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
        contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
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

