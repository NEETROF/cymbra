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

import '../theme/cymbra_theme.dart';

/// How long a toast stays before dismissing itself.
const Duration kToastDuration = Duration(seconds: 4);

/// Widest a toast gets; beyond this it would read as a banner rather than a
/// transient confirmation.
const double _maxToastWidth = 420;

/// The single live toast, so a new message replaces the previous one instead of
/// stacking. Module-level because there is at most one toast in the app at a
/// time — it belongs to the screen, not to any widget.
OverlayEntry? _entry;
Timer? _timer;

/// Shows [message] as a snackbar, **replacing** any current or queued one first
/// (so messages never stack up) and with a **close button** so the user can
/// dismiss it manually.
///
/// Takes a [ScaffoldMessengerState] rather than a `BuildContext` so it is safe to
/// call after an `await` (capture `ScaffoldMessenger.of(context)` before the gap).
///
/// A snackbar is painted by the Scaffold of the route that registered it, so a
/// dialog or bottom sheet above that route **covers** it. When the message has to
/// be seen over a modal — or to outlive the surface that triggered it — use
/// [showAppToast] instead.
void showAppSnackBar(ScaffoldMessengerState messenger, String message) {
  messenger
    ..clearSnackBars()
    ..showSnackBar(SnackBar(content: Text(message), showCloseIcon: true));
}

/// Shows [message] as a **toast**: a floating card pinned to the bottom-right,
/// dismissing itself after [kToastDuration], **replacing** any toast already on
/// screen, and carrying a close button for manual dismissal.
///
/// Inserted into the **root overlay**, which sits above every route — so it is
/// visible over the end-of-run summary dialog and the exit sheet, and it survives
/// the route that triggered it being popped. Pass
/// `Overlay.of(context, rootOverlay: true)`; capture it before any `await`, since
/// the caller's own context may be gone by the time the message is ready.
void showAppToast(OverlayState overlay, String message) {
  dismissAppToast();
  final entry = OverlayEntry(
    builder: (_) => AppToast(message: message, onClose: dismissAppToast),
  );
  _entry = entry;
  overlay.insert(entry);
  _timer = Timer(kToastDuration, dismissAppToast);
}

/// Removes the live toast, if any. Idempotent, so the auto-dismiss timer and the
/// close button can both call it.
void dismissAppToast() {
  _timer?.cancel();
  _timer = null;
  _entry?.remove();
  _entry = null;
}

/// The toast card: bottom-right, fading and rising into place.
class AppToast extends StatefulWidget {
  const AppToast({super.key, required this.message, required this.onClose});

  final String message;
  final VoidCallback onClose;

  @override
  State<AppToast> createState() => _AppToastState();
}

class _AppToastState extends State<AppToast>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 180),
  )..forward();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Positioned(
    right: 16,
    bottom: 16,
    child: SafeArea(
      child: FadeTransition(
        opacity: _controller,
        child: SlideTransition(
          position: Tween<Offset>(begin: const Offset(0, 0.3), end: Offset.zero)
              .animate(
                CurvedAnimation(parent: _controller, curve: Curves.easeOut),
              ),
          child: Material(
            color: CymbraColors.surfaceContainerHigh,
            elevation: 6,
            borderRadius: BorderRadius.circular(12),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: _maxToastWidth),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 10, 8, 10),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Flexible(
                      child: Text(
                        widget.message,
                        style: const TextStyle(
                          color: CymbraColors.onSurface,
                          fontSize: 14,
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    IconButton(
                      icon: const Icon(Icons.close, size: 18),
                      color: CymbraColors.onSurfaceVariant,
                      visualDensity: VisualDensity.compact,
                      tooltip: MaterialLocalizations.of(
                        context,
                      ).closeButtonTooltip,
                      onPressed: widget.onClose,
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}
