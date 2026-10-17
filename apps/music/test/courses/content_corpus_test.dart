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

/// The contract of the first-party course corpus (change: add-notation-courses).
///
/// Every JSON file in `backend/content/courses/` ships as a DB row and is
/// parsed **by this app's real parser** — a file that fails here would silently
/// lose blocks (or a whole course) in production. So the corpus is validated
/// with `parseCourseManifest` itself, plus the authoring rules of the
/// curriculum (see the corpus README): all four locales everywhere, every
/// block understood, real interactivity in every lesson, rhythms that fill
/// their bars.
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/widgets/course_diagram.dart' show kCourseDiagramIds;

const Set<String> _locales = {'en', 'fr', 'es', 'it'};
const Map<String, String> _unitLevels = {
  'u1': 'beginner',
  'u2': 'beginner',
  'u3': 'beginner',
  'u4': 'intermediate',
  'u5': 'intermediate',
  'u6': 'intermediate',
  'u7': 'advanced',
};

/// Block types that gate the lesson (the learner *does* something).
const Set<String> _interactive = {
  'readPlay',
  'nameNote',
  'placeNote',
  'rhythmTap',
  'earChoice',
  'buildChord',
  'playKey',
  'question',
};
const Set<String> _v2Interactive = {
  'readPlay',
  'nameNote',
  'placeNote',
  'rhythmTap',
  'earChoice',
  'buildChord',
};

/// Every inline-i18n map in the tree must carry all four locales, non-empty.
void _checkI18n(Object? node, String path, List<String> errors) {
  if (node is Map) {
    if (node.containsKey('en') || node.containsKey('fr')) {
      for (final l in _locales) {
        final v = node[l];
        if (v is! String || v.trim().isEmpty) {
          errors.add('$path: missing/empty "$l" translation');
        }
      }
      return; // an i18n leaf — do not recurse into it
    }
    node.forEach((k, v) => _checkI18n(v, '$path.$k', errors));
  } else if (node is List) {
    for (var i = 0; i < node.length; i++) {
      _checkI18n(node[i], '$path[$i]', errors);
    }
  }
}

double _figBeats(Map f, int beatType) {
  const fractions = {
    'whole': 1.0,
    'half': 0.5,
    'quarter': 0.25,
    'eighth': 0.125,
    '16th': 0.0625,
  };
  final base = fractions[f['fig']] ?? 0;
  final dots = f['dots'] is int ? f['dots'] as int : 0;
  return base * (2 - (1 / (1 << dots))) * beatType;
}

