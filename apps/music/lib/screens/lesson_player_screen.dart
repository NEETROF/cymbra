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

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../services/course_catalog_service.dart';
import '../state/course_completion_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/course_diagram.dart';
import '../widgets/play_key_view.dart';
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

  /// The blocks the player can show — dropping the ones it doesn't render
  /// (unsupported/unknown), so a forward-incompatible block never traps the user.
  List<CourseBlock> _steps(CourseManifest m) =>
      m.blocks.where((b) => b is! UnsupportedBlock).toList();

  void _finish() {
    ref.read(courseCompletionProvider.notifier).markCompleted(widget.courseId);
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final async = ref.watch(courseManifestProvider(widget.courseId));

    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        backgroundColor: CymbraColors.surfaceContainerLowest,
        title: Text(
          async.valueOrNull == null
              ? ''
              : resolveInline(async.value!.title, lang),
        ),
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
        if (mounted) _finish();
      });
      return const SizedBox.shrink();
    }
    final index = _index.clamp(0, steps.length - 1);
    final block = steps[index];
    final isLast = index == steps.length - 1;
    // Advancing — used by the Next control (a non-blocking skip) and by an
    // interactive block that has been satisfied (e.g. the right key was played).
    void advance() => isLast ? _finish() : setState(() => _index = index + 1);

    return Column(
      children: [
        _ProgressBar(current: index + 1, total: steps.length),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(20),
            child: _BlockView(
              block: block,
              lang: lang,
              l10n: l10n,
              onSatisfied: advance,
            ),
          ),
        ),
        _ControlBar(
          canGoBack: index > 0,
          isLast: isLast,
          l10n: l10n,
          onBack: () => setState(() => _index = index - 1),
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
    required this.l10n,
    required this.onBack,
    required this.onNext,
  });

  final bool canGoBack;
  final bool isLast;
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
        FilledButton(
          key: const Key('lesson-next'),
          onPressed: onNext,
          child: Text(isLast ? l10n.lessonFinish : l10n.lessonNext),
        ),
      ],
    ),
  );
}

/// Renders one block. `playKey` is interactive (validated input advances via
/// [onSatisfied]); `score` engraves the inline MusicXML excerpt; media
/// (`image`/`video`) render from their URL, degrading to a caption card.
class _BlockView extends StatelessWidget {
  const _BlockView({
    required this.block,
    required this.lang,
    required this.l10n,
    required this.onSatisfied,
  });

  final CourseBlock block;
  final String lang;
  final AppLocalizations l10n;

  /// Called by an interactive block once its gate is met (the lesson advances).
  final VoidCallback onSatisfied;

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
/// always continue (the control bar's Next is never disabled).
class _QuestionView extends StatefulWidget {
  const _QuestionView({
    required this.block,
    required this.lang,
    required this.l10n,
  });

  final QuestionBlock block;
  final String lang;
  final AppLocalizations l10n;

  @override
  State<_QuestionView> createState() => _QuestionViewState();
}

class _QuestionViewState extends State<_QuestionView> {
  int? _selected;

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
        for (var i = 0; i < b.options.length; i++)
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: OutlinedButton(
              key: Key('lesson-option-$i'),
              onPressed: () => setState(() => _selected = i),
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
