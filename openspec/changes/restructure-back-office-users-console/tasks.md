## 1. Routes, navigation et redirections

- [x] 1.1 Ajouter les routes `/users` (`users`), `/users/:userId` (`user-detail`, `props: true`) et `/campaigns` (`campaigns`) dans `src/router.ts`, toutes en `meta: { admin: true }`
- [x] 1.2 Rediriger `/roles` → `/users` et `/plans` → `/campaigns` (favoris et liens existants)
- [x] 1.3 Remplacer les entrées de nav `nav.roles`/`nav.plans` par `nav.users`/`nav.campaigns` dans `src/App.vue` (icônes conservées)

## 2. Stores

- [x] 2.1 `stores/roles.ts` : ajouter `account: Async<AccountRow | null>` et `loadAccount(userId)` sur `ListAccounts({ ids: [userId], limit: 1 })`, `null` quand la page est vide
- [x] 2.2 `stores/roles.ts` : permettre à `grant`/`revoke` de rafraîchir **un compte** au lieu de la page d'annuaire (rafraîchissement passé par l'appelant)
- [x] 2.3 `stores/roles.ts` : exposer une remise à `idle` de `grants` et `reliability` (changement de compte)
- [x] 2.4 `stores/plans.ts` : ajouter `lookupUser(userId)` (adressage explicite par id) et remplacer `lastLookup: string` par `lastTarget: AccountRef` — la page détail connaît l'id, elle ne doit pas repasser par l'heuristique texte d'`accountRef`

## 3. Panneaux extraits

- [x] 3.1 Extraire le bloc (a) de `PlansView.vue` dans `components/AccountPlanPanel.vue` (plan effectif, lignes de droit, adhésions, lien RevenueCat) avec ses dialogues grant/enrol/raison/confirmation, focus et Escape inclus
- [x] 3.2 Extraire les bascules de rôle de `RolesView.vue` dans `components/AccountRolesPanel.vue`, avec un bloc par scope autorisé
- [x] 3.3 Extraire la table d'audit dans `components/AccountHistoryPanel.vue`
- [x] 3.4 Rendre `CuratorReliabilityDrawer.vue` utilisable en panneau sur la page détail (ou l'y ouvrir tel quel)

## 4. Écran Utilisateurs (`/users`)

- [x] 4.1 Renommer `RolesView.vue` en `UsersView.vue` et retirer les actions par ligne (rôles, historique, fiabilité, sessions) ainsi que les panneaux dépliants
- [x] 4.2 Rendre la cellule handle `RouterLink` vers `/users/{id}` et la ligne cliquable en confort (clavier compris)
- [x] 4.3 Conserver recherche, sélecteur de scope, filtres plan/bêta, pagination et états vides localisés

## 5. Page détail (`/users/{user_id}`)

- [x] 5.1 Créer `UserDetailView.vue` : en-tête d'identité, chargement par `userId` au montage et au changement de param, purge des états partagés avant chaque chargement
- [x] 5.2 Monter les panneaux rôles / historique / fiabilité / révocation des sessions (avec confirmation) et le retour vers l'annuaire
- [x] 5.3 Monter `AccountPlanPanel` derrière le garde `showPlans` (admin scope `music`) — aucun RPC plan émis sinon
- [x] 5.4 États `Async` : squelette en `loading`, « compte introuvable » localisé quand l'id ne correspond à rien, erreurs en toast localisé (jamais de string gRPC)

## 6. Écran Campagnes (`/campaigns`)

- [x] 6.1 Renommer `PlansView.vue` en `CampaignsView.vue` et supprimer le bloc de recherche de compte (et le `handleFor` devenu inutile hors membres)
- [x] 6.2 Lier chaque membre d'une campagne vers `/users/{user_id}` (handle ou `IdBadge`)
- [x] 6.3 Vérifier que campagnes, codes, membres et export CSV sont inchangés

## 7. i18n

- [x] 7.1 Renommer l'espace `roles.*` en `users.*` et ajouter les libellés de la page détail (`en` et `fr` dans la même passe)
- [x] 7.2 Basculer les libellés d'écran de `plans.*` vers « Campagnes » et ajouter `nav.users` / `nav.campaigns`
- [x] 7.3 Vérifier l'absence de dérive entre `en` et `fr` (`test/i18n.spec.ts`)

## 8. Tests

- [x] 8.1 Étendre `test/roles.spec.ts` (chargement d'un compte par id, compte inconnu, purge entre comptes, rafraîchissement ciblé) — le fichier garde son nom : il teste le **store** `roles`, qui n'est pas renommé ; la liste est couverte en e2e
- [x] 8.2 Nouveau `test/user-detail.spec.ts` : chargement direct par id, compte introuvable, purge entre deux comptes, absence totale du bloc plan pour un admin non-`music`, grant/revoke de rôle et révocation des sessions
- [x] 8.3 Adapter `test/plans.spec.ts` aux campagnes seules et au déplacement du bloc compte
- [x] 8.4 Réécrire `e2e/roles.spec.ts` et `e2e/plans.spec.ts` en `e2e/users.spec.ts`, `e2e/user-detail.spec.ts` et `e2e/campaigns.spec.ts` sur le seam de faux client existant, en couvrant chaque geste déplacé
- [x] 8.5 Couvrir les redirections `/roles` → `/users` et `/plans` → `/campaigns`

## 9. Vérifications finales

- [x] 9.1 `yarn lint`, `yarn typecheck`, `yarn format:check` (aucun fichier Dart/Rust touché : `melos run analyze` et `cargo fmt` sans objet)
- [x] 9.2 `yarn test` (couverture ≥ 80 %) et `yarn e2e` verts
- [ ] 9.3 Vérification manuelle sur `bo.cymbra.app` (ou le dev server) : parcours annuaire → détail → retour, deep-link `/users/{id}`, `/campaigns` sans recherche de compte, anciennes URL redirigées
