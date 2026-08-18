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

import 'dart:io';

import 'package:flutter/services.dart' show PlatformException;
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/app_platform.dart';
import 'package:music/services/store_client.dart';
import 'package:purchases_flutter/purchases_flutter.dart'
    show PurchasesErrorCode;

import 'store_client_test.mocks.dart';

@GenerateNiceMocks([MockSpec<RcSdk>()])
PlatformException _rcError(PurchasesErrorCode code) =>
    PlatformException(code: '${code.index}', message: code.name);

void main() {
  group('RevenueCatStoreClient', () {
    late MockRcSdk sdk;
    late RevenueCatStoreClient client;

    setUp(() {
      sdk = MockRcSdk();
      when(
        sdk.configure(
          apiKey: anyNamed('apiKey'),
          appUserId: anyNamed('appUserId'),
        ),
      ).thenAnswer((_) async {});
      when(sdk.logIn(any)).thenAnswer((_) async {});
      when(sdk.logOut()).thenAnswer((_) async {});
      client = RevenueCatStoreClient(apiKey: 'appl_test', sdk: sdk);
    });

    tearDown(() => client.dispose());

    test('identity: configure at first sign-in, logIn on switch, logOut on '
        'sign-out; nothing else ever reaches the SDK', () async {
      expect(await client.isAvailable(), isTrue);
      await client.setAccount('u1');
      verify(sdk.configure(apiKey: 'appl_test', appUserId: 'u1')).called(1);
      await client.setAccount('u1'); // no-op
      await client.setAccount('u2');
      verify(sdk.logIn('u2')).called(1);
      await client.setAccount(null);
      verify(sdk.logOut()).called(1);
      // Sign-out before any configure is a no-op (nothing to log out of).
      final fresh = RevenueCatStoreClient(apiKey: 'k', sdk: sdk);
      await fresh.setAccount(null);
      verifyNever(sdk.configure(apiKey: 'k', appUserId: anyNamed('appUserId')));
      await fresh.dispose();
    });

    test('products come from the SDK only once bound', () async {
      when(sdk.products(any)).thenAnswer(
        (_) async => const [
          StoreProduct(
            id: 'premium_monthly',
            title: 'Premium',
            description: '',
            price: '4,99 €',
          ),
        ],
      );
      expect(await client.products({'premium_monthly'}), isEmpty);
      await client.setAccount('u1');
      final list = await client.products({'premium_monthly'});
      expect(list.single.price, '4,99 €');
    });

    test('buy: success ⇒ receipt without payload; SDK errors map to events; '
        'unbound account ⇒ error event', () async {
      final events = <StoreEvent>[];
      final sub = client.events.listen(events.add);
      // Not bound yet.
      await client.buy('premium_monthly', accountToken: 'u1');
      await client.setAccount('u1');
      // Bound to another account than the one asked for.
      await client.buy('premium_monthly', accountToken: 'u2');
      when(sdk.purchase('premium_monthly')).thenAnswer((_) async {});
      await client.buy('premium_monthly', accountToken: 'u1');
      when(
        sdk.purchase('premium_monthly'),
      ).thenThrow(_rcError(PurchasesErrorCode.purchaseCancelledError));
      await client.buy('premium_monthly', accountToken: 'u1');
      when(
        sdk.purchase('premium_monthly'),
      ).thenThrow(_rcError(PurchasesErrorCode.paymentPendingError));
      await client.buy('premium_monthly', accountToken: 'u1');
      when(
        sdk.purchase('premium_monthly'),
      ).thenThrow(_rcError(PurchasesErrorCode.receiptAlreadyInUseError));
      await client.buy('premium_monthly', accountToken: 'u1');
      when(
        sdk.purchase('premium_monthly'),
      ).thenThrow(_rcError(PurchasesErrorCode.storeProblemError));
      await client.buy('premium_monthly', accountToken: 'u1');
      when(sdk.purchase('premium_monthly')).thenThrow(StateError('boom'));
      await client.buy('premium_monthly', accountToken: 'u1');
      await Future<void>.delayed(Duration.zero);
      await sub.cancel();
      expect(events, hasLength(8));
      expect(events[0], isA<StoreEventError>());
      expect(events[1], isA<StoreEventError>());
      expect(
        events[2],
        const StoreEvent.receipt(StoreReceipt(productId: 'premium_monthly')),
      );
      expect(events[3], const StoreEvent.cancelled());
      expect(events[4], const StoreEvent.pending());
      expect(events[5], const StoreEvent.otherAccount());
      expect(events[6], isA<StoreEventError>());
      expect(events[7], isA<StoreEventError>());
    });

    test('restore: found ⇒ restored receipt, none ⇒ nothingToRestore, '
        'other account ⇒ otherAccount', () async {
      final events = <StoreEvent>[];
      final sub = client.events.listen(events.add);
      await client.setAccount('u1');
      when(sdk.restore()).thenAnswer((_) async => true);
      await client.restore(accountToken: 'u1');
      when(sdk.restore()).thenAnswer((_) async => false);
      await client.restore(accountToken: 'u1');
      when(
        sdk.restore(),
      ).thenThrow(_rcError(PurchasesErrorCode.receiptAlreadyInUseError));
      await client.restore(accountToken: 'u1');
      await Future<void>.delayed(Duration.zero);
      await sub.cancel();
      expect(events, [
        const StoreEvent.receipt(StoreReceipt(productId: '', restored: true)),
        const StoreEvent.nothingToRestore(),
        const StoreEvent.otherAccount(),
      ]);
      // complete() is a no-op: the aggregator finishes transactions itself.
      await client.complete(const StoreReceipt(productId: 'x'));
    });
  });

  group('store client provider', () {
    test('key selection per platform; empty ⇒ no-op client', () {
      expect(rcApiKeyFor(AppPlatform.ios, apple: 'a', google: 'g'), 'a');
      expect(rcApiKeyFor(AppPlatform.macos, apple: 'a', google: 'g'), 'a');
      expect(rcApiKeyFor(AppPlatform.android, apple: 'a', google: 'g'), 'g');
      for (final p in [
        AppPlatform.linux,
        AppPlatform.windows,
        AppPlatform.web,
      ]) {
        expect(rcApiKeyFor(p, apple: 'a', google: 'g'), isEmpty);
      }
      expect(storeSupported(AppPlatform.linux), isFalse);
      const noop = NoopStoreClient();
      expect(noop.events, emitsDone);
    });
  });

  test('the SDK stays behind the seam: only store_client.dart imports it', () {
    final offenders = <String>[];
    for (final f in Directory('lib').listSync(recursive: true)) {
      if (f is! File || !f.path.endsWith('.dart')) continue;
      if (f.path.endsWith('services/store_client.dart')) continue;
      if (f.readAsStringSync().contains('package:purchases_flutter/')) {
        offenders.add(f.path);
      }
    }
    expect(
      offenders,
      isEmpty,
      reason: 'purchases_flutter leaked past the seam',
    );
  });
}
