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

import 'lesson_pitch.dart';
import 'lesson_rhythm.dart';

part 'course_manifest.freezed.dart';

/// The highest manifest `schemaVersion` this build understands. A manifest above
/// it is declined by [parseCourseManifest]. v2 added the interactive solfège
/// blocks (`staff`, `readPlay`, `nameNote`, `placeNote`, `rhythmTap`,
/// `earChoice`, `buildChord`).
const int kCourseSchemaVersion = 2;

/// Inline-localized text: language code → string. Kept as a plain map so a
/// third-party manifest is self-contained (no dependency on the app's ARBs).
typedef InlineText = Map<String, String>;

/// Resolves inline-localized [text] for [languageCode], falling back to English
/// then any available value, and finally the empty string.
String resolveInline(InlineText text, String languageCode) =>
    text[languageCode] ??
    text['en'] ??
    (text.isNotEmpty ? text.values.first : '');

/// Parses an inline-i18n JSON object (a `{en, fr, …}` map, as carried in the
/// gRPC summary's `title_json`) into an [InlineText]. Non-object or malformed
/// JSON yields an empty map.
InlineText parseInlineJson(String source) {
  try {
    return _inline(jsonDecode(source));
  } catch (_) {
    return const {};
  }
}

/// Lightweight listing metadata for a course — what the home screen needs to
/// draw a tile and group it, without fetching the (potentially large) manifest
/// body. Mirrors the gRPC `CourseSummary`.
@freezed
sealed class CourseListing with _$CourseListing {
  const factory CourseListing({
    required String id,
    required int schemaVersion,
    @Default('piano') String instrument,
    @Default('solfege') String track,
    @Default('beginner') String level,
    @Default('') String unit,
    @Default(<String, String>{}) InlineText unitTitle,
    @Default(0) int sortOrder,
    @Default(<String, String>{}) InlineText title,
  }) = _CourseListing;
}

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

/// One element drawn on a lesson staff: a pitched note or a rest, with its
/// rhythmic figure. Pitch is null exactly when [fig] is a rest.
class LessonStaffElement {
  final LessonPitch? pitch;
  final RhythmFigure fig;

  const LessonStaffElement({this.pitch, required this.fig});

  @override
  bool operator ==(Object other) =>
      other is LessonStaffElement && other.pitch == pitch && other.fig == fig;

  @override
  int get hashCode => Object.hash(pitch, fig);
}

/// A meter drawn on a lesson staff (e.g. 3/4).
class LessonTimeSig {
  final int beats;
  final int beatType;

  const LessonTimeSig(this.beats, this.beatType);

  @override
  bool operator ==(Object other) =>
      other is LessonTimeSig &&
      other.beats == beats &&
      other.beatType == beatType;

  @override
  int get hashCode => Object.hash(beats, beatType);
}

/// One answer of an `earChoice` block: a stable id + its inline-localized label.
class EarOption {
  final String id;
  final InlineText label;

  const EarOption(this.id, this.label);

  @override
  bool operator ==(Object other) =>
      other is EarOption &&
      other.id == id &&
      other.label.length == label.length &&
      label.entries.every((e) => other.label[e.key] == e.value);

  @override
  int get hashCode => Object.hash(id, label.length);
}

/// How a keyboard exercise reveals note names on the awaited keys.
enum LessonLabelMode { always, afterMiss, never }

/// How a `readPlay` block sequences its notes.
enum ReadPlayMode {
  /// One note shown at a time; each correct answer advances the queue.
  drill,

  /// The whole phrase is shown; play it left to right (wait-mode style).
  melody,

