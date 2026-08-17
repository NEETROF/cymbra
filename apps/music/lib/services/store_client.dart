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
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:in_app_purchase/in_app_purchase.dart';
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

/// What a store hands back for a purchase or a restore: the product and the
/// opaque payload the server verifies (Apple: signed transaction JWS; Google:
/// purchase token), plus the handle to complete/acknowledge the store side.
@freezed
abstract class StoreReceipt with _$StoreReceipt {
  const factory StoreReceipt({
    required String productId,
    required String payload,

    /// True for a restored (not new) transaction.
    @Default(false) bool restored,
  }) = _StoreReceipt;
}

/// A store event surfaced to the purchase flow.
@freezed
sealed class StoreEvent with _$StoreEvent {
  /// A verified-by-the-store receipt to report to the server.
  const factory StoreEvent.receipt(StoreReceipt receipt) = StoreEventReceipt;

  /// The user cancelled.
  const factory StoreEvent.cancelled() = StoreEventCancelled;

  /// The store reported an error (message is technical — never shown raw).
  const factory StoreEvent.error(String message) = StoreEventError;

  /// The purchase is pending (parental approval, deferred payment).
  const factory StoreEvent.pending() = StoreEventPending;
}

/// Seam over the platform store SDK (StoreKit 2 / Play Billing through the
/// official `in_app_purchase` plugin; change: add-premium-subscription).
/// Injectable so the paywall and purchase flow are testable without a store,
/// and so desktop builds get a no-op client.
abstract class StoreClient {
  /// Whether a store is reachable on this device.
  Future<bool> isAvailable();

  /// The store's listing for [ids] (localized prices). Unknown ids are absent.
  Future<List<StoreProduct>> products(Set<String> ids);

  /// Start a purchase; the outcome arrives on [events]. [accountToken] is the
  /// Cymbra account id — the store binds the transaction to it (Apple
  /// `appAccountToken`, Google `obfuscatedAccountId`), so the server can refuse
  /// a receipt reported by another account.
  Future<void> buy(String productId, {required String accountToken});

  /// Re-deliver the account's transactions on [events] as restored receipts.
  Future<void> restore({required String accountToken});

  /// Tell the store the receipt was consumed server-side (finishes / acknowledges).
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

/// Production [StoreClient] over `in_app_purchase` (StoreKit 2 is the plugin's
/// default on Apple platforms; Play Billing on Android). Receipts carry
/// `serverVerificationData` — the JWS / purchase token the backend verifies.
class InAppPurchaseStoreClient implements StoreClient {
  InAppPurchaseStoreClient({InAppPurchase? plugin})
    : _iap = plugin ?? InAppPurchase.instance {
    _sub = _iap.purchaseStream.listen(
      _onPurchases,
      onError: (Object e) {
        _events.add(StoreEvent.error(e.toString()));
      },
    );
  }

  final InAppPurchase _iap;
  final _events = StreamController<StoreEvent>.broadcast();
  final Map<String, ProductDetails> _details = {};
  final Map<String, PurchaseDetails> _pending = {};
  StreamSubscription<List<PurchaseDetails>>? _sub;

  void _onPurchases(List<PurchaseDetails> purchases) {
    for (final p in purchases) {
      switch (p.status) {
        case PurchaseStatus.pending:
          _events.add(const StoreEvent.pending());
        case PurchaseStatus.canceled:
          _events.add(const StoreEvent.cancelled());
        case PurchaseStatus.error:
          _events.add(StoreEvent.error(p.error?.message ?? 'store error'));
        case PurchaseStatus.purchased:
        case PurchaseStatus.restored:
          final payload = p.verificationData.serverVerificationData;
          if (payload.isEmpty) {
            _events.add(const StoreEvent.error('empty receipt'));
            continue;
          }
          _pending['${p.productID}:$payload'] = p;
          _events.add(
            StoreEvent.receipt(
              StoreReceipt(
                productId: p.productID,
                payload: payload,
                restored: p.status == PurchaseStatus.restored,
              ),
            ),
          );
      }
    }
  }

  @override
  Future<bool> isAvailable() => _iap.isAvailable();

  @override
  Future<List<StoreProduct>> products(Set<String> ids) async {
    final resp = await _iap.queryProductDetails(ids);
    for (final d in resp.productDetails) {
      _details[d.id] = d;
    }
    return [
      for (final d in resp.productDetails)
        StoreProduct(
          id: d.id,
          title: d.title,
          description: d.description,
          price: d.price,
        ),
    ];
  }

  @override
  Future<void> buy(String productId, {required String accountToken}) async {
    var details = _details[productId];
    if (details == null) {
      await products({productId});
      details = _details[productId];
    }
    if (details == null) {
      _events.add(const StoreEvent.error('unknown product'));
      return;
    }
    // Subscriptions go through the non-consumable path on both stores.
    await _iap.buyNonConsumable(
      purchaseParam: PurchaseParam(
        productDetails: details,
        applicationUserName: accountToken,
      ),
    );
  }

  @override
  Future<void> restore({required String accountToken}) =>
      _iap.restorePurchases(applicationUserName: accountToken);

  @override
  Future<void> complete(StoreReceipt receipt) async {
    final p = _pending.remove('${receipt.productId}:${receipt.payload}');
    if (p != null && p.pendingCompletePurchase) {
      await _iap.completePurchase(p);
    }
  }

  @override
  Stream<StoreEvent> get events => _events.stream;

  /// Release the store subscription (tests / hot restart).
  Future<void> dispose() async {
    await _sub?.cancel();
    await _events.close();
  }
}

/// Production store-client provider: the plugin on store builds, a no-op client
/// elsewhere. Override in tests with a mock.
@Riverpod(keepAlive: true)
StoreClient storeClient(Ref ref) {
  final platform = ref.watch(appPlatformProvider);
  if (!platform.isStoreBuild) return const NoopStoreClient();
  final client = InAppPurchaseStoreClient();
  ref.onDispose(() {
    unawaited(client.dispose());
  });
  return client;
}

/// Debug helper: whether the store SDK is even worth calling here.
@visibleForTesting
bool storeSupported(AppPlatform p) => p.isStoreBuild;
