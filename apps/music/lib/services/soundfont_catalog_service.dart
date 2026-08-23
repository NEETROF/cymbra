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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/score.pbgrpc.dart' as score;
import '../state/piano_catalog.dart';
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'soundfont_catalog_service.g.dart';

/// Seam over the backend's `ScoreService.ListSoundFonts` (change:
/// add-soundfont-catalog-db): the downloadable pianos the server actually hosts.
///
/// The app sources its `download`-kind catalog from here — never a hardcoded list
/// — so the picker only ever proposes fonts that exist on the server. Behind a
/// provider so tests inject a fake set (or an empty/error result) without a
/// backend.
abstract class SoundFontCatalogService {
  /// The server's downloadable pianos, as `download`-kind [PianoEntry]s. The
  /// bundled default is filtered out (it is offered as the bundled entry, not a
  /// download). Non-throwing: any failure (offline, unauthenticated, backend
  /// down) resolves to an **empty** list so the picker degrades to bundled +
  /// imports.
  Future<List<PianoEntry>> listDownloadable();
}

/// Production [SoundFontCatalogService] over the generated `ScoreServiceClient`,
/// authenticated through [AuthedRunner] like the other music RPCs.
class GrpcSoundFontCatalogService implements SoundFontCatalogService {
  GrpcSoundFontCatalogService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = score.ScoreServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<List<PianoEntry>> listDownloadable() async {
    try {
      return await _authed((bearer) async {
        final resp = await _client.listSoundFonts(
          score.ListSoundFontsRequest(),
          options: bearerOptions(bearer),
        );
        return resp.soundfonts
            // The bundled default is served locally, never as a download.
            .where((f) => f.id != defaultPianoId)
            .map(
              (f) => PianoEntry(
                id: f.id,
                label: f.label,
                kind: PianoKind.download,
                // The delivery-route font id (GET /soundfonts/{id}).
                source: f.id,
                license: f.license.isEmpty ? null : f.license,
                attribution: f.attribution.isEmpty ? null : f.attribution,
                // Opt-in public "proposé par" credit (change:
                // add-soundfont-uploader-attribution); empty when the uploader
                // has no public profile (or the font is seeded).
                contributorCredit: f.contributorCredit.isEmpty
                    ? null
                    : f.contributorCredit,
                // Whether a server preview clip exists (change:
                // add-soundfont-entitlement-previews) — greys a locked font's play
                // control up front when there is nothing to audition.
                hasPreview: f.hasPreview,
              ),
            )
            .toList();
      });
    } catch (_) {
      // Degrade: no downloadable fonts (picker still has bundled default + imports).
      return const [];
    }
  }
}

/// Production catalog-service provider. Override in tests with a fake.
@riverpod
SoundFontCatalogService soundFontCatalogService(Ref ref) =>
    GrpcSoundFontCatalogService(
      channel: ref.watch(cymbraChannelProvider),
      authed: ref.watch(authedRunnerProvider),
      deadlines: ref.watch(rpcDeadlinesProvider),
    );

/// The server's downloadable pianos, fetched once and cached by the provider.
/// Folded into [pianoCatalogProvider]; refresh by invalidating this provider.
@riverpod
Future<List<PianoEntry>> serverSoundFonts(Ref ref) =>
    ref.watch(soundFontCatalogServiceProvider).listDownloadable();