  /// All notes shown together (a chord); play them in any order.
  set,
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
  /// [options] entry, [feedback] is shown after answering. With [keyboard], a
  /// tappable (sounding) piano is shown above the choices — for questions that
  /// say "look at the keyboard", the keyboard must be there to look at.
  const factory CourseBlock.question({
    required InlineText prompt,
    required List<InlineText> options,
    required int answerIndex,
    @Default(<String, String>{}) InlineText feedback,
    @Default(false) bool keyboard,
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

  // --- v2 interactive solfège blocks --------------------------------------

  /// A static staff illustration built from typed [elements] — the flexible
  /// successor to the closed `diagram` set for anything note-bearing.
  const factory CourseBlock.staff({
    required LessonClef clef,
    @Default(0) int keyFifths,
    LessonTimeSig? time,
    @Default(<LessonStaffElement>[]) List<LessonStaffElement> elements,
    @Default(false) bool labels,
    @Default(<String, String>{}) InlineText caption,
  }) = StaffBlock;

  /// Read notes on the staff and play them on the keyboard (or MIDI) — the core
  /// staff→key drill, in three shapes ([ReadPlayMode]).
  const factory CourseBlock.readPlay({
    required List<LessonPitch> notes,
    @Default(ReadPlayMode.drill) ReadPlayMode mode,
    LessonClef? clef,
    @Default(0) int keyFifths,
    @Default(LessonLabelMode.afterMiss) LessonLabelMode labels,
    @Default(<String, String>{}) InlineText prompt,
  }) = ReadPlayBlock;

  /// Name the staff note among localized name chips (chips sound on tap).
  const factory CourseBlock.nameNote({
    required List<LessonPitch> notes,
    LessonClef? clef,
    @Default(0) int keyFifths,
    @Default(3) int choiceCount,
    @Default(<String, String>{}) InlineText prompt,
  }) = NameNoteBlock;

  /// Place a named note on a tappable staff (position-only — targets are
  /// naturals; the tapped step sounds so a wrong spot is *heard*).
  const factory CourseBlock.placeNote({
    required List<LessonPitch> targets,
    required LessonClef clef,
    @Default(<String, String>{}) InlineText prompt,
  }) = PlaceNoteBlock;

  /// Tap a written rhythm on a pad against the metronome.
  const factory CourseBlock.rhythmTap({
    required List<RhythmFigure> pattern,
    @Default(4) int beats,
    @Default(4) int beatType,
    @Default(80) int bpm,
    @Default(0.7) double passRatio,
    @Default(<String, String>{}) InlineText prompt,
  }) = RhythmTapBlock;

  /// Listen to a short sequence and answer a choice question about it.
  const factory CourseBlock.earChoice({
    required List<LessonPitch> notes,
    required List<EarOption> choices,
    required String answerId,
    @Default(700) int gapMs,
    @Default(false) bool harmonic,
    @Default(true) bool reveal,
    @Default(<String, String>{}) InlineText prompt,
  }) = EarChoiceBlock;

  /// Build a chord by toggling keys on the keyboard; the selection strums when
  /// complete so a wrong set is heard, not just marked.
  const factory CourseBlock.buildChord({
    required List<LessonPitch> notes,
    @Default(<String, String>{}) InlineText prompt,
  }) = BuildChordBlock;

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
        keyboard: raw['keyboard'] == true,
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
    case 'staff':
      final clef = _clef(raw['clef']);
      final elements = _staffElements(raw['elements']);
      if (clef == null || elements == null || elements.isEmpty) {
        return const CourseBlock.unsupported(type: 'staff');
      }
      return CourseBlock.staff(
        clef: clef,
        keyFifths: _fifths(raw['keyFifths']),
        time: _time(raw['time']),
        elements: elements,
        labels: raw['labels'] == true,
        caption: _inline(raw['caption']),
      );
    case 'readPlay':
      final notes = _pitches(raw['notes']);
      if (notes == null || notes.isEmpty) {
        return const CourseBlock.unsupported(type: 'readPlay');
      }
      return CourseBlock.readPlay(
        notes: notes,
        mode: switch (raw['mode']) {
          'melody' => ReadPlayMode.melody,
          'set' => ReadPlayMode.set,
          _ => ReadPlayMode.drill,
        },
        clef: _clef(raw['clef']),
        keyFifths: _fifths(raw['keyFifths']),
        labels: switch (raw['labels']) {
          'always' => LessonLabelMode.always,
          'never' => LessonLabelMode.never,
          _ => LessonLabelMode.afterMiss,
        },
        prompt: _inline(raw['prompt']),
      );
    case 'nameNote':
      final notes = _pitches(raw['notes']);
      if (notes == null || notes.isEmpty) {
        return const CourseBlock.unsupported(type: 'nameNote');
      }
      final count = raw['choiceCount'];
      return CourseBlock.nameNote(
        notes: notes,
        clef: _clef(raw['clef']),
        keyFifths: _fifths(raw['keyFifths']),
        choiceCount: count is int ? count.clamp(2, 4) : 3,
        prompt: _inline(raw['prompt']),
      );
    case 'placeNote':
      final targets = _pitches(raw['targets']);
      final clef = _clef(raw['clef']);
      // Placement is position-only, so altered targets make no sense here.
      if (targets == null ||
          targets.isEmpty ||
          clef == null ||
          targets.any((t) => t.alter != 0)) {
        return const CourseBlock.unsupported(type: 'placeNote');
      }
      return CourseBlock.placeNote(
        targets: targets,
        clef: clef,
        prompt: _inline(raw['prompt']),
      );
    case 'rhythmTap':
      final rawPattern = raw['pattern'];
      final pattern = <RhythmFigure>[];
      if (rawPattern is List) {
        for (final e in rawPattern) {
          final f = RhythmFigure.parse(e);
          if (f == null) {
            return const CourseBlock.unsupported(type: 'rhythmTap');
          }
          pattern.add(f);
        }
      }
      final beats = raw['beats'];
      final beatType = raw['beatType'];
      final bpm = raw['bpm'];
      final ratio = raw['passRatio'];
      if (pattern.isEmpty || pattern.every((f) => f.rest)) {
        return const CourseBlock.unsupported(type: 'rhythmTap');
      }
      return CourseBlock.rhythmTap(
        pattern: pattern,
        beats: beats is int ? beats.clamp(1, 12) : 4,
        beatType: beatType is int ? beatType.clamp(1, 16) : 4,
        bpm: bpm is int ? bpm.clamp(30, 240) : 80,
        passRatio: ratio is num ? ratio.toDouble().clamp(0.1, 1.0) : 0.7,
        prompt: _inline(raw['prompt']),
      );
    case 'earChoice':
      final notes = _pitches(raw['notes']);
      final rawChoices = raw['choices'];
      final answerId = raw['answerId'];
      final choices = <EarOption>[
        if (rawChoices is List)
          for (final c in rawChoices)
            if (c is Map && c['id'] is String && (c['id'] as String).isNotEmpty)
              EarOption(c['id'] as String, _inline(c['label'])),
      ];
      if (notes == null ||
          notes.isEmpty ||
          choices.length < 2 ||
          answerId is! String ||
          !choices.any((c) => c.id == answerId)) {
        return const CourseBlock.unsupported(type: 'earChoice');
      }
      final gap = raw['gapMs'];
      return CourseBlock.earChoice(
        notes: notes,
        choices: choices,
        answerId: answerId,
        gapMs: gap is int ? gap.clamp(200, 3000) : 700,
        harmonic: raw['harmonic'] == true,
        reveal: raw['reveal'] != false,
        prompt: _inline(raw['prompt']),
      );
    case 'buildChord':
      final notes = _pitches(raw['notes']);
      if (notes == null || notes.length < 2 || notes.length > 5) {
        return const CourseBlock.unsupported(type: 'buildChord');
      }
      return CourseBlock.buildChord(
        notes: notes,
        prompt: _inline(raw['prompt']),
      );
    default:
      return CourseBlock.unsupported(type: type is String ? type : 'unknown');
  }
}

/// Parses a manifest clef token; null for absent/unknown (the caller decides
/// whether that means "default" or "decline").
LessonClef? _clef(Object? raw) => switch (raw) {
  'treble' => LessonClef.treble,
  'bass' => LessonClef.bass,
  _ => null,
};

int _fifths(Object? raw) => raw is int ? raw.clamp(-7, 7) : 0;

LessonTimeSig? _time(Object? raw) {
  if (raw is! Map) return null;
  final beats = raw['beats'];
  final beatType = raw['beatType'];
  return beats is int && beatType is int && beats > 0 && beatType > 0
      ? LessonTimeSig(beats.clamp(1, 12), beatType.clamp(1, 16))
      : null;
}

/// Parses a list of pitch strings strictly: one invalid spelling declines the
/// whole list (a drill silently missing a note would change its meaning).
List<LessonPitch>? _pitches(Object? raw) {
  if (raw is! List || raw.isEmpty) return null;
  final out = <LessonPitch>[];
  for (final e in raw) {
    final p = e is String ? LessonPitch.parse(e) : null;
    if (p == null) return null;
    out.add(p);
  }
  return out;
}

/// Parses staff elements: notes `{"p": "G4", "fig": "quarter"}` and rests
/// `{"rest": true, "fig": "quarter"}`. One malformed element declines the list.
List<LessonStaffElement>? _staffElements(Object? raw) {
  if (raw is! List) return null;
  final out = <LessonStaffElement>[];
  for (final e in raw) {
    if (e is! Map) return null;
    final fig = RhythmFigure.parse(e);
    if (fig == null) return null;
    if (fig.rest) {
      out.add(LessonStaffElement(fig: fig));
      continue;
    }
    final p = e['p'] is String ? LessonPitch.parse(e['p'] as String) : null;
    if (p == null) return null;
    out.add(LessonStaffElement(pitch: p, fig: fig));
  }
  return out;
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
