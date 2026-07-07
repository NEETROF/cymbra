<!--
  BROUILLON — à relire et compléter (champs «...») avant publication, et idéalement
  à faire valider par un juriste. Rédigé pour Cymbra (app FR, RGPD), aligné sur les
  données réellement traitées par le backend. Dernière mise à jour : «JJ/MM/AAAA».
-->

# Politique de confidentialité — Cymbra

**Dernière mise à jour : 07/07/2026**

La présente politique explique quelles données personnelles l'application **Cymbra**
traite, pourquoi, sur quelle base légale, avec qui elles sont partagées, combien de
temps elles sont conservées, et quels sont vos droits.

## 1. Responsable du traitement

**NEETROF — SASU, 948723887**, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCE.
Contact : **gfortin@neetrof.fr**.

## 2. Données que nous traitons

Nous appliquons la **minimisation** : nous ne collectons que ce qui est nécessaire au
fonctionnement du compte et de l'application.

| Donnée | Origine | Finalité |
|---|---|---|
| Adresse email | vous (inscription email) ou votre fournisseur (Google/Apple) | identifiant de compte, vérification, réinitialisation de mot de passe |
| Mot de passe (empreinte **argon2**, jamais en clair) | vous (inscription email) | authentification |
| Identifiant de connexion externe (identifiant Google/Apple « sub ») | Google / Apple | connexion via « Se connecter avec Google/Apple » |
| Pseudo (« handle ») et nom affiché | vous | identification dans l'application |
| Préférences applicatives | vous | mémoriser vos réglages |
| Jetons de session (refresh tokens) | généré au login | maintenir votre session connectée |
| Journaux techniques (adresse IP, horodatages, erreurs) | serveur | sécurité, prévention des abus, bon fonctionnement |

Nous **ne** collectons **pas** de données de localisation précise, ne vendons aucune
donnée, et n'utilisons pas de publicité tierce ni de traceurs publicitaires.

## 3. Bases légales (RGPD art. 6)

- **Exécution du contrat** : création et gestion de votre compte, fourniture de
  l'application.
- **Intérêt légitime** : sécurité, prévention de la fraude/des abus, limitation de
  débit, bon fonctionnement.
- **Consentement** : connexion via Google/Apple (vous choisissez ce mode).

## 4. Sous-traitants et tiers

Nous partageons le strict nécessaire avec des prestataires agissant pour notre compte :

- **OVHcloud** (France, UE) — hébergement du serveur et des sauvegardes.
- **Brevo** (UE) — envoi des emails transactionnels (vérification, réinitialisation).
- **Google** / **Apple** — uniquement si vous utilisez leur connexion (vérification de
  votre identité via leur jeton).

Vos données sont **hébergées dans l'Union européenne** (France). Nous ne procédons à
aucun transfert hors UE.

## 5. Durée de conservation

- Données de compte : conservées **tant que votre compte existe**.
- **Suppression du compte** : lorsque vous supprimez votre compte (voir §7), vos
  données personnelles (email, empreinte de mot de passe, identité externe, pseudo,
  nom, sessions) sont **effacées**.
- Sauvegardes : les sauvegardes chiffrées tournent sur une fenêtre glissante
  (14 jours) puis sont écrasées ; une donnée supprimée disparaît donc au plus tard à
  l'expiration de cette fenêtre.
- Journaux techniques : 7 jours.

## 6. Sécurité

Chiffrement en transit (**TLS**), mots de passe stockés sous forme d'empreinte
**argon2** (jamais en clair), sauvegardes **chiffrées** et stockées hors du serveur,
accès serveur restreint. Aucune mesure n'étant infaillible, nous ne pouvons garantir
une sécurité absolue.

## 7. Vos droits (RGPD)

Vous disposez des droits d'**accès**, de **rectification**, d'**effacement**, de
**limitation**, d'**opposition** et de **portabilité**.

- **Effacement (droit à l'oubli)** : vous pouvez **supprimer votre compte directement
  dans l'application** (Réglages → Supprimer mon compte). La suppression est
  irréversible et efface vos données personnelles.
- Pour toute autre demande, écrivez à **privacy@cymbra.app**. Vous pouvez aussi
  introduire une réclamation auprès de la **CNIL** (www.cnil.fr).

## 8. Mineurs

Cymbra n'est pas destinée aux enfants de moins de 12 ans ; nous ne collectons pas
sciemment leurs données.

## 9. Modifications

Nous pouvons mettre à jour cette politique ; la date « Dernière mise à jour » ci-dessus
sera modifiée en conséquence. En cas de changement important, nous vous en informerons.

## 10. Contact

**gfortin@neetrof.fr** — NEETROF, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCE.
