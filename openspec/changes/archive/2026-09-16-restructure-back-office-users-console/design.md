# Design — Users console

## Context

Le back-office traite un compte à deux endroits :

- [RolesView.vue](../../../apps/back-office/src/views/RolesView.vue) (`/roles`) — annuaire
  paginé (`ListAccounts`), colonnes rôles/plan/bêtas (badges via `GetPlansForAccounts`),
  et **cinq boutons par ligne** (+ modérateur, + admin, historique, fiabilité, révoquer les
  sessions) dont deux ouvrent des panneaux dépliants partagés par toute la table
  (`selected` + `panel`).
- [PlansView.vue](../../../apps/back-office/src/views/PlansView.vue) (`/plans`) — 34 ko en
  trois blocs indépendants : (a) recherche de compte → plan effectif + lignes de droit +
  adhésions + grant/enrol/RevenueCat, (b) campagnes + codes, (c) membres d'une campagne.

Contraintes :

- **Aucun changement serveur n'est nécessaire ni souhaité.** `LookupAccountPlan` accepte
  déjà `user_id` (`accountRef()` détecte l'UUID) et `ListAccounts` accepte déjà un filtre
  `ids` — c'est exactement ce qu'il faut pour une page adressable par id.
- `/roles` est ouvert à **tout admin** (`meta.admin`), les RPC plans sont
  `require_admin_in_scope("music")` : la page détail mélange donc deux niveaux
  d'autorisation et doit garder le garde-fou `showPlans` déjà présent dans la liste.
- Règles [vue-frontend-architecture](../../../.claude/skills/vue-frontend-architecture) :
  un composant n'appelle jamais l'API (seuls les stores le font, derrière `lib/api.ts`),
  et chaque ressource async est **une** union `Async<T>` matchée en `ts-pattern`.

## Goals / Non-Goals

**Goals**

- Un compte = une adresse (`/users/{user_id}`), atteignable par clic depuis l'annuaire,
  par favori et par rechargement.
- L'annuaire redevient une surface de lecture/filtre/navigation.
- `/campaigns` ne parle plus que de campagnes.
- Zéro régression fonctionnelle : chaque geste existant reste accessible, au même coût
  d'autorisation et avec le même audit.

**Non-Goals**

- Aucun nouveau RPC, aucun changement de proto, aucune migration de données.
- Aucune ouverture aux modérateurs (la page détail reste admin-only, route + serveur).
- Aucun changement de sémantique des campagnes (fermeture = pause, etc.).
- Pas de résolution serveur handle↔id supplémentaire (voir Risques).

## Decisions

### D1 — Trois vues, des panneaux extraits, pas de réécriture

`UsersView.vue` (liste), `UserDetailView.vue` (détail), `CampaignsView.vue` (campagnes).
Le corps des blocs existants est **déplacé dans des composants**, pas réécrit :

| Composant | Origine |
|---|---|
| `AccountPlanPanel.vue` | bloc (a) de `PlansView` : plan effectif, lignes de droit, adhésions, dialogues grant/enrol/raison |
| `AccountRolesPanel.vue` | boutons de rôle + logique `rolesInScope` de `RolesView` |
| `AccountHistoryPanel.vue` | table d'historique de `RolesView` |
| `CuratorReliabilityDrawer.vue` | existant — réutilisé **en panneau** sur la page détail |

*Alternative écartée* : garder une seule vue « users » avec un détail en drawer. Rejetée :
un drawer n'est pas adressable, or l'adressabilité est tout l'objet du change.

### D2 — La page détail charge ses propres données, par id

`UserDetailView` ne suppose jamais que la liste a été visitée. Au montage (et à chaque
changement de `route.params.userId`) :

1. `roles.loadAccount(userId)` — nouveau, `ListAccounts({ ids: [userId], limit: 1 })`,
   exposé en `Async<AccountRow | null>` (`null` = compte inconnu → état localisé
   « compte introuvable »).
2. si `showPlans` (admin scope `music`) : `plans.lookup(userId)`.
3. à la demande : `roles.listGrants(userId)`, `roles.loadReliability(userId)`.

*Alternative écartée* : passer la `AccountRow` en `props` depuis la liste. Rejetée : casse
au premier F5 et au premier favori.

### D3 — Le store `plans` est un singleton : purger avant d'afficher

`lookupResult` est un état de store partagé. En naviguant de `/users/A` vers `/users/B` on
afficherait une frame des droits de **A** sous le nom de **B** — inacceptable sur un écran
où l'on révoque des droits. La vue appelle donc `plans.clearLookup()` **avant** chaque
chargement (montage et watch du param), et le rendu ne s'appuie que sur l'union `Async` :
`idle`/`loading` → squelette, jamais la donnée précédente. Même règle pour `roles.grants`
et `roles.reliability`, remis à `idle` au changement de compte.

### D4 — Le lien est un vrai lien

La cellule handle est un `<RouterLink>` vers `/users/{id}` (focusable au clavier, ouvrable
en nouvel onglet, lisible par un lecteur d'écran) ; le `@click` sur le `<tr>` n'est qu'un
confort qui ignore les clics portant sur un contrôle interactif. Une ligne rendue cliquable
par le seul `@click` sur `<tr>` serait inatteignable au clavier — l'annuaire est un outil
d'admin utilisé au clavier.

### D5 — Les rôles : tous les scopes autorisés d'un coup sur le détail

Sur la liste, le sélecteur de scope reste (il décide ce qu'affiche la colonne `RÔLES`).
Sur le détail il disparaît : la page montre un bloc par scope **autorisé** avec ses
bascules. Un `global/admin` voit `global`/`music`/`live` sans manipuler un sélecteur pour
répondre à « quels rôles a ce compte ? » ; un `music/admin` ne voit que `music`. Les
mutations passent par les mêmes `grantRole`/`revokeRole`, scope explicite — l'autorisation
scope-matched reste celle du serveur.

*Conséquence store* : `roles.grant/revoke` re-liste aujourd'hui la page courante de
l'annuaire. Depuis le détail il faut rafraîchir **le compte**, pas la liste : la fonction
prend un rafraîchissement optionnel, le détail passe `loadAccount(userId)`.

### D6 — Routes et redirections

```
/users            name: users          meta: { admin: true }
/users/:userId    name: user-detail    meta: { admin: true }, props: true
/campaigns        name: campaigns      meta: { admin: true }
/roles  → redirect /users
/plans  → redirect /campaigns
```

Les redirections restent en place : `bo.cymbra.app/roles` et `/plans` sont dans les favoris
des admins, et une 404 silencieuse sur un outil interne se paie en tickets. Nav :
`nav.users` (icône `roles` conservée) et `nav.campaigns` (icône `plans` conservée).

### D7 — i18n : renommer les espaces de clés, pas les traduire deux fois

`roles.*` → `users.*`, et les clés du bloc compte de `plans.*` restent dans `plans.*`
(elles décrivent des concepts « plan », pas un écran) ; seuls les libellés d'écran
(`plans.title`, `plans.intro`, `nav.plans`) deviennent ceux de « Campagnes », et les
libellés du détail arrivent en `users.*`. `en` et `fr` sont modifiés dans la même passe —
la CI vérifie l'alignement.

## Risks / Trade-offs

- **Les membres d'une campagne n'affichent souvent qu'un id.** `handleFor()` résout le
  handle depuis la page d'annuaire chargée ou depuis le dernier lookup ; sans lookup sur
  `/campaigns`, la liste des membres tombera presque toujours sur `IdBadge`. → Le lien vers
  `/users/{id}` répare l'usage (un clic donne le handle et tout le reste) ; la résolution
  handle côté serveur pour l'export reste hors périmètre, l'export CSV conserve son
  comportement actuel.
- **Deux niveaux d'autorisation sur une même page.** Un admin non-`music` ne doit pas voir
  un bloc vide « Abonnement ». → Le bloc entier est absent (`v-if="showPlans"`), et aucun
  RPC plan n'est émis — c'est un scénario de spec, pas une intention.
- **Régression d'e2e volumineuse.** `e2e/roles.spec.ts` et `e2e/plans.spec.ts` couvrent les
  deux écrans existants. → Ils sont réécrits en trois fichiers (`users`, `user-detail`,
  `campaigns`) sur le même seam de faux client ; la couverture par geste est conservée
  geste par geste, pas globalement.
- **PlansView est gros et ses dialogues sont partagés** (raison/confirmation, focus dans le
  `<dialog>`, Escape). → Les dialogues suivent le bloc dans `AccountPlanPanel`, ceux des
  campagnes restent dans `CampaignsView` ; le comportement de focus (corrigé en #306) est
  déplacé tel quel, pas réinventé.

## Migration Plan

Un seul déploiement, pas d'état à migrer :

1. Routes + redirections + nav (les anciennes URL fonctionnent immédiatement).
2. Extraction des panneaux, puis les trois vues.
3. i18n `en`/`fr` dans la même passe.
4. Tests unitaires et e2e réécrits ; `melos run analyze`, `yarn test`, `yarn e2e`.

Rollback : revert du commit — aucune donnée, aucun contrat serveur touché.

## Open Questions

- Faut-il donner à « Utilisateurs » une icône distincte de l'actuel bouclier de « Rôles »
  (qui évoquait la gestion des rôles) ? Cosmétique, décidable à l'implémentation.
- La colonne `RÔLES` de la liste garde-t-elle le sélecteur de scope, ou affiche-t-elle les
  rôles de tous les scopes autorisés préfixés par leur scope ? Le sélecteur est conservé
  par défaut (moins de churn) ; à réévaluer si la colonne paraît pauvre en usage réel.