void main() {
  final dir = Directory('../../backend/content/courses');
  final files =
      dir
          .listSync()
          .whereType<File>()
          .where((f) => f.path.endsWith('.json'))
          // The solfège path; the placement course has its own group below.
          .where((f) => f.uri.pathSegments.last.startsWith('sol-'))
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));
  final placementFile = File('${dir.path}/placement-piano.json');

  test('the corpus exists and is the full 49-course curriculum', () {
    expect(dir.existsSync(), isTrue);
    expect(
      files.length,
      greaterThanOrEqualTo(49),
      reason:
          'the solfège curriculum is 7 units × 6 lessons, each unit closed by '
          'a piece lesson (change: add-music-piece-lessons)',
    );
  });

  test('ids, units and sort orders are coherent and unique', () {
    final ids = <String>{};
    final orders = <int>{};
    for (final f in files) {
      final doc = jsonDecode(f.readAsStringSync()) as Map;
      final id = doc['id'] as String;
      final name = f.uri.pathSegments.last;
      expect(name, '$id.json', reason: 'filename must match id');
      expect(ids.add(id), isTrue, reason: 'duplicate id $id');

      final m = RegExp(r'^sol-(u[1-7])-(\d\d)-[a-z0-9-]+$').firstMatch(id);
      expect(m, isNotNull, reason: '$id must be sol-u<unit>-<nn>-<slug>');
      final unit = m!.group(1)!;
      final lesson = int.parse(m.group(2)!);
      expect(doc['unit'], unit, reason: '$id: unit field mismatch');
      expect(
        doc['sortOrder'],
        int.parse(unit.substring(1)) * 100 + lesson,
        reason: '$id: sortOrder must be unit*100 + lesson',
      );
      expect(orders.add(doc['sortOrder'] as int), isTrue);
      expect(
        doc['level'],
        _unitLevels[unit],
        reason: '$id: level must follow its unit',
      );
    }
  });

  test('every course parses cleanly with the real parser', () {
    for (final f in files) {
      final doc = jsonDecode(f.readAsStringSync()) as Map;
      final id = doc['id'] as String;

      expect(doc['status'], 'published', reason: id);
      expect(doc['instrument'], 'piano', reason: id);
      expect(doc['track'], 'solfege', reason: id);
      expect(doc['schemaVersion'], kCourseSchemaVersion, reason: id);

      final content = doc['content'] as Map;
      expect(content['id'], id, reason: '$id: content.id mismatch');
      expect(content['schemaVersion'], kCourseSchemaVersion, reason: id);

      final manifest = parseCourseManifest(jsonEncode(content));
      expect(manifest, isNotNull, reason: '$id: parser declined the manifest');
      // Mechanical musical truth: a higher/lower ear question must actually
      // sound the way its answer claims.
      for (final b in manifest!.blocks.whereType<EarChoiceBlock>()) {
        if (b.notes.length == 2 && const {'up', 'down'}.contains(b.answerId)) {
          final rises = b.notes[1].midi > b.notes[0].midi;
          expect(
            b.answerId == 'up',
            rises,
            reason: '$id: earChoice answer contradicts the pitches',
          );
        }
      }
      final unsupported = manifest.blocks.whereType<UnsupportedBlock>();
      expect(
        unsupported,
        isEmpty,
        reason:
            '$id: blocks degraded to unsupported '
            '(${unsupported.map((b) => b.type).join(', ')}) — '
            'they would be silently skipped in production',
      );
      expect(manifest.blocks.length, inInclusiveRange(5, 14), reason: id);
    }
  });

  test('every inline text carries all four locales', () {
    for (final f in files) {
      final doc = jsonDecode(f.readAsStringSync()) as Map;
      final errors = <String>[];
      _checkI18n(doc['title'], '${doc['id']}.title', errors);
      _checkI18n(doc['unitTitle'], '${doc['id']}.unitTitle', errors);
      _checkI18n(doc['content'], '${doc['id']}.content', errors);
      expect(errors, isEmpty, reason: errors.join('\n'));
    }
  });

  test('lessons are interactive, well-shaped and kind', () {
    for (final f in files) {
      final doc = jsonDecode(f.readAsStringSync()) as Map;
      final id = doc['id'] as String;
      final blocks = ((doc['content'] as Map)['blocks'] as List).cast<Map>();
      final types = blocks.map((b) => b['type'] as String).toList();

      expect(types.first, 'text', reason: '$id: first block must be the hook');
      expect(types.last, 'text', reason: '$id: last block must be the recap');
      for (var i = 1; i < types.length; i++) {
        expect(
          types[i] == 'text' && types[i - 1] == 'text',
          isFalse,
          reason: '$id: two consecutive text blocks at $i',
        );
      }
      // A piece lesson's exercise IS the piece (README rule 7): it carries at
      // least one piece step and is exempt from the v2 minimum.
      final pieceLesson = types.contains('piece');
      if (pieceLesson) {
        expect(
          types.where((t) => t == 'piece').length,
          greaterThanOrEqualTo(1),
          reason: '$id: a piece lesson needs a piece step',
        );
      } else {
        expect(
          types.where(_interactive.contains).length,
          greaterThanOrEqualTo(4),
          reason: '$id: a lesson is something you DO — ≥4 interactive blocks',
        );
        expect(
          types.where(_v2Interactive.contains).length,
          greaterThanOrEqualTo(2),
          reason: '$id: ≥2 v2 interactive blocks (staff/keyboard/ear/rhythm)',
        );
      }

      for (final b in blocks.where((b) => b['type'] == 'diagram')) {
        expect(
          kCourseDiagramIds,
          contains(b['id']),
          reason: '$id: unknown diagram id ${b['id']}',
        );
      }
      for (final b in blocks.where((b) => b['type'] == 'rhythmTap')) {
        final beats = b['beats'] is int ? b['beats'] as int : 4;
        final beatType = b['beatType'] is int ? b['beatType'] as int : 4;
        final total = (b['pattern'] as List).cast<Map>().fold<double>(
          0,
          (sum, fig) => sum + _figBeats(fig, beatType),
        );
        final bars = total / beats;
        expect(
          bars == 1.0 || bars == 2.0,
          isTrue,
          reason:
              '$id: rhythmTap pattern fills $bars bars '
              '(must be exactly 1 or 2)',
        );
      }
      for (final b in blocks.where((b) => b['type'] == 'readPlay')) {
        final len = (b['notes'] as List).length;
        final mode = b['mode'] ?? 'drill';
        expect(
          len,
          lessThanOrEqualTo(mode == 'melody' ? 16 : 8),
          reason: '$id: readPlay too long for a 3-minute lesson',
        );
      }
      for (final b in blocks.where((b) => b['type'] == 'score')) {
        final xml = b['musicXml'] as String;
        expect(
          xml.contains('<score-partwise') && xml.contains('<pitch>'),
          isTrue,
          reason: '$id: score block must be MINIMAL-fixture-shaped MusicXML',
        );
      }
    }
  });

  // --- Piece lessons (change: add-music-piece-lessons) -----------------------

  test(
    'every unit closes with a piece lesson whose chain ends free at tempo',
    () {
      for (final unit in _unitLevels.keys) {
        final closing = files
            .map((f) => jsonDecode(f.readAsStringSync()) as Map)
            .where((d) => d['unit'] == unit)
            .reduce(
              (a, b) =>
                  (a['sortOrder'] as int) > (b['sortOrder'] as int) ? a : b,
            );
        final id = closing['id'] as String;
        final blocks = ((closing['content'] as Map)['blocks'] as List)
            .cast<Map>();
        final pieces = blocks.where((b) => b['type'] == 'piece').toList();
        expect(
          pieces,
          isNotEmpty,
          reason: '$unit must close on a piece lesson',
        );
        final refs = pieces.map((b) => (b['score'] as Map)['bundled']).toSet();
        expect(refs.length, 1, reason: '$id: one piece per closing chain');
        expect(
          refs.single,
          isA<String>(),
          reason:
              '$id: first-party content '
              'references a bundled score (catalogue refs cannot be scanned)',
        );
        final last = pieces.last;
        expect(last['waitMode'], isFalse, reason: '$id: last step is free-run');
        expect(
          (last['speed'] as num).toDouble(),
          1.0,
          reason: '$id: last step is at full tempo',
        );
        for (final b in pieces) {
          final pass = b['passPct'];
          expect(
            pass is int && pass >= 1 && pass <= 100,
            isTrue,
            reason: '$id: passPct in 1..100',
          );
        }
      }
    },
  );

  group('the placement course (change: add-music-placement-test)', () {
    Map doc() => jsonDecode(placementFile.readAsStringSync()) as Map;
    List<Map> blocks() =>
        ((doc()['content'] as Map)['blocks'] as List).cast<Map>();

    test('exists, is published on the placement track, and parses', () {
      expect(placementFile.existsSync(), isTrue);
      final d = doc();
      expect(d['id'], 'placement-piano');
      expect(d['status'], 'published');
      expect(d['track'], kPlacementTrack);
      expect((d['content'] as Map)['track'], kPlacementTrack);
      expect(d['sortOrder'], 1);
      final m = parseCourseManifest(jsonEncode(d['content']));
      expect(m, isNotNull);
      expect(m!.track, kPlacementTrack);
      expect(m.blocks.whereType<UnsupportedBlock>(), isEmpty);
      final errors = <String>[];
      _checkI18n(d['title'], 'placement.title', errors);
      _checkI18n(d['unitTitle'], 'placement.unitTitle', errors);
      _checkI18n(d['content'], 'placement.content', errors);
      expect(errors, isEmpty, reason: errors.join('\n'));
    });

    test('a hook, five to seven exercises plus one piece, a recap', () {
      final types = blocks().map((b) => b['type'] as String).toList();
      expect(types.first, 'text');
      expect(types.last, 'text');
      final exercises = types
          .where(_interactive.contains)
          .where((t) => t != 'piece');
      expect(exercises.length, inInclusiveRange(5, 7));
      expect(types.where((t) => t == 'piece').length, 1);
    });

    test('every exercise probes a unit, in path order, covering them all', () {
      // Every probe — the exercises and the closing piece — is tagged.
      final tags = [
        for (final b in blocks())
          if (_interactive.contains(b['type']) || b['type'] == 'piece')
            b['placementUnit'],
      ];
      expect(tags, everyElement(isA<String>()), reason: 'untagged exercise');
      final units = _unitLevels.keys.toList();
      for (final t in tags) {
        expect(units, contains(t), reason: 'unknown unit tag $t');
      }
      final order = tags.map((t) => units.indexOf(t as String)).toList();
      for (var i = 1; i < order.length; i++) {
        expect(order[i] >= order[i - 1], isTrue, reason: 'tags out of order');
      }
      expect(tags.toSet(), units.toSet(), reason: 'every unit is probed');
      // The parser keeps the tags, by block index.
      final m = parseCourseManifest(jsonEncode(doc()['content']))!;
      expect(m.placementUnits.length, tags.length);
    });

    test('its piece is a bundled score the last unit allows', () {
      final piece = blocks().singleWhere((b) => b['type'] == 'piece');
      expect(piece['placementUnit'], _unitLevels.keys.last);
      final ref = (piece['score'] as Map)['bundled'] as String;
      expect(_bundledAssets, contains(ref), reason: 'unknown bundled $ref');
    });
  });

  test('a closing piece only uses what its unit has taught', () {
    for (final f in files) {
      final doc = jsonDecode(f.readAsStringSync()) as Map;
      final id = doc['id'] as String;
      final unit = doc['unit'] as String;
      final blocks = ((doc['content'] as Map)['blocks'] as List).cast<Map>();
      for (final b in blocks.where((b) => b['type'] == 'piece')) {
        final ref = (b['score'] as Map)['bundled'] as String;
        final asset = _bundledAssets[ref];
        expect(asset, isNotNull, reason: '$id: unknown bundled score $ref');
        final notes = _scanNotes(File('$_scoresDir/$asset').readAsStringSync());
        final allowed = _taught[unit]!;
        final hands = b['hands'] as String? ?? 'both';
        final staves = switch (hands) {
          'right' => {1},
          'left' => {2},
          _ => {1, 2},
        };
        if (staves.contains(2)) {
          expect(
            allowed.bass,
            isTrue,
            reason: '$id: $unit has not taught the bass clef yet',
          );
        }
        for (final n in notes.where((n) => staves.contains(n.staff))) {
          expect(
            allowed.figures,
            contains(n.type),
            reason: '$id: ${n.type} in $ref not taught by $unit',
          );
          if (n.dotted) {
            expect(
              allowed.dots,
              isTrue,
              reason: '$id: dotted ${n.type} in $ref not taught by $unit',
            );
          }
          if (n.rest) {
            expect(
              allowed.rests,
              isTrue,
              reason: '$id: rests in $ref not taught by $unit',
            );
            continue;
          }
          if (n.alter != 0) {
            expect(
              allowed.accidentals,
              isTrue,
              reason: '$id: accidental in $ref not taught by $unit',
            );
          }
          expect(
            n.midi >= allowed.lowest && n.midi <= allowed.highest,
            isTrue,
            reason:
                '$id: ${n.step}${n.octave} in $ref is outside '
                '${allowed.lowest}..${allowed.highest} taught by $unit',
          );
        }
        final fifths = _fifths(File('$_scoresDir/$asset').readAsStringSync());
        if (fifths != 0) {
          expect(
            allowed.keySignatures,
            isTrue,
            reason: '$id: key signature in $ref not taught by $unit',
          );
        }
      }
    }
  });
}

