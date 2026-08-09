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

import '../courses/course_manifest.dart';
import '../courses/lesson_sounder.dart';
import '../l10n/gen/app_localizations.dart';
import '../services/audio_service.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/build_chord_view.dart';
import '../widgets/course_diagram.dart';
import '../widgets/ear_choice_view.dart';
import '../widgets/lesson_celebration.dart';
import '../widgets/lesson_keyboard.dart';
import '../widgets/lesson_midi_chip.dart';
import '../widgets/lesson_staff.dart';
import '../widgets/name_note_view.dart';
import '../widgets/place_note_view.dart';
import '../widgets/play_key_view.dart';
import '../widgets/read_play_view.dart';
import '../widgets/rhythm_tap_view.dart';
import '../widgets/score_block_view.dart';

/// Opens the lesson player on [courseId] (change: add-notation-courses).
void openLessonPlayer(BuildContext context, String courseId) =>
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => LessonPlayerScreen(courseId: courseId),
      ),
    );

/// Runs a course's blocks at the user's pace: read, watch, answer. Skippable and
/// leaveable at any time; reaching the end marks the course completed (via
/// [CourseCompletion]) and it stays replayable. All block types render here
/// (text, diagram, image/video, question, playKey, score).
class LessonPlayerScreen extends ConsumerStatefulWidget {
  const LessonPlayerScreen({super.key, required this.courseId});

  final String courseId;

  @override
  ConsumerState<LessonPlayerScreen> createState() => _LessonPlayerScreenState();
}

class _LessonPlayerScreenState extends ConsumerState<LessonPlayerScreen> {
  int _index = 0;

  /// Exercise results of this run, by step index: true = solved without a
  /// single wrong attempt. Skipped steps never enter the map.
  final Map<int, bool> _flawless = {};

  /// Steps whose gate has been satisfied (their Next is re-enabled).
  final Set<int> _satisfied = {};

  /// Whether the discreet "skip" escape hatch is visible for the current
  /// gated step — appears only after [_kSkipDelay], so the gate guides
  /// without ever trapping.
  bool _skipVisible = false;
  Timer? _skipTimer;

  static const Duration _kSkipDelay = Duration(seconds: 12);

  /// The blocks the player can show — dropping the ones it doesn't render
  /// (unsupported/unknown), so a forward-incompatible block never traps the user.
  List<CourseBlock> _steps(CourseManifest m) =>
      m.blocks.where((b) => b is! UnsupportedBlock).toList();

  /// Whether [block] gates the Next control until the learner has done it.
  /// Questions gate too — answering (right OR wrong) unlocks, so nobody is
  /// forced into the correct answer, but nobody skips past unanswered either.
  static bool _gates(CourseBlock block) => switch (block) {
    ReadPlayBlock() ||
    NameNoteBlock() ||
    PlaceNoteBlock() ||
    RhythmTapBlock() ||
    EarChoiceBlock() ||
    BuildChordBlock() ||
    PlayKeyBlock() ||
    QuestionBlock() => true,
    _ => false,
  };

  @override
  void dispose() {
    _skipTimer?.cancel();
    super.dispose();
  }

  /// Arms the skip escape hatch for a newly-entered gated step.
  void _armSkip(CourseBlock block) {
    _skipTimer?.cancel();
    _skipVisible = false;
    if (!_gates(block) || _satisfied.contains(_index)) return;
    _skipTimer = Timer(_kSkipDelay, () {
      if (mounted) setState(() => _skipVisible = true);
    });
  }

  /// The listing that follows this course in the catalogue order, if the
  /// catalogue is available and the app supports it — the "Continue" target.
  CourseListing? _nextListing() {
    final all = ref.read(coursesProvider).valueOrNull ?? const [];
    final supported = all
        .where((l) => l.schemaVersion <= kCourseSchemaVersion)
        .toList();
    final i = supported.indexWhere((l) => l.id == widget.courseId);
    if (i < 0 || i + 1 >= supported.length) return null;
    return supported[i + 1];
  }

