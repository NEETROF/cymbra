# Design — add-lingua-back-office

## Context

La pile locale (`add-lingua-analysis` → `add-lingua-agent`) livre un produit local-only ; `add-lingua-backend` ajoute le backend (`backend/lingua` : schéma, sync, `StatsService`). Ce change ferme la boucle d'exploitation : ce que l'administrateur voit de Lingua, et surtout ce qu'il ne voit **pas**. Décision utilisateur actée en amont : périmètre OPS uniquement — packs + flags + agrégats — **pas de vue support par compte**, pour la vie privée (les données de lecture d'une personne sont sensibles).

Contraintes héritées : skill `vue-frontend-architecture` (seam `api()`, `Async<T>` exhaustif, aucun appel API dans un composant, e2e fake-client, i18n en/fr alignées) ; mémoire « séparation des pouvoirs » (jamais un check de rôle plat sur un token console : le token `back-office` unionne les rôles de tous les scopes) ; convention coverage (adaptateurs minces exclus, logique host-testée) ; règle UI « jamais “lemme” à l'écran ».

## Goals / Non-Goals

**Goals :**
- Un administrateur **du scope lingua** voit l'usage agrégé (comptes actifs, mots appris/jour, révisions/jour, répartition par langue étudiée), le registre des versions de packs, et gère les flags Lingua depuis la console `/flags` existante.
- La vie privée est garantie **par le schéma** : aucun RPC admin Lingua ne peut transporter une donnée par compte.

