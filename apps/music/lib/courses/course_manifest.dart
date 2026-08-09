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

//! The client model + defensive parser for an interactive course manifest
//! (change: add-notation-courses). A course is a self-describing, versioned JSON
//! document of typed **blocks** stored server-side and delivered opaquely; the
//! client owns the format and its **forward-compatibility**: a block whose
//! `type` it does not understand becomes an [UnsupportedBlock] (skipped at play
//! time), never a parse failure, and a manifest whose `schemaVersion` it cannot
//! handle is declined (so a newer server course never crashes an older app).
//!
//! All human-readable copy is **localized inline** in the manifest (a
//! `{en, fr, es, it}` map) so a course is self-contained — see [resolveInline].

import 'dart:convert';

import 'package:freezed_annotation/freezed_annotation.dart';

part 'course_manifest.freezed.dart';

/// The highest manifest `schemaVersion` this build understands. A manifest above
/// it is declined by [parseCourseManifest].
const int kCourseSchemaVersion = 1;

/// Inline-localized text: language code → string. Kept as a plain map so a
/// third-party manifest is self-contained (no dependency on the app's ARBs).
typedef InlineText = Map<String, String>;

/// Resolves inline-localized [text] for [languageCode], falling back to English
/// then any available value, and finally the empty string.
String resolveInline(InlineText text, String languageCode) =>
    text[languageCode] ??
    text['en'] ??
    (text.isNotEmpty ? text.values.first : '');

/// A parsed course: its grouping metadata (instrument/track/level), inline
/// titles, and the ordered blocks the lesson player runs.
@freezed
sealed class CourseManifest with _$CourseManifest {
  const factory CourseManifest({
    required int schemaVersion,
    required String id,
    @Default('piano') String instrument,
    @Default('solfege') String track,
    @Default('beginner') String level,
    @Default(<String, String>{}) InlineText title,
    @Default(<String, String>{}) InlineText summary,
    @Default(<CourseBlock>[]) List<CourseBlock> blocks,
  }) = _CourseManifest;
}

/// One step of a course. Discriminated on the manifest's block `type`; an
/// unknown type is preserved as [UnsupportedBlock] so the player can skip it
/// while the rest of the course still runs (forward-compatibility).
@freezed
sealed class CourseBlock with _$CourseBlock {
  /// A localized explanation.
  const factory CourseBlock.text({required InlineText text}) = TextBlock;

  /// A built-in notation diagram, referenced by a closed-set [id] the app knows
  /// how to render (never an arbitrary asset).
  const factory CourseBlock.diagram({required String id}) = DiagramBlock;

  /// An image shown from [url] (media is referenced, never embedded).
  const factory CourseBlock.image({
    required InlineText url,
    @Default(<String, String>{}) InlineText caption,
  }) = ImageBlock;

  /// A video shown from [url].
  const factory CourseBlock.video({
    required InlineText url,
    @Default(<String, String>{}) InlineText caption,
  }) = VideoBlock;

  /// A multiple-choice / true-false question; [answerIndex] is the correct
  /// [options] entry, [feedback] is shown after answering.
  const factory CourseBlock.question({
    required InlineText prompt,
    required List<InlineText> options,
    required int answerIndex,
    @Default(<String, String>{}) InlineText feedback,
  }) = QuestionBlock;

  /// Asks the user to play [notes] (MIDI) on the keyboard or a MIDI instrument.
  const factory CourseBlock.playKey({
    required List<int> notes,
    @Default(<String, String>{}) InlineText prompt,
  }) = PlayKeyBlock;

  /// An embedded notation excerpt ([musicXml]); when [playable] the user can
  /// perform it.
  const factory CourseBlock.score({
    required String musicXml,
    @Default(false) bool playable,
    @Default(<String, String>{}) InlineText prompt,
  }) = ScoreBlock;

  /// A block whose `type` this build does not support — skipped at play time.
  const factory CourseBlock.unsupported({required String type}) =
      UnsupportedBlock;
}