const _scoresDir = '../../apps/music/assets/scores';

/// Bundled score id → asset path (mirrors `_bundledEntries`).
const Map<String, String> _bundledAssets = {
  'au-clair-de-la-lune': 'beginner/au_clair_de_la_lune.musicxml',
  'mary-had-a-little-lamb': 'beginner/mary_had_a_little_lamb.musicxml',
  'twinkle-simple': 'beginner/twinkle_simple.musicxml',
  'frere-jacques': 'beginner/frere_jacques.musicxml',
  'ode-to-joy': 'beginner/ode_to_joy.musicxml',
  'twinkle': 'beginner/twinkle.musicxml',
  'minuet-in-g': 'intermediate/minuet_in_g.musicxml',
};

/// What each unit has taught by its end (cumulative), for the hands a piece
/// step imposes. Ranges are MIDI numbers (C4 = 60).
class _Taught {
  const _Taught({
    required this.figures,
    required this.dots,
    required this.rests,
    required this.bass,
    required this.accidentals,
    required this.keySignatures,
    required this.lowest,
    required this.highest,
  });
  final Set<String> figures;
  final bool dots;
  final bool rests;
  final bool bass;
  final bool accidentals;
  final bool keySignatures;
  final int lowest;
  final int highest;
}

const Map<String, _Taught> _taught = {
  // u1: treble C4–G4, plain quarters / halves / wholes.
  'u1': _Taught(
    figures: {'quarter', 'half', 'whole'},
    dots: false,
    rests: false,
    bass: false,
    accidentals: false,
    keySignatures: false,
    lowest: 60,
    highest: 67,
  ),
  // u2: + rests, the dotted half (3/4).
  'u2': _Taught(
    figures: {'quarter', 'half', 'whole'},
    dots: true,
    rests: true,
    bass: false,
    accidentals: false,
    keySignatures: false,
    lowest: 60,
    highest: 67,
  ),
  // u3: the whole treble staff and ledger lines.
  'u3': _Taught(
    figures: {'quarter', 'half', 'whole'},
    dots: true,
    rests: true,
    bass: false,
    accidentals: false,
    keySignatures: false,
    lowest: 55,
    highest: 84,
  ),
  // u4: + bass clef.
  'u4': _Taught(
    figures: {'quarter', 'half', 'whole'},
    dots: true,
    rests: true,
    bass: true,
    accidentals: false,
    keySignatures: false,
    lowest: 36,
    highest: 84,
  ),
  // u5: + eighths, dotted quarter, 6/8.
  'u5': _Taught(
    figures: {'quarter', 'half', 'whole', 'eighth'},
    dots: true,
    rests: true,
    bass: true,
    accidentals: false,
    keySignatures: false,
    lowest: 36,
    highest: 84,
  ),
  // u6: + accidentals and key signatures.
  'u6': _Taught(
    figures: {'quarter', 'half', 'whole', 'eighth'},
    dots: true,
    rests: true,
    bass: true,
    accidentals: true,
    keySignatures: true,
    lowest: 36,
    highest: 84,
  ),
  // u7: intervals, chords, the ear — same vocabulary, a real piece.
  'u7': _Taught(
    figures: {'quarter', 'half', 'whole', 'eighth'},
    dots: true,
    rests: true,
    bass: true,
    accidentals: true,
    keySignatures: true,
    lowest: 36,
    highest: 84,
  ),
};

