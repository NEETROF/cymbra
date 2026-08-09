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
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../painters/notation_palette.dart';
import '../painters/partition_painter.dart';
import '../services/notation_engine.dart';
import '../src/rust/api/musicxml.dart' show ScoreDocument;
import '../theme/cymbra_theme.dart';

/// A course `score` block (change: add-notation-courses): an embedded notation
/// excerpt engraved from the manifest's inline MusicXML, reusing the app's
/// notation engine (parse) and [PartitionPainter] (render).
///
/// v1 renders the excerpt **statically**; the `playable` performance (embedding
/// the player/scoring so the user performs it) is a deferred refinement. Parsing
/// runs through the injectable `notationEngineProvider` seam, so tests supply a
/// hand-built document without the native library.
class ScoreBlockView extends ConsumerStatefulWidget {
  const ScoreBlockView({
    super.key,
    required this.musicXml,
    required this.playable,
    required this.prompt,
  });

  final String musicXml;
  final bool playable;
  final String prompt;

  @override
  ConsumerState<ScoreBlockView> createState() => _ScoreBlockViewState();
}

class _ScoreBlockViewState extends ConsumerState<ScoreBlockView> {
  ScoreDocument? _doc;
  bool _failed = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final doc = await ref
          .read(notationEngineProvider)
          .parse(Uint8List.fromList(utf8.encode(widget.musicXml)));
      if (mounted) setState(() => _doc = doc);
    } catch (_) {
      if (mounted) setState(() => _failed = true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (widget.prompt.isNotEmpty) ...[
          Text(
            widget.prompt,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 16,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 12),
        ],
        _engraving(),
      ],
    );
  }

  Widget _engraving() {
    if (_failed) {
      return const SizedBox(
        height: 96,
        child: Center(
          child: Icon(
            Icons.music_off_outlined,
            color: CymbraColors.onSurfaceVariant,
          ),
        ),
      );
    }
    final doc = _doc;
    if (doc == null) {
      return const SizedBox(
        height: 96,
        child: Center(child: CircularProgressIndicator()),
      );
    }
    return Container(
      decoration: BoxDecoration(
        color: NotationPalette.dark.background,
        borderRadius: BorderRadius.circular(12),
      ),
      clipBehavior: Clip.antiAlias,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final width = constraints.maxWidth;
          final painter = PartitionPainter(
            document: doc,
            systems: ref.read(notationEngineProvider).layout(doc, width),
          );
          final full = painter.heightFor(width);
          // Bound the visible height; a taller excerpt scrolls.
          final boxHeight = math.min(full, 280.0);
          return SizedBox(
            height: boxHeight,
            child: SingleChildScrollView(
              child: CustomPaint(size: Size(width, full), painter: painter),
            ),
          );
        },
      ),
    );
  }
}