  Future<void> _finish(CourseManifest m, List<CourseBlock> steps) async {
    ref.read(courseCompletionProvider.notifier).markCompleted(widget.courseId);
    if (steps.isEmpty) {
      // Nothing was shown — nothing to celebrate.
      Navigator.of(context).pop();
      return;
    }
    final lang = Localizations.localeOf(context).languageCode;
    final gated = steps.where(_gates).length;
    final next = _nextListing();
    final action = await showLessonCelebration(
      context,
      lessonTitle: resolveInline(m.title, lang),
      flawless: _flawless.values.where((f) => f).length,
      gated: gated,
      nextLessonTitle: next == null ? null : resolveInline(next.title, lang),
    );
    if (!mounted) return;
    if (action == LessonCelebrationAction.next && next != null) {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute<void>(
          builder: (_) => LessonPlayerScreen(courseId: next.id),
        ),
      );
    } else {
      Navigator.of(context).pop();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final async = ref.watch(courseManifestProvider(widget.courseId));
    // Keep the catalogue warm so the celebration knows the next lesson.
    ref.watch(coursesProvider);

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(
          async.valueOrNull == null
              ? ''
              : resolveInline(async.value!.title, lang),
        ),
        actions: const [LessonMidiChip(), SizedBox(width: 4)],
      ),
      body: SafeArea(
        child: switch (async) {
          AsyncData(:final value) when value != null => _run(value, l10n, lang),
          AsyncLoading() => const Center(child: CircularProgressIndicator()),
          // Unknown/unpublished course, or an unsupported schema version.
          _ => const Center(
            child: Icon(
              Icons.school_outlined,
              size: 40,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
        },
      ),
    );
  }

  Widget _run(CourseManifest m, AppLocalizations l10n, String lang) {
    final steps = _steps(m);
    if (steps.isEmpty) {
      // Nothing to show — treat as instantly complete.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _finish(m, steps);
      });
      return const SizedBox.shrink();
    }
    final index = _index.clamp(0, steps.length - 1);
    final block = steps[index];
    final isLast = index == steps.length - 1;
    void goTo(int next) => setState(() {
      _index = next;
      _armSkip(steps[next.clamp(0, steps.length - 1)]);
    });
    // Advancing — used by the Next control, by a satisfied gate, and by the
    // late skip escape hatch.
    void advance() => isLast ? _finish(m, steps) : goTo(index + 1);

    // A gated step holds Next until its exercise reports completion; the
    // exercise records this run's first-try stat as it does.
    final gate = _gates(block) && !_satisfied.contains(index);
    if (_skipTimer == null && gate) _armSkip(block);

    return Column(
      children: [
        _ProgressBar(current: index + 1, total: steps.length),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(20),
            child: _BlockView(
              // Rebuild the exercise from scratch when the step changes, so
              // state (queues, selections) never leaks across steps.
              key: ValueKey('lesson-block-$index'),
              block: block,
              lang: lang,
              l10n: l10n,
              // The legacy playKey gate has no wrong-answer concept (stray
              // keys just don't count), so satisfying it counts as first-try.
              onSatisfied: () {
                if (_gates(block)) {
                  _flawless[index] = _flawless[index] ?? true;
                  _satisfied.add(index);
                }
                advance();
              },
              onCompleted: ({required bool flawless}) {
                _flawless[index] = flawless;
                _satisfied.add(index);
                advance();
              },
              // A question satisfies its gate WITHOUT advancing: the learner
              // reads the feedback and moves on at their own pace.
              onAnswered: ({required bool flawless}) {
                setState(() {
                  _flawless[index] = _flawless[index] ?? flawless;
                  _satisfied.add(index);
                  _skipTimer?.cancel();
                  _skipVisible = false;
                });
              },
            ),
          ),
        ),
        _ControlBar(
          canGoBack: index > 0,
          isLast: isLast,
          gated: gate,
          skipVisible: _skipVisible,
          l10n: l10n,
          onBack: () => goTo(index - 1),
          onNext: advance,
        ),
      ],
    );
  }
}

class _ProgressBar extends StatelessWidget {
  const _ProgressBar({required this.current, required this.total});

  final int current;
  final int total;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
    child: LinearProgressIndicator(
      value: total == 0 ? 0 : current / total,
      minHeight: 4,
      backgroundColor: CymbraColors.surfaceContainerHigh,
      color: CymbraColors.secondary,
    ),
  );
}

class _ControlBar extends StatelessWidget {
  const _ControlBar({
    required this.canGoBack,
    required this.isLast,
    required this.gated,
    required this.skipVisible,
    required this.l10n,
    required this.onBack,
    required this.onNext,
  });

  final bool canGoBack;
  final bool isLast;

