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

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show PlatformException;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:purchases_flutter/purchases_flutter.dart' as rc;
import 'package:riverpod_annotation/riverpod_annotation.dart';

import 'app_platform.dart';

part 'store_client.freezed.dart';
part 'store_client.g.dart';

/// A store product as the store lists it (localized price included).
@freezed
abstract class StoreProduct with _$StoreProduct {
  const factory StoreProduct({
    required String id,
    required String title,
    required String description,

    /// Localized, ready to display (e.g. "4,99 €").
    required String price,
  }) = _StoreProduct;
}

/// What the store side hands back once a purchase / restore is settled at the
/// aggregator: the product, and whether it was a restore. The app carries no
/// receipt (change: swap-store-billing-to-revenuecat, D2): the aggregator has
/// verified it with the store; the server reconciles from the aggregator on
/// `SyncStorePlan`. [payload] stays for symmetry and is empty.
@freezed
abstract class StoreReceipt with _$StoreReceipt {
  const factory StoreReceipt({
    required String productId,
    @Default('') String payload,

    /// True for a restored (not new) transaction.
    @Default(false) bool restored,
  }) = _StoreReceipt;
}

/// A store event surfaced to the purchase flow.
@freezed
sealed class StoreEvent with _$StoreEvent {
  /// The store side is done (purchase or restore): sync the plan.
  const factory StoreEvent.receipt(StoreReceipt receipt) = StoreEventReceipt;

  /// The user cancelled.
  const factory StoreEvent.cancelled() = StoreEventCancelled;

  /// The store reported an error (message is technical — never shown raw).
  const factory StoreEvent.error(String message) = StoreEventError;

  /// The purchase is pending (parental approval, deferred payment).
  const factory StoreEvent.pending() = StoreEventPending;

  /// The receipt is bound to another Cymbra account: the aggregator's restore
  /// policy ("keep with original") refused to move it.
  const factory StoreEvent.otherAccount() = StoreEventOtherAccount;

  /// A restore found no subscription for this account.
  const factory StoreEvent.nothingToRestore() = StoreEventNothingToRestore;
}

/// Seam over the store aggregator SDK (RevenueCat: StoreKit 2 / Play Billing
/// behind it; change: swap-store-billing-to-revenuecat). Injectable so the
/// paywall and purchase flow are testable without a store, and so desktop builds
/// get a no-op client. **No attribute API is exposed** — the aggregator only
/// ever learns the Cymbra account id (D6).
abstract class StoreClient {
  /// Whether a store is reachable on this device.
  Future<bool> isAvailable();

  /// Bind the SDK to the signed-in Cymbra account (`null` = signed out). Called
  /// on every session change; a purchase before an account is set fails.
  Future<void> setAccount(String? userId);

  /// The store's listing for [ids] (localized prices). Unknown ids are absent.
  Future<List<StoreProduct>> products(Set<String> ids);

  /// Start a purchase; the outcome arrives on [events]. [accountToken] is the
  /// Cymbra account id the SDK must already be bound to.
  Future<void> buy(String productId, {required String accountToken});

  /// Restore this account's store transactions; the outcome arrives on [events]
  /// (a `receipt(restored: true)`, `nothingToRestore`, or `otherAccount`).
  Future<void> restore({required String accountToken});

  /// Tell the store the receipt was consumed server-side. The aggregator
  /// finishes transactions itself: a no-op kept for symmetry.
  Future<void> complete(StoreReceipt receipt);

  /// Purchase outcomes, restores, errors.
  Stream<StoreEvent> get events;
}

/// A store-less client (Linux / Windows / tests): nothing available, no events.
class NoopStoreClient implements StoreClient {
  const NoopStoreClient();

  @override
  Future<bool> isAvailable() async => false;

  @override
  Future<void> setAccount(String? userId) async {}

  @override
  Future<List<StoreProduct>> products(Set<String> ids) async => const [];

