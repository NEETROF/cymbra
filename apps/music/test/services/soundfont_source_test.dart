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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/grpc_client.dart';
import 'package:music/services/soundfont_source.dart';

void main() {
  group('soundFontDeliveryOrigin', () {
    test('derives the dev HTTP port for a plaintext endpoint', () {
      // gRPC dev is localhost:50051, but the HTTP delivery route is on :8081 — a
      // different port. Deriving from the host alone (port 80) would 404/refuse and
      // silently revert the piano selection, so the dev origin must carry :8081.
      const ep = CymbraEndpoint(host: 'localhost', port: 50051, secure: false);
      expect(soundFontDeliveryOrigin(ep), 'http://localhost:8081');
    });

    test('derives https on the standard TLS port for a secure endpoint', () {
      // Prod fronts everything with Caddy on 443, so no explicit port is emitted.
      const ep = CymbraEndpoint(host: 'api.example.com', port: 443, secure: true);
      expect(soundFontDeliveryOrigin(ep), 'https://api.example.com');
    });
  });
}