/// Parses a course manifest [source] (JSON text) into a [CourseManifest], or
/// returns null when it must be **declined**: malformed JSON, a missing/invalid
/// `id`, or a `schemaVersion` this build cannot handle (absent, < 1, or greater
/// than [maxSchemaVersion]). It never throws, and never fails on an unknown
/// block type — that block degrades to [UnsupportedBlock].
CourseManifest? parseCourseManifest(
  String source, {
  int maxSchemaVersion = kCourseSchemaVersion,
}) {
  final Object? decoded;
  try {
    decoded = jsonDecode(source);
  } catch (_) {
    return null;
  }
  if (decoded is! Map) return null;

  final schemaVersion = decoded['schemaVersion'];
  if (schemaVersion is! int ||
      schemaVersion < 1 ||
      schemaVersion > maxSchemaVersion) {
    return null;
  }
  final id = decoded['id'];
  if (id is! String || id.isEmpty) return null;

  final rawBlocks = decoded['blocks'];
  final blocks = <CourseBlock>[
    if (rawBlocks is List)
      for (final b in rawBlocks) _parseBlock(b),
  ];

  return CourseManifest(
    schemaVersion: schemaVersion,
    id: id,
    instrument: _str(decoded['instrument'], 'piano'),
    track: _str(decoded['track'], 'solfege'),
    level: _str(decoded['level'], 'beginner'),
    title: _inline(decoded['title']),
    summary: _inline(decoded['summary']),
    blocks: blocks,
  );
}

/// Parses one block; any shape it can't make sense of degrades to an
/// [UnsupportedBlock] rather than throwing (forward-compatibility).
CourseBlock _parseBlock(Object? raw) {
  if (raw is! Map) return const CourseBlock.unsupported(type: 'invalid');
  final type = raw['type'];
  switch (type) {
    case 'text':
      return CourseBlock.text(text: _inline(raw['text']));
    case 'diagram':
      final id = raw['id'];
      return id is String && id.isNotEmpty
          ? CourseBlock.diagram(id: id)
          : const CourseBlock.unsupported(type: 'diagram');
    case 'image':
      return CourseBlock.image(
        url: _inline(raw['url']),
        caption: _inline(raw['caption']),
      );
    case 'video':
      return CourseBlock.video(
        url: _inline(raw['url']),
        caption: _inline(raw['caption']),
      );
    case 'question':
      final options = raw['options'];
      final answer = raw['answerIndex'];
      if (options is! List || options.isEmpty || answer is! int) {
        return const CourseBlock.unsupported(type: 'question');
      }
      final opts = [for (final o in options) _inline(o)];
      final idx = answer.clamp(0, opts.length - 1);
      return CourseBlock.question(
        prompt: _inline(raw['prompt']),
        options: opts,
        answerIndex: idx,
        feedback: _inline(raw['feedback']),
      );
    case 'playKey':
      final notes = raw['notes'];
      final parsed = <int>[
        if (notes is List)
          for (final n in notes)
            if (n is int) n,
      ];
      return parsed.isEmpty
          ? const CourseBlock.unsupported(type: 'playKey')
          : CourseBlock.playKey(notes: parsed, prompt: _inline(raw['prompt']));
    case 'score':
      final xml = raw['musicXml'];
      return xml is String && xml.isNotEmpty
          ? CourseBlock.score(
              musicXml: xml,
              playable: raw['playable'] == true,
              prompt: _inline(raw['prompt']),
            )
          : const CourseBlock.unsupported(type: 'score');
    default:
      return CourseBlock.unsupported(type: type is String ? type : 'unknown');
  }
}

/// Coerces [raw] to an [InlineText] map (string→string), dropping non-string
/// entries; anything else yields an empty map.
InlineText _inline(Object? raw) {
  if (raw is! Map) return const {};
  final out = <String, String>{};
  raw.forEach((k, v) {
    if (k is String && v is String) out[k] = v;
  });
  return out;
}

String _str(Object? raw, String fallback) =>
    raw is String && raw.isNotEmpty ? raw : fallback;
