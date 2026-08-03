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

import 'package:flutter/foundation.dart' show immutable;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/soundfont_catalog_service.dart';
import 'imported_soundfonts.dart';

part 'piano_catalog.g.dart';

/// Where a piano's SoundFont bytes come from — which decides how
/// `SoundFontSource` resolves it to a local file.
enum PianoKind {
  /// Shipped in the app bundle as an asset; always available, no network.
  bundled,

  /// Fetched once from the backend SoundFont-delivery route and cached
  /// (change `add-soundfont-delivery`); until fetched it falls back to default.
  download,

  /// Imported by the user from their device and copied into app storage.
  user,
}

/// A selectable piano in the catalog: a stable [id], a display [label], its
/// [kind], and a [source] whose meaning depends on the kind — a bundled asset
/// path, a download font id (the delivery-route key), or an imported file path.
/// [license]/[attribution] carry the credit a CC-BY font must surface.
@immutable
class PianoEntry {
  const PianoEntry({
    required this.id,
    required this.label,
    required this.kind,
    required this.source,
    this.license,
    this.attribution,
    this.remoteId,
    this.proposalStatus,
  });

  /// Hand-rolled JSON (the repo does not use `json_serializable`), used to
  /// persist the imported registry. Only `user` pianos are ever serialized, so
  /// the kind round-trips by name with a safe default.
  factory PianoEntry.fromJson(Map<String, dynamic> json) => PianoEntry(
    id: json['id'] as String,
    label: json['label'] as String,
    kind:
        PianoKind.values.asNameMap()[json['kind'] as String?] ?? PianoKind.user,
    source: json['source'] as String,
    license: json['license'] as String?,
    attribution: json['attribution'] as String?,
    remoteId: json['remoteId'] as String?,
    proposalStatus: json['proposalStatus'] as String?,
  );

  final String id;
  final String label;
  final PianoKind kind;
  final String source;
  final String? license;
  final String? attribution;

  /// Id of this font in the user's **private server library**, once uploaded
  /// (change: add-soundfont-moderation). Set for `user` pianos that have synced;
  /// `null` for a purely local import or a non-user piano. Drives cross-device
  /// sync and the "propose to catalog" action.
  final String? remoteId;

  /// Moderation status of this font's public-catalog proposal, or `null` when not
  /// proposed (change: add-soundfont-moderation): `pending`/`accepted`/`rejected`.
  /// Persisted (so a submitted proposal keeps its tag across relaunches) and
  /// upgraded by the sync from the server (`pending` → `accepted`/`rejected`).
  final String? proposalStatus;

  /// A copy with individual fields replaced (only what the sync flow needs).
  PianoEntry copyWith({
    String? label,
    String? source,
    String? remoteId,
    String? proposalStatus,
  }) => PianoEntry(
    id: id,
    label: label ?? this.label,
    kind: kind,
    source: source ?? this.source,
    license: license,
    attribution: attribution,
    remoteId: remoteId ?? this.remoteId,
    proposalStatus: proposalStatus ?? this.proposalStatus,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'label': label,
    'kind': kind.name,
    'source': source,
    if (license != null) 'license': license,
    if (attribution != null) 'attribution': attribution,
    if (remoteId != null) 'remoteId': remoteId,
    if (proposalStatus != null) 'proposalStatus': proposalStatus,
  };

  @override
  bool operator ==(Object other) =>
      other is PianoEntry &&
      other.id == id &&
      other.label == label &&
      other.kind == kind &&
      other.source == source &&
      other.license == license &&
      other.attribution == attribution &&
      other.remoteId == remoteId &&
      other.proposalStatus == proposalStatus;

  @override
  int get hashCode => Object.hash(
    id,
    label,
    kind,
    source,
    license,
    attribution,
    remoteId,
    proposalStatus,
  );
}

/// Id of the bundled CC0 default piano — the fallback whenever a chosen piano
/// is unavailable, and the instrument `audio_init` loads at startup.
const String defaultPianoId = 'upright-piano-kw';

/// Asset path of the bundled CC0 default SoundFont (registered in
/// `pubspec.yaml`; see `assets/soundfonts/CREDITS.md`).
const String defaultPianoAsset =
    'assets/soundfonts/UprightPianoKW-20220221.sf2';

/// The built-in pianos: only the bundled CC0 default (always present, no
/// network). Downloadable pianos are **not** hardcoded here — they come from the
/// server catalog listing (`serverSoundFontsProvider`, change
/// add-soundfont-catalog-db) so the app never proposes a font the server does not
/// actually host.
const List<PianoEntry> builtInPianos = [
  PianoEntry(
    id: defaultPianoId,
    label: 'Upright Piano KW',
    kind: PianoKind.bundled,
    source: defaultPianoAsset,
    license: 'CC0 1.0',
  ),
];

/// The default piano entry — always the first built-in.
PianoEntry get defaultPiano => builtInPianos.first;

/// The full catalog of selectable pianos: the bundled default, plus the server's
/// downloadable fonts, plus the persisted user-imported registry. Synchronous and
/// reactive — it rebuilds when the server list resolves or an import is
/// added/removed, so the picker and the selection notifier stay in sync.
///
/// Both async sources degrade to empty until they resolve (or on error), so the
/// picker always at least offers the bundled default. Override
/// [serverSoundFontsProvider] / [importedSoundFontsProvider] in tests.
@riverpod
List<PianoEntry> pianoCatalog(Ref ref) {
  // The server download list + imports both load asynchronously; until they
  // resolve (or if they fail) the catalog is just the bundled default.
  final download = ref.watch(serverSoundFontsProvider).valueOrNull ?? const [];
  final imported =
      ref.watch(importedSoundFontsProvider).valueOrNull ?? const [];
  // A server entry that shares a built-in's id (e.g. the bundled default, also in
  // the server catalog) is dropped so it is never listed twice.
  final builtInIds = builtInPianos.map((p) => p.id).toSet();
  final downloadOnly = download.where((p) => !builtInIds.contains(p.id));
  return [...builtInPianos, ...downloadOnly, ...imported];
}