  /// The current step's exercise has not been done yet: Next yields to the
  /// exercise itself, and only the late [skipVisible] escape hatch advances.
  final bool gated;
  final bool skipVisible;
  final AppLocalizations l10n;
  final VoidCallback onBack;
  final VoidCallback onNext;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(16),
    child: Row(
      children: [
        if (canGoBack)
          IconButton(
            key: const Key('lesson-back'),
            onPressed: onBack,
            icon: const Icon(Icons.arrow_back),
            color: CymbraColors.onSurfaceVariant,
          ),
        const Spacer(),
        if (!gated)
          FilledButton(
            key: const Key('lesson-next'),
            onPressed: onNext,
            child: Text(isLast ? l10n.lessonFinish : l10n.lessonNext),
          )
        else if (skipVisible)
          TextButton(
            key: const Key('lesson-skip'),
            onPressed: onNext,
            child: Text(
              l10n.lessonSkip,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ),
      ],
    ),
  );
}

/// Renders one block. Interactive blocks report through [onCompleted] (with
/// this run's first-try flag) or the legacy [onSatisfied]; `score` engraves the
/// inline MusicXML excerpt; media (`image`/`video`) render from their URL,
/// degrading to a caption card.
class _BlockView extends StatelessWidget {
  const _BlockView({
    super.key,
    required this.block,
    required this.lang,
    required this.l10n,
    required this.onSatisfied,
    required this.onCompleted,
    required this.onAnswered,
  });

  final CourseBlock block;
  final String lang;
  final AppLocalizations l10n;

  /// Called by an interactive block once its gate is met (the lesson advances).
  final VoidCallback onSatisfied;

  /// Called by a v2 exercise with its first-try result (the lesson advances
  /// and the run's celebration stat grows).
  final void Function({required bool flawless}) onCompleted;

  /// Called by a question on its first answer: the gate opens (Next returns)
  /// but the lesson stays put so the feedback can be read.
  final void Function({required bool flawless}) onAnswered;

  @override
  Widget build(BuildContext context) {
    return switch (block) {
      TextBlock(:final text) => Text(
        resolveInline(text, lang),
        style: const TextStyle(
          color: CymbraColors.onSurface,
          fontSize: 16,
          height: 1.4,
        ),
      ),
      DiagramBlock(:final id) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: CourseDiagram(id: id),
      ),
      ImageBlock(:final url, :final caption) => _Media(
        url: resolveInline(url, lang),
        caption: resolveInline(caption, lang),
        isVideo: false,
      ),
      VideoBlock(:final caption) => _Media(
        url: '',
        caption: resolveInline(caption, lang),
        isVideo: true,
      ),
      QuestionBlock() => _QuestionView(
        block: block as QuestionBlock,
        lang: lang,
        l10n: l10n,
        onAnswered: onAnswered,
      ),
      PlayKeyBlock(:final notes, :final prompt) => PlayKeyView(
        notes: notes,
        prompt: resolveInline(prompt, lang),
        onSatisfied: onSatisfied,
      ),
      ScoreBlock(:final musicXml, :final playable, :final prompt) =>
        ScoreBlockView(
          musicXml: musicXml,
          playable: playable,
          prompt: resolveInline(prompt, lang),
        ),
      StaffBlock(
        :final clef,
        :final keyFifths,
        :final time,
        :final elements,
        :final labels,
        :final caption,
      ) =>
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            LessonStaff(
              clef: clef,
              keyFifths: keyFifths,
              time: time,
              elements: elements,
              labels: labels,
            ),
            if (resolveInline(caption, lang).isNotEmpty) ...[
              const SizedBox(height: 6),
              Text(
                resolveInline(caption, lang),
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 13,
                ),
              ),
            ],
          ],
        ),
      ReadPlayBlock() => ReadPlayView(
        block: block as ReadPlayBlock,
        onCompleted: onCompleted,
      ),
      NameNoteBlock() => NameNoteView(
        block: block as NameNoteBlock,
        onCompleted: onCompleted,
      ),
      PlaceNoteBlock() => PlaceNoteView(
        block: block as PlaceNoteBlock,
        onCompleted: onCompleted,
      ),
      RhythmTapBlock() => RhythmTapView(
        block: block as RhythmTapBlock,
        onCompleted: onCompleted,
      ),
      EarChoiceBlock() => EarChoiceView(
        block: block as EarChoiceBlock,
        onCompleted: onCompleted,
      ),
      BuildChordBlock() => BuildChordView(
        block: block as BuildChordBlock,
        onCompleted: onCompleted,
      ),
      UnsupportedBlock() => const SizedBox.shrink(),
    };
  }
}

