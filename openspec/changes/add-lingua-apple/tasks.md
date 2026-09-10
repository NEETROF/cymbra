# Tasks — add-lingua-apple

## 1. Apple : app conteneur + extension Safari

- [ ] 1.1 Scaffold `apps/lingua-apple` : `safari-web-extension-converter` sur la variante `safari` (nouvelle variante du build multi-cibles), projet Xcode **universal purchase** (une fiche iOS + macOS, bundle `com.cymbra.lingua`)
- [ ] 1.2 Handler natif : `SafariWebExtensionHandler` → FFI `lingua-core` (lib statique), packs dans le bundle de l'app ; impl `AnalyzerPort` nativeMessaging (event page, lots, mémoïsation) ; dégradation douce testée (pont coupé ⇒ pas de surlignage, page intacte)
- [ ] 1.3 App conteneur (SwiftUI minimal, charte Cymbra) : decks + révision FSRS + calibration + import LingQ + réglages — l'app vit sans l'extension
- [ ] 1.4 Activation guidée : walkthrough iOS pas-à-pas + heartbeat App Group ; macOS deep link `showPreferencesForExtension` + état `SFSafariExtensionManager` ; accueil reflétant l'état
- [ ] 1.5 Signing/TestFlight : cloner le pattern `release-build` de music (App ID, profils, lane CI) ; dogfooding via TestFlight interne
- [ ] 1.6 Parcours manuel Safari macOS (dev-mode puis signé) et iOS (TestFlight) : activation → calibration → lecture → +Deck → révision ; soumission App Store
- [ ] 1.7 `ci-units` : ajouter `apps/lingua-apple` (et la lane qui le surveille) au filtre