  @override
  Future<void> buy(String productId, {required String accountToken}) async {}

  @override
  Future<void> restore({required String accountToken}) async {}

  @override
  Future<void> complete(StoreReceipt receipt) async {}

  @override
  Stream<StoreEvent> get events => const Stream.empty();
}

/// The subset of the aggregator SDK the client uses — a seam so the client's
/// mapping (errors → events, restore → outcome) is unit-testable without the
/// plugin. [PurchasesRcSdk] is the production implementation.
abstract class RcSdk {
  Future<void> configure({required String apiKey, required String appUserId});
  Future<void> logIn(String appUserId);
  Future<void> logOut();
  Future<List<StoreProduct>> products(Set<String> ids);

  /// Throws a [PlatformException] carrying a RevenueCat error code on failure.
  Future<void> purchase(String productId);

  /// Restore; `true` when the account holds an active subscription afterwards.
  /// Throws a [PlatformException] on failure (incl. `receiptAlreadyInUse`).
  Future<bool> restore();
}

/// Production [RcSdk] over `purchases_flutter`. Never sets subscriber
/// attributes; ad-network / device-advertising collection stays at the SDK
/// default (off).
class PurchasesRcSdk implements RcSdk {
  final Map<String, rc.StoreProduct> _cache = {};

  @override
  Future<void> configure({
    required String apiKey,
    required String appUserId,
  }) async {
    if (kDebugMode) {
      await rc.Purchases.setLogLevel(rc.LogLevel.info);
    }
    await rc.Purchases.configure(
      rc.PurchasesConfiguration(apiKey)..appUserID = appUserId,
    );
  }

  @override
  Future<void> logIn(String appUserId) => rc.Purchases.logIn(appUserId);

  @override
  Future<void> logOut() => rc.Purchases.logOut();

  @override
  Future<List<StoreProduct>> products(Set<String> ids) async {
    final list = await rc.Purchases.getProducts(
      ids.toList(),
      productCategory: rc.ProductCategory.subscription,
    );
    for (final p in list) {
      _cache[p.identifier] = p;
    }
    return [
      for (final p in list)
        StoreProduct(
          id: p.identifier,
          title: p.title,
          description: p.description,
          price: p.priceString,
        ),
    ];
  }

  @override
  Future<void> purchase(String productId) async {
    var product = _cache[productId];
    if (product == null) {
      await products({productId});
      product = _cache[productId];
    }
    if (product == null) {
      throw PlatformException(
        code:
            '${rc.PurchasesErrorCode.productNotAvailableForPurchaseError.index}',
        message: 'unknown product $productId',
      );
    }
    await rc.Purchases.purchase(rc.PurchaseParams.storeProduct(product));
  }

  @override
  Future<bool> restore() async {
    final info = await rc.Purchases.restorePurchases();
    return info.activeSubscriptions.isNotEmpty ||
        info.entitlements.active.isNotEmpty;
  }
}

/// Production [StoreClient] over the aggregator SDK. Identity: the SDK is
/// configured with the Cymbra account id at the first sign-in and re-bound
/// (`logIn` / `logOut`) on every session change; it never sees an anonymous
/// purchase (the paywall requires sign-in).
class RevenueCatStoreClient implements StoreClient {
  RevenueCatStoreClient({required String apiKey, RcSdk? sdk})
    : _apiKey = apiKey,
      _sdk = sdk ?? PurchasesRcSdk();

  final String _apiKey;
  final RcSdk _sdk;
  final _events = StreamController<StoreEvent>.broadcast();
  bool _configured = false;
  String? _account;

  @override
  Future<bool> isAvailable() async => _apiKey.isNotEmpty;