class _Media extends StatelessWidget {
  const _Media({
    required this.url,
    required this.caption,
    required this.isVideo,
  });

  final String url;
  final String caption;
  final bool isVideo;

  @override
  Widget build(BuildContext context) {
    final placeholder = Container(
      height: 160,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Icon(
        isVideo ? Icons.play_circle_outline : Icons.image_outlined,
        size: 40,
        color: CymbraColors.onSurfaceVariant,
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        ClipRRect(
          borderRadius: BorderRadius.circular(12),
          child: (!isVideo && url.isNotEmpty)
              ? Image.network(
                  url,
                  height: 160,
                  fit: BoxFit.contain,
                  errorBuilder: (_, _, _) => placeholder,
                )
              : placeholder,
        ),
        if (caption.isNotEmpty) ...[
          const SizedBox(height: 6),
          Text(
            caption,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 13,
            ),
          ),
        ],
      ],
    );
  }
}

/// A multiple-choice question: immediate feedback on an answer, but the user can
/// always continue (the control bar's Next is never disabled). With
/// [QuestionBlock.keyboard], a piano sits above the choices — tappable and
/// audible, because a question that says "look at the keyboard" must show one.
class _QuestionView extends ConsumerStatefulWidget {
  const _QuestionView({
    required this.block,
    required this.lang,
    required this.l10n,
    required this.onAnswered,
  });

  final QuestionBlock block;
  final String lang;
  final AppLocalizations l10n;

  /// Reports the FIRST answer (its correctness = the first-try stat); later
  /// taps are free exploration and change nothing upstream.
  final void Function({required bool flawless}) onAnswered;

  @override
  ConsumerState<_QuestionView> createState() => _QuestionViewState();
}

class _QuestionViewState extends ConsumerState<_QuestionView> {
  int? _selected;
  LessonSounder? _sounder;

  @override
  void initState() {
    super.initState();
    if (widget.block.keyboard) {
      _sounder = LessonSounder(ref.read(audioServiceProvider));
    }
  }

  @override
  void dispose() {
    _sounder?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final b = widget.block;
    final answered = _selected != null;
    final correct = _selected == b.answerIndex;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          resolveInline(b.prompt, widget.lang),
          style: const TextStyle(
            color: CymbraColors.onSurface,
            fontSize: 17,
            fontWeight: FontWeight.w700,
          ),
        ),
        const SizedBox(height: 12),
        if (b.keyboard) ...[
          LessonKeyboard(
            paintKey: const Key('question-keyboard'),
            rangeTargets: const [60],
            onKeyDown: (pitch) => _sounder?.tap(pitch),
          ),
          const SizedBox(height: 12),
        ],
        for (var i = 0; i < b.options.length; i++)
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: OutlinedButton(
              key: Key('lesson-option-$i'),
              onPressed: () {
                final first = _selected == null;
                setState(() => _selected = i);
                if (first) {
                  widget.onAnswered(flawless: i == b.answerIndex);
                }
              },
              style: OutlinedButton.styleFrom(
                alignment: Alignment.centerLeft,
                backgroundColor: answered && i == b.answerIndex
                    ? CymbraColors.tertiary.withValues(alpha: 0.15)
                    : (answered && i == _selected
                          ? CymbraColors.error.withValues(alpha: 0.12)
                          : null),
              ),
              child: Text(
                resolveInline(b.options[i], widget.lang),
                style: const TextStyle(color: CymbraColors.onSurface),
              ),
            ),
          ),
        if (answered) ...[
          const SizedBox(height: 4),
          Text(
            correct
                ? widget.l10n.lessonQuizCorrect
                : widget.l10n.lessonQuizWrong,
            style: TextStyle(
              color: correct ? CymbraColors.tertiary : CymbraColors.error,
              fontWeight: FontWeight.w700,
            ),
          ),
          if (resolveInline(b.feedback, widget.lang).isNotEmpty) ...[
            const SizedBox(height: 4),
            Text(
              resolveInline(b.feedback, widget.lang),
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ],
        ],
      ],
    );
  }
}
