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

import 'dart:typed_data';

import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/sound_clip_player.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_preview_service.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/piano_catalog.dart';

/// In-memory [SoundFontSource]: resolves each entry to a deterministic fake path
/// and records the entries it was asked for. Ids in [failIds] throw
/// [SoundFontUnavailableException] so tests can drive the download/missing-file
/// fallback without touching assets, network, or the filesystem.
class FakeSoundFontSource implements SoundFontSource {
  FakeSoundFontSource({this.failIds = const {}});

  final Set<String> failIds;
  final List<PianoEntry> resolved = [];

  @override
  Future<String> resolve(PianoEntry entry) async {
    resolved.add(entry);
    if (failIds.contains(entry.id)) {
      throw SoundFontUnavailableException('forced failure for ${entry.id}');
    }
    return '/fake/soundfonts/${entry.id}.sf2';
  }
}

/// In-memory [SoundFontImporter]: returns a canned entry (or `null` to model a
/// cancel), or throws [SoundFontImportException] to model an invalid file.
/// Records deletions so a test can assert the copied file was cleaned up.
class FakeSoundFontImporter implements SoundFontImporter {
  FakeSoundFontImporter({this.next, this.throwInvalid = false, this.picked});

  /// The entry a successful import returns; `null` models the user cancelling.
  PianoEntry? next;

  /// What [pick] returns (a chosen file), or `null` for a cancel.
  PickedSoundFont? picked;

  /// When true, [importSoundFont]/[pick] throw [SoundFontImportException].
  bool throwInvalid;

  int importCalls = 0;
  int pickCalls = 0;
  final List<({Uint8List bytes, String label})> saved = [];
  final List<PianoEntry> deleted = [];

  @override
  Future<PickedSoundFont?> pick() async {
    pickCalls++;
    if (throwInvalid) throw const SoundFontImportException();
    return picked;
  }

  @override
  Future<PianoEntry> save(Uint8List bytes, String label) async {
    saved.add((bytes: bytes, label: label));
    return PianoEntry(
      id: 'saved-${saved.length}',
      label: label,
      kind: PianoKind.user,
      source: '/saved/${saved.length}.sf2',
    );
  }

  @override
  Future<PianoEntry?> importSoundFont() async {
    importCalls++;
    if (throwInvalid) throw const SoundFontImportException();
    return next;
  }

  @override
  Future<void> deleteImport(PianoEntry entry) async => deleted.add(entry);
}

/// In-memory [SoundFontCatalogService]: returns a fixed set of `download`-kind
/// pianos (the server's list). The production service already swallows errors to
/// an empty list, so tests model "listing unavailable" with `downloadable: []`.
class FakeSoundFontCatalogService implements SoundFontCatalogService {
  FakeSoundFontCatalogService({this.downloadable = const []});

  List<PianoEntry> downloadable;
  int listCalls = 0;

  @override
  Future<List<PianoEntry>> listDownloadable() async {
    listCalls++;
    return downloadable;
  }
}

/// In-memory [PrivateSoundFontService]: models the per-user server library.
/// Records imports/proposals/deletions so tests can assert the sync flow without
/// network or a token store. `import` is idempotent by label (stands in for the
/// server's content dedup) and assigns a deterministic `remote-N` id.
class FakePrivateSoundFontService implements PrivateSoundFontService {
  FakePrivateSoundFontService({List<RemoteSoundFont>? library})
    : library = [...?library];

  final List<RemoteSoundFont> library;
  final List<String> imported = [];
  final List<
    ({
      String id,
      String license,
      String attribution,
      bool attestation,
      String? resubmissionNote,
    })
  >
  proposed = [];
  final List<String> deleted = [];
  bool failImport = false;
  int _seq = 0;

  @override
  Future<List<RemoteSoundFont>> list() async => List.of(library);

  /// Families declared by sync calls, in order (assertable by tests).
  final List<String?> importedFamilies = [];

  @override
  Future<RemoteSoundFont> import(
    Uint8List bytes,
    String label, {
    String? family,
  }) async {
    importedFamilies.add(family);
    if (failImport) {
      throw const PrivateSoundFontException('forced import failure');
    }
    imported.add(label);
    final existing = library.where((f) => f.label == label);
    if (existing.isNotEmpty) return existing.first;
    final font = RemoteSoundFont(
      id: 'remote-${_seq++}',
      label: label,
      sizeBytes: bytes.length,
    );
    library.add(font);
    return font;
  }

  @override
  Future<Uint8List> download(String id) async =>
      Uint8List.fromList('RIFF____sfbk$id'.codeUnits);

  @override
  Future<void> delete(String id) async {
    deleted.add(id);
    library.removeWhere((f) => f.id == id);
  }

  @override
  Future<void> propose(
    String id, {
    required String license,
    String attribution = '',
    required bool attestation,
    String? resubmissionNote,
  }) async {
    proposed.add((
      id: id,
      license: license,
      attribution: attribution,
      attestation: attestation,
      resubmissionNote: resubmissionNote,
    ));
  }
}

/// In-memory [SoundFontPreviewService]: records the ids auditioned and reports
/// preview availability from [available]; ids in [failIds] throw. Used to drive the
/// locked-font audition path without network or an audio device.
class FakeSoundFontPreviewService implements SoundFontPreviewService {
  FakeSoundFontPreviewService({
    this.available = const {},
    this.failIds = const {},
  });

  /// Ids that HAVE a preview clip — [audition] returns `true` (playback started);
  /// any other id returns `false` (no preview yet).
  final Set<String> available;

  /// Ids whose fetch fails — [audition] throws [SoundFontPreviewException].
  final Set<String> failIds;

  final List<String> auditioned = [];
  int stopCalls = 0;

  @override
  Future<bool> audition(String fontId) async {
    auditioned.add(fontId);
    if (failIds.contains(fontId)) {
      throw const SoundFontPreviewException('forced failure');
    }
    return available.contains(fontId);
  }

  @override
  Future<void> stop() async => stopCalls++;
}

/// In-memory [SoundClipPlayer]: records the clips it was asked to play + stop calls,
/// so the preview service's fetch/play wiring is testable without an audio device.
class FakeSoundClipPlayer implements SoundClipPlayer {
  final List<Uint8List> played = [];
  int stopCalls = 0;

  @override
  Future<void> play(Uint8List bytes) async => played.add(bytes);

  @override
  Future<void> stop() async => stopCalls++;
}

/// A `download`-kind [PianoEntry] as the server catalog would yield.
PianoEntry fakeDownloadPiano({
  required String id,
  required String label,
  SoundFamily family = SoundFamily.keyboard,
  String license = 'CC-BY 3.0',
  String? attribution,
  String? contributorCredit,
  bool hasPreview = false,
}) => PianoEntry(
  id: id,
  label: label,
  kind: PianoKind.download,
  source: id,
  family: family,
  license: license,
  attribution: attribution,
  contributorCredit: contributorCredit,
  hasPreview: hasPreview,
);

/// A user-kind [PianoEntry] for tests.
PianoEntry fakeUserPiano({
  String id = 'user-1',
  String label = 'My Piano',
  String source = '/copied/user-1.sf2',
}) => PianoEntry(id: id, label: label, kind: PianoKind.user, source: source);