  @override
  Future<void> setAccount(String? userId) async {
    if (userId == _account) return;
    try {
      if (userId == null) {
        if (_configured) await _sdk.logOut();
      } else if (!_configured) {
        await _sdk.configure(apiKey: _apiKey, appUserId: userId);
        _configured = true;
      } else {
        await _sdk.logIn(userId);
      }
      _account = userId;
    } catch (e) {
      debugPrint('store identity update failed: $e');
    }
  }

  @override
  Future<List<StoreProduct>> products(Set<String> ids) async {
    if (!_configured) return const [];
    return _sdk.products(ids);
  }

  StoreEvent _eventFor(PlatformException e) {
    final code = rc.PurchasesErrorHelper.getErrorCode(e);
    return switch (code) {
      rc.PurchasesErrorCode.purchaseCancelledError =>
        const StoreEvent.cancelled(),
      rc.PurchasesErrorCode.paymentPendingError => const StoreEvent.pending(),
      rc.PurchasesErrorCode.receiptAlreadyInUseError =>
        const StoreEvent.otherAccount(),
      _ => StoreEvent.error('${code.name}: ${e.message ?? ''}'),
    };
  }

  bool _bound(String accountToken) {
    if (_configured && _account == accountToken) return true;
    _events.add(const StoreEvent.error('store not bound to the account'));
    return false;
  }

  @override
  Future<void> buy(String productId, {required String accountToken}) async {
    if (!_bound(accountToken)) return;
    try {
      await _sdk.purchase(productId);
      _events.add(StoreEvent.receipt(StoreReceipt(productId: productId)));
    } on PlatformException catch (e) {
      _events.add(_eventFor(e));
    } catch (e) {
      _events.add(StoreEvent.error(e.toString()));
    }
  }

  @override
  Future<void> restore({required String accountToken}) async {
    if (!_bound(accountToken)) return;
    try {
      final any = await _sdk.restore();
      _events.add(
        any
            ? const StoreEvent.receipt(
                StoreReceipt(productId: '', restored: true),
              )
            : const StoreEvent.nothingToRestore(),
      );
    } on PlatformException catch (e) {
      _events.add(_eventFor(e));
    } catch (e) {
      _events.add(StoreEvent.error(e.toString()));
    }
  }

  @override
  Future<void> complete(StoreReceipt receipt) async {}

  @override
  Stream<StoreEvent> get events => _events.stream;

  /// Release the event stream (tests / hot restart).
  Future<void> dispose() => _events.close();
}

/// Public SDK keys per store (not secrets), supplied at build time:
/// `--dart-define=CYMBRA_RC_APPLE_KEY=… --dart-define=CYMBRA_RC_GOOGLE_KEY=…`.
/// Empty ⇒ the store client is a no-op (CI, dev builds without a store).
const String kRcAppleKey = String.fromEnvironment('CYMBRA_RC_APPLE_KEY');
const String kRcGoogleKey = String.fromEnvironment('CYMBRA_RC_GOOGLE_KEY');

/// The public SDK key for [platform], or empty when this build has none.
String rcApiKeyFor(
  AppPlatform platform, {
  String apple = kRcAppleKey,
  String google = kRcGoogleKey,
}) => switch (platform) {
  AppPlatform.ios || AppPlatform.macos => apple,
  AppPlatform.android => google,
  AppPlatform.linux || AppPlatform.windows || AppPlatform.web => '',
};

/// Production store-client provider: the aggregator SDK on store builds with a
/// key, a no-op client elsewhere. Override in tests with a mock.
@Riverpod(keepAlive: true)
StoreClient storeClient(Ref ref) {
  final platform = ref.watch(appPlatformProvider);
  final key = rcApiKeyFor(platform);
  if (!platform.isStoreBuild || key.isEmpty) return const NoopStoreClient();
  final client = RevenueCatStoreClient(apiKey: key);
  ref.onDispose(() {
    unawaited(client.dispose());
  });
  return client;
}

/// Debug helper: whether the store SDK is even worth calling here.
@visibleForTesting
bool storeSupported(AppPlatform p) => p.isStoreBuild;
