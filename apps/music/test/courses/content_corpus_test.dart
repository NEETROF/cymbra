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
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));

  test('the corpus exists and is the full 42-course curriculum', () {
    expect(dir.existsSync(), isTrue);
    expect(
      files.length,
      greaterThanOrEqualTo(42),
      reason: 'the solfège curriculum is 7 units × 6 lessons',
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
}
