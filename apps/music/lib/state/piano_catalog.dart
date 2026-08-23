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

/// Instrument family of a SoundFont, in the score vocabulary (change:
/// add-drum-audio-channel): which scores a font can sound. Decides which synth
/// channel its notes take and which picker offers it — a percussion score lists
/// `percussion` fonts, everything else `keyboard`.
enum SoundFamily {
  /// Melodic fonts (pianos, harpsichords…) — the historical catalog.
  keyboard,

  /// Drum-kit fonts, whose presets live in bank 128 and sound on the drum
  /// channel.
  percussion,
}

/// Maps the wire `SoundFont.instrument` value onto [SoundFamily] (change:
/// add-drum-audio-channel). `percussion` is the only value that selects the
/// kit family; `keyboard` — and the legacy `piano` spelling still served by
/// not-yet-migrated backends — map to [SoundFamily.keyboard], as does anything
/// unknown (fail-open to the historical family, never to a silent kit).
SoundFamily soundFamilyFromWire(String instrument) =>
    instrument.trim().toLowerCase() == SoundFamily.percussion.name
    ? SoundFamily.percussion
    : SoundFamily.keyboard;

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
    this.family = SoundFamily.keyboard,
    this.license,
    this.attribution,
    this.contributorCredit,
    this.remoteId,
    this.proposalStatus,
    this.proposalRejectionReason,
    this.hasPreview = false,
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
    // Registries persisted before the family existed hold keyboard fonts only
    // (imports were always pianos), so absence decodes to keyboard.
    family:
        SoundFamily.values.asNameMap()[json['family'] as String?] ??
        SoundFamily.keyboard,
    license: json['license'] as String?,
    attribution: json['attribution'] as String?,
    contributorCredit: json['contributorCredit'] as String?,
    remoteId: json['remoteId'] as String?,
    proposalStatus: json['proposalStatus'] as String?,
    proposalRejectionReason: json['proposalRejectionReason'] as String?,
    hasPreview: json['hasPreview'] as bool? ?? false,
  );

  final String id;
  final String label;
  final PianoKind kind;
  final String source;

  /// Instrument family (change: add-drum-audio-channel): sourced from the
  /// server listing for catalog fonts, detected from preset banks for imports.
  /// Drives the family-scoped picker and the per-family selection memory.
  final SoundFamily family;

  final String? license;
  final String? attribution;

  /// Opt-in public "proposé par" credit of a user-contributed catalog font
  /// (change: add-soundfont-uploader-attribution): the uploader's public
  /// handle/display name, or `null` when they haven't opted into a public
  /// profile. Distinct from [attribution] (the licence's sample author).
  final String? contributorCredit;

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

  /// The moderator's motive when [proposalStatus] is `rejected` (change:
  /// add-soundfont-uploader-attribution); `null` otherwise. Sourced from the
  /// private-library sync, shown so the uploader knows why it was refused.
  final String? proposalRejectionReason;

  /// Whether the server has a rendered preview clip for this font (change:
  /// add-soundfont-entitlement-previews). Sourced from the catalog listing; drives
  /// the up-front greying of a **locked** font's play control (no preview → nothing to
  /// audition). Defaults to `false` for bundled/imported fonts (they play locally).
  final bool hasPreview;

  /// A copy with individual fields replaced (only what the sync flow needs).
  /// `proposalRejectionReason` is nullable-aware (a re-proposal clears it), so it
  /// replaces rather than coalesces.
  PianoEntry copyWith({
    String? label,
    String? source,
    String? remoteId,
    String? proposalStatus,
    Object? proposalRejectionReason = _unset,
    bool? hasPreview,
  }) => PianoEntry(
    id: id,
    label: label ?? this.label,
    kind: kind,
    source: source ?? this.source,
    family: family,
    license: license,
    attribution: attribution,
    contributorCredit: contributorCredit,
    remoteId: remoteId ?? this.remoteId,
    proposalStatus: proposalStatus ?? this.proposalStatus,
    proposalRejectionReason: proposalRejectionReason == _unset
        ? this.proposalRejectionReason
        : proposalRejectionReason as String?,
    hasPreview: hasPreview ?? this.hasPreview,
  );

  static const Object _unset = Object();

  Map<String, dynamic> toJson() => {
    'id': id,
    'label': label,
    'kind': kind.name,
    'source': source,
    // Keyboard is the implicit default (registries predating the family decode
    // to it), so only the non-default family is written.
    if (family != SoundFamily.keyboard) 'family': family.name,
    if (license != null) 'license': license,
    if (attribution != null) 'attribution': attribution,
    if (contributorCredit != null) 'contributorCredit': contributorCredit,
    if (remoteId != null) 'remoteId': remoteId,
    if (proposalStatus != null) 'proposalStatus': proposalStatus,
    if (proposalRejectionReason != null)
      'proposalRejectionReason': proposalRejectionReason,
    if (hasPreview) 'hasPreview': hasPreview,
  };

  @override
  bool operator ==(Object other) =>
      other is PianoEntry &&
      other.id == id &&
      other.label == label &&
      other.kind == kind &&
      other.source == source &&
      other.family == family &&
      other.license == license &&
      other.attribution == attribution &&
      other.contributorCredit == contributorCredit &&
      other.remoteId == remoteId &&
      other.proposalStatus == proposalStatus &&
      other.proposalRejectionReason == proposalRejectionReason &&
      other.hasPreview == hasPreview;

  @override
  int get hashCode => Object.hash(
    id,
    label,
    kind,
    source,
    family,
    license,
    attribution,
    contributorCredit,
    remoteId,
    proposalStatus,
    proposalRejectionReason,
    hasPreview,
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

/// Stable id of the bundled drum kit (change: add-drum-audio-channel): the
/// percussion family's default and fallback, and the id the same kit is seeded
/// under in the server catalog — the FluidR3 GM percussion bank named in the
/// change's design. The id is defined ahead of the asset: it is what the kit
/// selection defaults and self-heals to, so it must be stable **before** the
/// bytes land.
const String defaultKitId = 'fluid-r3-drums';

/// The built-in drum kits (change: add-drum-audio-channel). **Empty until the
/// bundled kit's licence sign-off lands the asset** (tasks 6.1/6.2/9.1): a
/// catalog must never point at bytes that do not exist, so with no entry here a
/// percussion score with no other resolvable kit stays honestly visual-only
/// (the readiness gate never resolves). When the asset lands, this becomes the
/// one-line entry mirroring [builtInPianos]:
/// `PianoEntry(id: defaultKitId, label: 'FluidR3 Drums', kind:
/// PianoKind.bundled, source: 'assets/soundfonts/FILE.sf2', family:
/// SoundFamily.percussion, license: 'MIT')`.
const List<PianoEntry> builtInKits = [];

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
  final builtIn = [...builtInPianos, ...builtInKits];
  final builtInIds = builtIn.map((p) => p.id).toSet();
  final downloadOnly = download.where((p) => !builtInIds.contains(p.id));
  return [...builtIn, ...downloadOnly, ...imported];
}
