# lingua-stats — agrégats d'apprentissage : serveur

## ADDED Requirements

### Requirement: Agrégats par jour, par langue et par appareil
`StatsService` SHALL stocker des agrégats d'apprentissage à la clé (jour UTC, langue étudiée, appareil) — expositions, mots appris, révisions faites — poussés par upsert idempotent depuis chaque appareil ; aucun évènement fin horodaté ne SHALL être stocké serveur, le grain jour × langue étant le plus fin autorisé.

#### Scenario: Deux appareils actifs le même jour
- **WHEN** l'utilisateur fait 20 révisions sur son Mac et 10 sur son iPhone le même jour
- **THEN** chaque appareil upserte sa propre ligne d'agrégat et la lecture consolidée du jour rapporte 30 révisions

#### Scenario: Upsert rejoué sans double compte
- **WHEN** un appareil re-pousse la ligne d'agrégat d'un jour déjà envoyé
- **THEN** la ligne est remplacée (upsert par clé), jamais additionnée à elle-même

### Requirement: Lecture consolidée multi-appareils
`StatsService` SHALL exposer une lecture des agrégats consolidés — sommés sur les appareils, en séries par jour et par langue, sur une plage de dates demandée — pour l'utilisateur authentifié uniquement (jamais les stats d'un autre compte).

#### Scenario: Série sur trente jours
- **WHEN** un client demande les stats des 30 derniers jours pour l'anglais
- **THEN** la réponse contient au plus une valeur par jour et par mesure, sommée sur tous les appareils du compte
