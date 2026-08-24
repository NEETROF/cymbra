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

import 'package:flutter_riverpod/flutter_riverpod.dart' show Ref;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/preferences_service.dart';
import 'drums_access.dart';

part 'instrument_context.freezed.dart';
part 'instrument_context.g.dart';

/// The instrument the user is here for (change: add-instrument-context) —
/// what the discovery surfaces seed from. NOT what the player reads: a score
/// carries its own instrument, and the context never governs it.
enum AppInstrument { keyboard, drums }

/// The persisted context record: the user's choice plus the durable
/// "choice already offered" marker. Per-installation, so both survive a
/// relaunch AND a sign-out/sign-in cycle (the naive per-session marker
/// re-prompts on every sign-in).
@freezed
abstract class InstrumentContextState with _$InstrumentContextState {
  const factory InstrumentContextState({
    /// The stored choice. **Sticky**: it changes only when the user changes
    /// it — opening, playing or finishing a score never writes it, and a
    /// visibility change never rewrites it (the fallback is presentational).
    @Default(AppInstrument.keyboard) AppInstrument context,

    /// Whether the one-time choice has already been offered on this
    /// installation.
    @Default(false) bool choiceOffered,

    /// Runtime-only (never persisted): whether the stored record has been
    /// read back. The choice modal MUST wait for it — checking [choiceOffered]
    /// before hydration reads the default `false` and re-offers on every
    /// launch.
    @Default(false) bool hydrated,
  }) = _InstrumentContextState;
}

/// Device-persisted instrument context. Seeded synchronously with defaults so
/// the first frame never blocks, then reconciled against storage — the same
/// pattern as the play preferences.
@Riverpod(keepAlive: true)
class InstrumentContext extends _$InstrumentContext {
  static const String prefsKey = 'instrument_context';

  @override
  InstrumentContextState build() {
    _restore();
    return const InstrumentContextState();
  }

  Future<void> _restore() async {
    try {
      final raw = await ref
          .read(preferencesServiceProvider)
          .getString(prefsKey);
      if (raw != null) {
        final m = jsonDecode(raw) as Map<String, dynamic>;
        state = state.copyWith(
          context:
              AppInstrument.values.asNameMap()[m['context'] as String?] ??
              AppInstrument.keyboard,
          choiceOffered: m['choiceOffered'] as bool? ?? false,
        );
      }
    } catch (_) {
      // storage unavailable / corrupt value → defaults stand
    }
    // Hydrated even on failure: the defaults ARE the answer then, and the
    // choice modal must not wait forever.
    state = state.copyWith(hydrated: true);
  }

  /// The user's explicit choice — the ONLY thing that writes the context.
  void select(AppInstrument instrument) =>
      _update(state.copyWith(context: instrument));

  /// Record that the one-time choice was offered (whatever was answered).
  void markChoiceOffered() => _update(state.copyWith(choiceOffered: true));

  void _update(InstrumentContextState next) {
    if (next == state) return;
    state = next;
    _persist(next);
  }

  Future<void> _persist(InstrumentContextState s) async {
    try {
      await ref
          .read(preferencesServiceProvider)
          .setString(
            prefsKey,
            jsonEncode({
              'context': s.context.name,
              'choiceOffered': s.choiceOffered,
            }),
          );
    } catch (_) {
      // Best-effort: the in-memory value still applies this session.
    }
  }
}

/// The context the surfaces PRESENT: the stored choice, falling back to
/// keyboard while drums are not currently visible — a campaign closed, or a
/// flag snapshot not yet resolved (which happens on EVERY cold start and after
/// every sign-out). Presentational only: the stored value is never rewritten,
/// so drums reapply the moment visibility returns.
@riverpod
AppInstrument effectiveInstrumentContext(Ref ref) {
  final stored = ref.watch(instrumentContextProvider.select((s) => s.context));
  final drumsVisible = ref.watch(drumsEnabledProvider);
  return drumsVisible ? stored : AppInstrument.keyboard;
}