**Non-Goals :**
- Vue support par compte (rejetée, pas différée), recherche d'un utilisateur, drill-down individuel.
- OTA des packs (le registre est en lecture, la distribution reste embarquée dans l'extension).
- Nouvelle UI de flags, nouvelle télémétrie côté clients Lingua, agrégats temps réel (quotidien suffit).

## Decisions

### D1 — Capability `admin-*`, une seule : `admin-lingua-console`
L'écran appartient au produit **admin** (précédents : `admin-plan-console`, `admin-account-directory`), même si les RPC vivent dans `backend/lingua` : le contrat décrit ce que la console permet et interdit. Alternative rejetée : une capability `lingua-admin` côté produit Lingua — le domaine `lingua-*` décrit l'expérience apprenant ; mélanger l'OPS dedans brouillerait la frontière que `config.yaml` établit.

### D2 — Périmètre OPS only : la vie privée est une exigence, pas une absence
L'admin ne voit **que des agrégats**. Le moyen le plus fort de le garantir est structurel : les messages de réponse des protos admin Lingua **ne comportent aucun champ d'identifiant de compte**, et le backend ne construit que des `COUNT`/`SUM` groupés par jour/langue. Une fuite exigerait de changer le `.proto` — visible en review et au gate `buf breaking`. Alternative rejetée : une vue support par compte gatée plus fort — même gatée, elle crée l'endpoint qu'on regrettera ; décision utilisateur explicite.

### D3 — Autorisation : `require_admin_in_scope(id, "lingua")`, jamais le set plat
Le token console unionne les rôles de tous les scopes (leçon « séparation des pouvoirs » : 25 sites sur 39 testaient le set plat — un admin d'un produit était admin de tout). Chaque RPC admin Lingua exige `admin` **dans le scope `lingua`** (`backend/platform/src/guard.rs`), break-glass `global/admin` inclus — le pattern déjà appliqué par `backend/analytics` (scope `music`) et les RPC admin de `backend/music`. Côté front, la route porte `meta: { admin: true, adminScope: "lingua" }` (précédent : `/takedowns`) et le lien de nav est masqué hors scope — UX seulement, chaque RPC est re-gaté serveur. Prérequis : le scope `lingua` déclaré dans `SCOPES`/`APP_SCOPES` (et attribuable via la console rôles) — normalement fait par `add-lingua-backend` ; sinon ce change le fait (vérification en tâche 2.1).

### D4 — Agrégats calculés sur les données de sync, aucune télémétrie nouvelle
`AdminGetLinguaUsage` (tuiles) et `AdminGetLinguaUsageSeries` (points par jour) agrègent en SQL le schéma `lingua` (pool `lingua_svc` — le privilège vient du pool, pas du fichier) : comptes actifs = comptes ayant synchronisé dans la fenêtre ; mots appris = transitions vers `known` datées ; révisions = journal de révision syncé ; répartition = langues étudiées des profils actifs. Biais assumé et **affiché** : un appareil qui ne synchronise pas est invisible (mention à l'écran « comptes synchronisés »). Alternatives rejetées : brancher `cymbra-analytics` (événements app-side, HMAC par période — conçu pour l'app Music, redondant ici : la sync possède déjà l'état) ; nouvelle télémétrie extension→backend (surface privacy nouvelle pour zéro besoin).

### D5 — Registre des packs : manifeste committé, embarqué au compile
`scripts/lingua-data` émet un `packs-manifest.json` (pack_version, analyzer_version, paire L2→L1, date de build CI, taille, NOTICE) — petit, déterministe, diffé en review comme n'importe quel changement de pack. Le backend l'embarque (`include_str!`) et `AdminListDataPacks` le sert tel quel. Alternatives rejetées : table `lingua.data_packs` alimentée par la CI — exige un chemin d'écriture CI→prod authentifié pour une donnée de release qui vit très bien dans le repo ; saisie manuelle BO — dérive garantie. Le jour de l'OTA, le manifeste migre vers DB/ObjectStorage et ce RPC change de source sans changer de forme.

### D6 — Flags : des clés de registry, zéro UI
Les flags Lingua sont des `KeyDef` app `lingua` dans le registry backend (`backend/feature-flags/src/registry.rs`) ; la console `/flags` existante les liste et les administre avec son gating actuel. Rien d'autre. (Le durcissement éventuel du gating d'écriture de `/flags` par scope est un chantier `feature-flags-admin` séparé, hors périmètre.)

### D7 — Front : le pattern `/usage`, à l'identique
Store Pinia `stores/lingua.ts` derrière `api()` (clients ajoutés dans `lib/transport.ts`, doublés dans le seam e2e), ressources en `Async<T>` (`report`, `series`, `packs`) matchées `match(...).exhaustive()`, filtres de fenêtre (30 jours par défaut), vue `LinguaView.vue` réutilisant les composants tuiles/graphes/tableaux de `UsageView`. Erreur RPC = message localisé dans l'union (jamais de code gRPC brut). i18n : `en.json`/`fr.json` alignées, aucun libellé contenant « lemme ».

## Risks / Trade-offs

- [Petites cohortes : « répartition par langue » avec n=1 désigne presque quelqu'un] → console interne gatée admin-lingua, risque accepté en v1 ; si Lingua s'ouvre à des opérateurs moins privilégiés, ajouter un plancher d'affichage (« < 5 ») — question ouverte.
- [Biais « sync-only » des agrégats] → assumé et libellé à l'écran ; c'est la contrepartie directe de « aucune télémétrie nouvelle » (D4).
- [Dépendance `add-lingua-backend` non mergée] → ce change ne compile pas sans le crate `backend/lingua` ; l'ordre d'implémentation est séquentiel, les specs peuvent être ratifiées en parallèle.
- [Manifeste committé ≠ état réel des stores] → le registre décrit ce que la CI a buildé, pas ce que chaque store a publié ; assumé (informatif) et écrit dans l'UI ; l'OTA apportera la vérité serveur.
- [Retirer un RPC admin plus tard = break `buf` FILE] → les trois RPC naissent dans un fichier proto dédié, volontairement minimal ; toute extension passe par des champs additifs.
- [Le seam e2e ne sait peut-être pas simuler un admin scopé hors-lingua] → étendre `fixtures.ts`/`e2e-seam.ts` (rôles par scope) fait partie de la tâche e2e, pas un imprévu.

## Migration Plan

Rien à migrer : tout est additif (nouveau fichier proto, nouveau service, nouvel écran). Rollback avant release = supprimer la route et ne pas monter le service. Après release, retirer un RPC = break intentionnel marqué (`feat(lingua)!:` — pattern du gate `proto`). Les agrégats ne créent aucune donnée nouvelle ; le manifeste de packs est un artefact de build versionné avec le repo.

## Open Questions

- Plancher d'affichage des petites cohortes (« < 5 ») dès la v1 ou seulement si la console s'ouvre au-delà des admins lingua ?
- Fenêtre par défaut : 30 jours (aligné `/usage`) — confirmer que la saisonnalité Lingua ne réclame pas 7 jours par défaut.
- `AdminGetLinguaUsage` et `AdminGetLinguaUsageSeries` : un RPC combiné suffirait-il ? Trancher à l'implémentation en regardant ce que l'écran appelle réellement (le découpage actuel mime `usage.proto`, qui a fait ses preuves).
