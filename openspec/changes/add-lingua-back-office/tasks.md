# Tasks — add-lingua-back-office

_Prérequis : `add-lingua-backend` implémenté (crate `backend/lingua`, schéma `lingua`, pool `lingua_svc`, sync + `StatsService`)._

## 1. Protos admin (backend/lingua/proto)

- [ ] 1.1 Créer `backend/lingua/proto/lingua_admin.proto` : service `LinguaAdminService` avec `AdminGetLinguaUsage` (fenêtre → tuiles : comptes actifs, mots appris, révisions, répartition par langue étudiée), `AdminGetLinguaUsageSeries` (fenêtre + dimension → points par jour), `AdminListDataPacks` (→ registre des packs)
- [ ] 1.2 Invariant vie privée au niveau du schéma : **aucun champ d'identifiant de compte** dans les messages de réponse — revue du `.proto` sur ce critère + commentaire en tête de fichier qui l'énonce
- [ ] 1.3 Vérifier que le workflow `proto` (gate `buf breaking`) couvre bien `backend/lingua/proto/**` via son glob `backend/*/proto/**` ; fichier nouveau ⇒ le gate passe

## 2. Backend — autorisation et agrégats

- [ ] 2.1 Scope `lingua` : vérifier qu'`add-lingua-backend` a déclaré `lingua` dans `SCOPES`/`APP_SCOPES` (`backend/platform/src/lib.rs`) et dans l'attribution de rôles (`backend/user/src/module.rs`) ; sinon, l'ajouter ici — avec le test « un `music/admin` n'a pas `lingua` »
- [ ] 2.2 Gating : chaque RPC de `LinguaAdminService` appelle `cymbra_platform::guard::require_admin_in_scope(&id, "lingua")` ; service monté dans `backend/server` derrière l'intercepteur **strict** (pattern `UsageServiceServer`), inerte sans `CYMBRA_LINGUA_DATABASE_URL`
- [ ] 2.3 Tests de gating (pattern `backend/analytics/src/grpc.rs`) : `lingua/admin` accepté, `global/admin` accepté (break-glass), `music/admin` refusé (`PermissionDenied`), token plat legacy refusé
- [ ] 2.4 Agrégats SQL sur le schéma `lingua` (pool `lingua_svc`) : comptes actifs (sync dans la fenêtre), mots appris/jour (transitions vers `known` datées), révisions/jour (journal syncé), répartition par langue étudiée — requêtes groupées par jour, uniquement des comptages
- [ ] 2.5 Découpage host-testable : la mise en forme des agrégats (fenêtres, séries, pourcentages) en module pur testé ; l'adaptateur Postgres mince suit la convention d'exclusion coverage (`pg*.rs`)

## 3. Registre des packs

- [ ] 3.1 `scripts/lingua-data` : émission d'un `packs-manifest.json` **committé** (pack_version, analyzer_version, paire L2→L1, date de build CI, taille, NOTICE) ; build reproductible ⇒ diff vide si le pack n'a pas changé
- [ ] 3.2 Backend : embarquer le manifeste (`include_str!`), le parser au démarrage (échec de build/boot si invalide), le servir tel quel via `AdminListDataPacks` ; test sur manifeste de fixture
- [ ] 3.3 CI : vérification que le manifeste committé correspond au pack buildé (le build de packs échoue si le manifeste est périmé)

## 4. Flags Lingua (aucune UI nouvelle)

- [ ] 4.1 Déclarer les clés Lingua dans le registry backend (`KeyDef`, app `lingua`, defaults sûrs, `doc`) — au minimum le kill-switch de sync ; vérifier leur apparition dans la console `/flags` existante sans autre modification
- [ ] 4.2 i18n BO : descriptions de flags dans `flag-descriptions.ts` (en/fr alignées)

## 5. Back-office — store, écran, navigation

- [ ] 5.1 `lib/transport.ts` : client `LinguaAdminService` ajouté à `createClients` (typé dans `Clients`), donc accessible via `api()` seulement
- [ ] 5.2 `stores/lingua.ts` (Pinia) : ressources `Async<T>` (`report`, `series`, `packs`), filtres de fenêtre (30 jours par défaut, pattern `stores/usage.ts`), `load`/`loadPacks` via `run` — aucune ref `loading`/`error` éparse, erreurs via `humanError` dans l'union
- [ ] 5.3 `views/LinguaView.vue` : tuiles + séries temporelles + répartition par langue (réutiliser les composants graphes/tableaux de `UsageView`), tableau des packs en lecture (NOTICE dépliable), mention « comptes synchronisés » (biais sync-only) ; `match(...).exhaustive()` sur chaque ressource ; aucun appel API dans le composant
- [ ] 5.4 Route `/lingua` avec `meta: { admin: true, adminScope: "lingua" }` (précédent `/takedowns`) ; type des scopes du store `auth` étendu à `lingua` ; lien de nav visible uniquement pour les admins du scope
- [ ] 5.5 i18n : libellés en/fr alignés ; lint/greps « aucune chaîne UI ne contient “lemme” » (dire « mots appris », « mots différents »)

## 6. Tests front

- [ ] 6.1 vitest sur `stores/lingua.ts` via `setClientsForTest` : succès (mapping tuiles/séries/packs), échec RPC ⇒ `error` localisée dans l'union, filtres appliqués aux requêtes
- [ ] 6.2 Seam e2e (`lib/e2e-seam.ts` + `e2e/fixtures.ts`) : fakes `LinguaAdminService` + données de seed ; ajouter si nécessaire un `loginAs` à rôles scopés (admin lingua vs admin music-only)
- [ ] 6.3 `e2e/lingua.spec.ts` (pattern `usage.spec.ts`) : un admin lingua voit tuiles + séries + packs ; un modérateur ne voit pas le lien et est redirigé ; un admin music-only est redirigé (gate `adminScope`)

## 7. Gates et finitions

- [ ] 7.1 `cargo fmt --all --check` + `clippy -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` (adaptateurs minces sous la convention d'exclusion existante — pas de nouvelle entrée sauf besoin avéré)
- [ ] 7.2 `yarn` back-office : vitest vert, Playwright vert, `back-office-check` passe ; `ci-units` inchangé (aucune nouvelle unité) — le vérifier avec `python3 scripts/check_ci_units.py --list`
- [ ] 7.3 Revue vie privée finale : relire chaque message de réponse des protos admin (aucun identifiant de compte), chaque requête SQL (uniquement des agrégats), l'écran (aucune recherche par compte)
- [ ] 7.4 `openspec validate add-lingua-back-office --strict` final + mise à jour des specs si l'implémentation a fait bouger un contrat
