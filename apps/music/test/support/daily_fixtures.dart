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

import 'dart:convert';

import 'package:music/courses/course_manifest.dart';
import 'package:music/services/course_catalog_service.dart';

/// A small learning path for the daily-session tests (change:
/// add-music-daily-session): three units, drills in the first two, a piece
/// step closing units one and two and the last lesson.
///
/// | id | unit | blocks                 |
/// |----|------|------------------------|
/// | a1 | u1   | text, readPlay         |
/// | a2 | u1   | text, rhythmTap        |
/// | a3 | u1   | text, piece            |
/// | b1 | u2   | text, earChoice        |
/// | b2 | u2   | text, piece            |
/// | c1 | u3   | text                   |
/// | c2 | u3   | text, piece            |
CourseListing dailyListing(String id, String unit, int order) => CourseListing(
  id: id,
  schemaVersion: 2,
  unit: unit,
  unitTitle: {'en': 'Unit $unit'},
  sortOrder: order,
  title: {'en': 'Lesson $id'},
);

final List<CourseListing> dailyListings = [
  dailyListing('a1', 'u1', 101),
  dailyListing('a2', 'u1', 102),
  dailyListing('a3', 'u1', 103),
  dailyListing('b1', 'u2', 201),
  dailyListing('b2', 'u2', 202),
  dailyListing('c1', 'u3', 301),
  dailyListing('c2', 'u3', 302),
];

const Map<String, Object> _text = {
  'type': 'text',
  'text': {'en': 'Read this.'},
};

const Map<String, Object> _readPlay = {
  'type': 'readPlay',
  'mode': 'drill',
  'clef': 'treble',
  'labels': 'always',
  'notes': ['G4'],
  'prompt': {'en': 'Play the G'},
};

const Map<String, Object> _rhythmTap = {
  'type': 'rhythmTap',
  'pattern': [
    {'fig': 'quarter'},
    {'fig': 'quarter'},
  ],
  'beats': 4,
  'beatType': 4,
  'bpm': 70,
  'prompt': {'en': 'Tap the beat'},
};

const Map<String, Object> _earChoice = {
  'type': 'earChoice',
  'notes': ['C3', 'C5'],
  'answerId': 'up',
  'choices': [
    {
      'id': 'up',
      'label': {'en': 'Higher'},
    },
    {
      'id': 'down',
      'label': {'en': 'Lower'},
    },
  ],
  'prompt': {'en': 'Higher or lower?'},
};

Map<String, Object> _piece(String id) => {
  'type': 'piece',
  'score': {'bundled': 'ode-to-joy'},
  'hands': 'right',
  'speed': 0.6,
  'waitMode': true,
  'passPct': 80,
  'title': {'en': 'Piece of $id'},
};

/// The raw manifest JSON of [id], or null for an unknown id.
String? dailyManifestJson(String id) {
  final blocks = switch (id) {
    'a1' => [_text, _readPlay],
    'a2' => [_text, _rhythmTap],
    'a3' => [_text, _piece('a3')],
    'b1' => [_text, _earChoice],
    'b2' => [_text, _piece('b2')],
    'c1' => [_text],
    'c2' => [_text, _piece('c2')],
    _ => null,
  };
  if (blocks == null) return null;
  return jsonEncode({
    'schemaVersion': 2,
    'id': id,
    'title': {'en': 'Lesson $id'},
    'blocks': blocks,
  });
}

/// The parsed manifests of every lesson, by id.
Map<String, CourseManifest> dailyManifests([Iterable<String>? only]) => {
  for (final l in dailyListings)
    if (only == null || only.contains(l.id))
      l.id: parseCourseManifest(dailyManifestJson(l.id)!)!,
};

/// A catalogue service serving the fixture path.
class DailyFakeCatalog implements CourseCatalogService {
  const DailyFakeCatalog([this.listings]);

  final List<CourseListing>? listings;

  @override
  Future<List<CourseListing>> listCourses() async => listings ?? dailyListings;

  @override
  Future<String?> getCourseManifestJson(String id) async =>
      dailyManifestJson(id);
}
