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
import 'dart:typed_data';

import 'package:music/services/score_asset_source.dart';
import 'package:music/state/score_catalog.dart';

/// A self-contained MusicXML fixture for the end-to-end test.
///
/// The integration test must not depend on the app's *shipping* scores — those
/// change independently (titles, contents) and would break the test for reasons
/// unrelated to the app's behaviour. This fixture is owned by the test, lives
/// outside `assets/` (it is never bundled into the app), and is still parsed and
/// laid out by the **real** Rust bridge, exactly like a shipped score.
///
/// `<work-title>` is "Ode to Joy" so the player header ("Now Playing: …") shows
/// it; the catalog entry below is what the library lists.
const String kFixtureScoreXml = '''<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="3.1">
  <work><work-title>Ode to Joy</work-title></work>
  <identification><creator type="composer">Beethoven</creator></identification>
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
      <note><pitch><step>F</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
      <note><pitch><step>G</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
    </measure>
    <measure number="2">
      <note><pitch><step>G</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
      <note><pitch><step>F</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
      <note><pitch><step>E</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
      <note><pitch><step>D</step><octave>4</octave></pitch><duration>1</duration><type>quarter</type></note>
    </measure>
  </part>
</score-partwise>''';

/// The single library entry the end-to-end test selects. Its `assetPath` is a
/// sentinel — [FixtureScoreAssetSource] ignores it and returns the fixture.
const CatalogEntry kFixtureCatalogEntry = CatalogEntry(
  id: 'fixture-ode',
  title: 'Ode to Joy (theme)',
  composer: 'Beethoven',
  assetPath: 'fixture://ode-to-joy',
  level: PracticeLevel.beginner,
);

/// A [ScoreAssetSource] that serves [kFixtureScoreXml] for any path, so the test
/// drives the real parse/layout without touching the asset bundle.
class FixtureScoreAssetSource implements ScoreAssetSource {
  const FixtureScoreAssetSource();

  @override
  Future<Uint8List> load(String assetPath) async =>
      Uint8List.fromList(utf8.encode(kFixtureScoreXml));
}