class _ScannedNote {
  const _ScannedNote({
    required this.step,
    required this.octave,
    required this.alter,
    required this.type,
    required this.dotted,
    required this.rest,
    required this.staff,
  });
  final String step;
  final int octave;
  final int alter;
  final String type;
  final bool dotted;
  final bool rest;
  final int staff;

  int get midi {
    const base = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11};
    return 12 * (octave + 1) + base[step]! + alter;
  }
}

/// A deliberately small MusicXML scan — enough to hold a piece to a unit's
/// vocabulary without a parser: every `<note>` block's step / octave / alter /
/// type / dot / rest / staff. Grace notes and chords are notes like any other.
List<_ScannedNote> _scanNotes(String xml) {
  final out = <_ScannedNote>[];
  for (final m in RegExp(r'<note[\s>][\s\S]*?</note>').allMatches(xml)) {
    final n = m.group(0)!;
    String? tag(String t) =>
        RegExp('<$t>([^<]*)</$t>').firstMatch(n)?.group(1)?.trim();
    final type = tag('type');
    if (type == null) continue; // an unpitched / untyped event: not a figure
    out.add(
      _ScannedNote(
        step: tag('step') ?? 'C',
        octave: int.tryParse(tag('octave') ?? '') ?? 4,
        alter: int.tryParse(tag('alter') ?? '') ?? 0,
        type: type,
        dotted: n.contains('<dot/>') || n.contains('<dot '),
        rest:
            n.contains('<rest/>') ||
            n.contains('<rest>') ||
            n.contains('<rest '),
        staff: int.tryParse(tag('staff') ?? '') ?? 1,
      ),
    );
  }
  return out;
}

int _fifths(String xml) =>
    int.tryParse(
      RegExp(r'<fifths>\s*(-?\d+)\s*</fifths>').firstMatch(xml)?.group(1) ??
          '0',
    ) ??
    0;
