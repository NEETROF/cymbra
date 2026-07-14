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
    final step = ref.watch(scoreUploadNotifierProvider.select((s) => s.step));
    return Scaffold(
      appBar: AppBar(
        title: const Text('Contribuer une partition'),
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(4),
          child: LinearProgressIndicator(
            value: switch (step) {
              UploadStep.upload => 1 / 3,
              UploadStep.verify => 2 / 3,
              UploadStep.confirm => 1.0,
            },
          ),
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
            const SizedBox(height: 16),
            FilledButton(
              onPressed: state.canLeaveUpload ? notifier.goToVerify : null,
              child: const Text('Vérifier'),
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

  AudioService get _audio => ref.read(audioServiceProvider);

  @override
  void initState() {
    super.initState();
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
    final summary = ref.watch(
      scoreUploadNotifierProvider.select((s) => s.summary),
    );
    final notifier = ref.read(scoreUploadNotifierProvider.notifier);
    final playback = _playback;

    return Column(
      children: [
        if (summary != null) _MetadataCard(summary: summary),
        // StaffPainter is a scrolling synchronized view: it draws a fixed
        // horizontal window (playhead at 25%) and the notes scroll past as
        // elapsedMs advances. So give it a normal viewport, not a giant canvas.
        Expanded(
          child: _error != null
              ? Center(child: Text('Aperçu indisponible : $_error'))
              : playback == null
                  ? const Center(child: CircularProgressIndicator())
                  : Container(
                      margin: const EdgeInsets.symmetric(horizontal: 8),
                      clipBehavior: Clip.hardEdge,
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(8),
                        color: Colors.black.withValues(alpha: 0.15),
                      ),
                      child: CustomPaint(
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
        ),
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
        _StepNav(
          onBack: () {
            _stop();
            notifier.backToUpload();
          },
          onNext: () {
            _stop();
            notifier.goToConfirm();
          },
          nextLabel: 'Continuer',
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
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Terminé'),
            ),
          ],
        ),
      );
    }

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (state.summary != null) _MetadataCard(summary: state.summary!),
        // Fallback inputs shown only when the file itself carries no title /
        // composer (a parsed value always wins server-side — design 2b).
        if (state.summary?.title == null)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: TextField(
              decoration: const InputDecoration(
                labelText: 'Titre (ce fichier n\'en contient pas)',
                border: OutlineInputBorder(),
              ),
              onChanged: notifier.setFallbackTitle,
            ),
          ),
        if (state.summary?.composer == null)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: TextField(
              decoration: const InputDecoration(
                labelText: 'Compositeur (optionnel)',
                border: OutlineInputBorder(),
              ),
              onChanged: notifier.setFallbackComposer,
            ),
          ),
        const SizedBox(height: 8),
        const Text('Niveau de difficulté'),
        const SizedBox(height: 8),
        SegmentedButton<PracticeLevel>(
          segments: [
            for (final l in PracticeLevel.values)
              ButtonSegment(value: l, label: Text(l.label)),
          ],
          selected: state.level == null ? const {} : {state.level!},
          emptySelectionAllowed: true,
          onSelectionChanged: (s) =>
              s.isEmpty ? null : notifier.setLevel(s.first),
        ),
        const SizedBox(height: 16),
        if (state.submitError != null)
          _Banner(
            icon: Icons.error_outline,
            color: Theme.of(context).colorScheme.error,
            text: 'Échec de l\'envoi : ${state.submitError}',
          ),
        const SizedBox(height: 8),
        _StepNav(
          onBack: notifier.backToVerify,
          onNext: state.canFinalize && !state.submitting
              ? notifier.submit
              : null,
          nextLabel: state.submitting ? 'Envoi…' : 'Envoyer',
          busy: state.submitting,
        ),
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

class _StepNav extends StatelessWidget {
  const _StepNav({
    required this.onBack,
    required this.onNext,
    required this.nextLabel,
    this.busy = false,
  });
  final VoidCallback? onBack;
  final VoidCallback? onNext;
  final String nextLabel;
  final bool busy;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(12),
    child: Row(
      children: [
        OutlinedButton(onPressed: onBack, child: const Text('Retour')),
        const Spacer(),
        FilledButton(
          onPressed: onNext,
          child: busy
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(nextLabel),
        ),
      ],
    ),
  );
}
