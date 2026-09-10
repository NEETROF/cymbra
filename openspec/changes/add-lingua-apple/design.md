# Design — add-lingua-apple

## Context

La pile a livré l'extension Chromium complète (`add-lingua-extension-reading` + `add-lingua-extension-review`, drawer injecté compris) et le système de variantes de build avec la variante Firefox (`add-lingua-firefox`). Ce change ajoute la troisième variante — `safari` — et l'app conteneur Apple qui l'héberge. Décisions héritées par référence : `AnalyzerPort` et cible WASM (`add-lingua-wasm`), surfaces de révision et drawer injecté (`add-lingua-extension-review`), variantes de manifest (`add-lingua-firefox`), schéma de storage versionné (`add-lingua-extension-reading`).

## Decisions

### D1 — Variante Safari dans la matrice de build (part Safari de D12 du design source)

La variante `safari` rejoint le build multi-cibles introduit par `add-lingua-firefox` : le build produit désormais chromium / firefox / safari depuis la même source. Ce qui varie reste confiné derrière les deux coutures existantes : l'**`AnalyzerPort`** — Safari = **aucun WASM** : nativeMessaging vers le handler natif de l'app conteneur, qui linke `lingua-core` compilé ARM — et la **surface de panneau** — drawer injecté seul sur Safari, qui n'a pas d'API de panneau. Les canaux tier 3 ne reçoivent ni test ni promesse.

### D2 — App conteneur Apple : une fiche, deux OS, l'analyse en natif (D13 du design source)

Un projet Xcode (`apps/lingua-apple`), **une fiche App Store universelle iOS + macOS** (universal purchase). L'app n'est pas une coquille (guideline 4.4) : elle héberge decks/révision et le parcours d'activation — indispensable car l'extension arrive **désactivée** : sur iOS, walkthrough pas-à-pas + détection par **heartbeat App Group** (aucune API d'état d'extension sur iOS) ; sur macOS, deep link `SFSafariApplication.showPreferencesForExtension` + `SFSafariExtensionManager` pour l'état réel. L'analyse passe par le `SafariWebExtensionHandler` (event page → `sendNativeMessage` → `lingua-core` natif) ; les **packs vivent dans le bundle de l'app** (contourne les quotas de storage d'extension iOS ~3 Mo). Sans sync (change ultérieur), chaque appareil a son état local, amorcé par calibration/import LingQ ; les schémas partagent les types de `lingua-core` pour une fusion mécanique. Signing/TestFlight : pattern `release-build` de music cloné (chaîne Apple existante) ; dogfooding iOS via TestFlight interne (sans review publique) ; cadence des fixes Safari = review App Store, donc les comportements se rodent d'abord sur Chromium.

## Risks / Trade-offs

- [Fragilités Safari (SW tués, quotas storage, review Apple sur chaque fix)] → event page partout chez Apple, analyse et packs côté natif, comportements rodés sur Chromium avant d'être figés côté Safari ; taxe de septembre (nouvel OS Apple) budgétée.
- [Funnel d'activation iOS (extension désactivée par défaut, sans aide système)] → walkthrough animé + heartbeat App Group ; c'est le décrochage documenté n°1 de la catégorie, traité comme un écran produit à part entière, pas un README.
- [Panne du pont nativeMessaging] → dégradation douce (pas de surlignage), page intacte — testée explicitement (tâche 1.2).
- [Quatre navigateurs pour un solo dev] → un artefact unique + coutures ; l'ordre d'implémentation reste Chromium → Firefox → Apple ; matrice de parcours manuel par navigateur avant chaque release.
