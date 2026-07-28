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
import 'package:flutter_test/flutter_test.dart';
import 'package:grpc/grpc.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/profile_service.dart';
import 'package:music/state/profile_notifier.dart';

@GenerateNiceMocks([MockSpec<ProfileService>()])
import 'profile_notifier_test.mocks.dart';

ProviderContainer _container(ProfileService service) {
  final c = ProviderContainer(
    overrides: [profileServiceProvider.overrideWithValue(service)],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('setting public with a DOB surfaces the new visibility', () async {
    final service = MockProfileService();
    when(
      service.setProfileVisibility(
        'public',
        dateOfBirth: anyNamed('dateOfBirth'),
      ),
    ).thenAnswer((_) async => 'public');
    final c = _container(service);

    await c
        .read(profileVisibilityControllerProvider.notifier)
        .setVisibility('public', dateOfBirth: DateTime(2000, 1, 1));

    expect(c.read(profileVisibilityControllerProvider).value, 'public');
  });

  test(
    'an under-age refusal is surfaced as an error state, not thrown',
    () async {
      final service = MockProfileService();
      when(
        service.setProfileVisibility(
          'public',
          dateOfBirth: anyNamed('dateOfBirth'),
        ),
      ).thenThrow(GrpcError.failedPrecondition('too young'));
      final c = _container(service);

      await c
          .read(profileVisibilityControllerProvider.notifier)
          .setVisibility('public', dateOfBirth: DateTime(2015, 1, 1));

      final state = c.read(profileVisibilityControllerProvider);
      expect(state.hasError, isTrue);
      expect((state.error as GrpcError).code, StatusCode.failedPrecondition);
    },
  );

  test('setting private needs no DOB', () async {
    final service = MockProfileService();
    when(
      service.setProfileVisibility('private', dateOfBirth: null),
    ).thenAnswer((_) async => 'private');
    final c = _container(service);

    await c
        .read(profileVisibilityControllerProvider.notifier)
        .setVisibility('private');

    expect(c.read(profileVisibilityControllerProvider).value, 'private');
  });
}
