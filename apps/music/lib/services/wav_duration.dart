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

/// The playing time of a canonical PCM WAV buffer (44-byte RIFF/WAVE header:
/// channels @22, sample rate @24, bits per sample @34, data size @40), or `null`
/// when [bytes] is not such a WAV. Pure — lets the clip player, which loops a clip
/// by design, be stopped after exactly one pass (change:
/// add-score-daily-access-rewards).
Duration? wavDuration(Uint8List bytes) {
  if (bytes.length < 44) return null;
  final b = ByteData.sublistView(bytes);
  if (String.fromCharCodes(bytes.sublist(0, 4)) != 'RIFF' ||
      String.fromCharCodes(bytes.sublist(8, 12)) != 'WAVE') {
    return null;
  }
  final channels = b.getUint16(22, Endian.little);
  final sampleRate = b.getUint32(24, Endian.little);
  final bitsPerSample = b.getUint16(34, Endian.little);
  final dataSize = b.getUint32(40, Endian.little);
  final frameBytes = channels * (bitsPerSample ~/ 8);
  if (channels == 0 || sampleRate == 0 || frameBytes == 0) return null;
  final frames = dataSize ~/ frameBytes;
  return Duration(microseconds: frames * 1000000 ~/ sampleRate);
}
