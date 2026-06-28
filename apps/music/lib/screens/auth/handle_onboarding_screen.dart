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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../services/auth_policy.dart';
import '../../services/auth_service.dart';
import '../../services/grpc_client.dart';
import '../../state/session_notifier.dart';
import '../../state/session_state.dart';
import '../../theme/cymbra_theme.dart';
import 'auth_messages.dart';
import 'auth_scaffold.dart';

/// Live validity of the candidate handle.
enum _HandleStatus { empty, invalid, checking, available, taken }

/// Blocking handle onboarding (spec: "Unique-handle onboarding"). Validates the
/// 1–15 letters/numbers policy client-side, debounces a `CheckHandleAvailability`
/// call, and commits with `UpdateAccount` — treating a write-time uniqueness
/// conflict (including a case-insensitive collision) as "pick another".
class HandleOnboardingScreen extends ConsumerStatefulWidget {
  const HandleOnboardingScreen({super.key});

  @override
  ConsumerState<HandleOnboardingScreen> createState() =>
      _HandleOnboardingScreenState();
}

class _HandleOnboardingScreenState
    extends ConsumerState<HandleOnboardingScreen> {
  final _handle = TextEditingController();
  Timer? _debounce;
  _HandleStatus _status = _HandleStatus.empty;
  bool _busy = false;

  /// Token guarding against a stale availability response overwriting a newer one.
  int _checkSeq = 0;

  @override
  void dispose() {
    _debounce?.cancel();
    _handle.dispose();
    super.dispose();
  }

  void _onChanged(String value) {
    _debounce?.cancel();
    final handle = value.trim();
    if (handle.isEmpty) {
      setState(() => _status = _HandleStatus.empty);
      return;
    }
    if (!isValidHandle(handle)) {
      setState(() => _status = _HandleStatus.invalid);
      return;
    }
    setState(() => _status = _HandleStatus.checking);
    final seq = ++_checkSeq;
    _debounce = Timer(const Duration(milliseconds: 400), () async {
      try {
        final available = await ref
            .read(accountServiceProvider)
            .checkHandleAvailability(handle);
        if (!mounted || seq != _checkSeq) return; // superseded
        setState(
          () => _status = available
              ? _HandleStatus.available
              : _HandleStatus.taken,
        );
      } on AuthException {
        if (!mounted || seq != _checkSeq) return;
        setState(() => _status = _HandleStatus.invalid);
      }
    });
  }

  /// Always-available escape from the gate (spec: "the handle gate is always
  /// escapable"). Discards a just-created account or signs an existing user out;
  /// either way the gate re-routes to the entry screen.
  Future<void> _abandon() async {
    setState(() => _busy = true);
    try {
      await ref.read(sessionNotifierProvider.notifier).abandonOnboarding();
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _commit() async {
    final handle = _handle.text.trim();
    if (_status != _HandleStatus.available) return;
    final session = ref.read(sessionNotifierProvider);
    final version = switch (session) {
      SessionAuthenticated(:final account) => account?.version ?? 0,
      _ => 0,
    };

    setState(() => _busy = true);
    try {
      final updated = await ref
          .read(accountServiceProvider)
          .updateHandle(handle: handle, expectedVersion: version);
      ref.read(sessionNotifierProvider.notifier).setAccount(updated);
      // The gate re-routes to the library once the account has a handle.
    } on AuthException catch (e) {
      if (!mounted) return;
      if (e.error == AuthError.alreadyExists || e.error == AuthError.conflict) {
        setState(() => _status = _HandleStatus.taken);
        showAuthError(
          context,
          e,
          fallback: 'That handle was just taken — please pick another.',
        );
      } else {
        showAuthError(context, e);
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  String? get _helperText => switch (_status) {
    _HandleStatus.invalid =>
      '1–15 letters or numbers only (no spaces or symbols).',
    _HandleStatus.taken => 'That handle is taken — try another.',
    _HandleStatus.available => 'Available!',
    _ => '1–15 letters or numbers.',
  };

  Color get _helperColor => switch (_status) {
    _HandleStatus.available => CymbraColors.tertiary,
    _HandleStatus.invalid || _HandleStatus.taken => CymbraColors.error,
    _ => CymbraColors.onSurfaceVariant,
  };

  @override
  Widget build(BuildContext context) {
    return AuthScaffold(
      title: 'Choose your handle',
      children: [
        const Text(
          'Your handle is how others will find you. You can use letters and '
          'numbers, up to $kHandleMaxLength characters.',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        TextField(
          key: const Key('handle-field'),
          controller: _handle,
          autocorrect: false,
          maxLength: kHandleMaxLength,
          onChanged: _onChanged,
          decoration: InputDecoration(
            labelText: 'Handle',
            prefixText: '@',
            helperText: _helperText,
            helperStyle: TextStyle(color: _helperColor),
            suffixIcon: _status == _HandleStatus.checking
                ? const Padding(
                    padding: EdgeInsets.all(12),
                    child: SizedBox.square(
                      dimension: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    ),
                  )
                : _status == _HandleStatus.available
                ? const Icon(Icons.check_circle, color: CymbraColors.tertiary)
                : null,
          ),
        ),
        const SizedBox(height: 24),
        FilledButton(
          key: const Key('handle-commit'),
          onPressed: (_busy || _status != _HandleStatus.available)
              ? null
              : _commit,
          child: _busy
              ? const SizedBox.square(
                  dimension: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Continue'),
        ),
        const SizedBox(height: 8),
        TextButton(
          key: const Key('handle-abandon'),
          onPressed: _busy ? null : _abandon,
          child: const Text('Use a different account'),
        ),
      ],
    );
  }
}
