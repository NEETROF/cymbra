// The onboarding page's copy (onboarding.html's static text and onboarding/level-row.ts), in
// French — the source module (add-lingua-interface-language).

export const onboarding = {
  pageTitle: "Bienvenue — Cymbra Lingua",
  heading: "Bienvenue dans Cymbra Lingua",
  lead: "Surligne les mots que tu ne connais pas encore pendant que tu lis, puis révise-les au bon moment — sans quitter ta page.",
  whichLanguages: "Quelles langues apprends-tu ?",
  levelNote:
    "Les mots en dessous de ton niveau ne seront pas surlignés. Rien n'est présumé tant que tu n'as pas choisi — et tu pourras changer ça à tout moment dans les réglages.",
  toStart: "Pour commencer",
  stepPin: "Épingle l'icône Cymbra Lingua dans la barre du navigateur.",
  stepOpen: "Ouvre une page dans la langue que tu apprends et clique l'icône pour l'analyser.",
  stepClick: "Clique un mot surligné pour le traduire ou l'ajouter à ton deck.",
  accountTitle: "Retrouve tes mots partout (facultatif)",
  accountLead:
    "Avec un compte Cymbra — le même que dans Cymbra Music — tes mots, ton deck et tes statistiques se synchronisent entre tes appareils. Sans compte, tes mots, ton deck et tes statuts restent sur cet appareil.",
  createAccount: "Créer un compte",
  later: "Plus tard",
  // — The level question (onboarding/level-row.ts) —
  beginnerChip: "Débutant — je pars de zéro",
  /** `level` as `languages.levelName` writes it. */
  levelSaved: (level: string) => `Niveau enregistré : ${level}. Tu peux fermer cet onglet et commencer à lire.`,
  beginnerSaved: "C'est noté — on part de zéro. Tu peux fermer cet onglet et commencer à lire.",
  // — The anonymous counts' first-run sentence (add-lingua-usage-counts D7) —
  usageInform: "Lingua compte de façon anonyme quelques étapes — installation, premiers pas, retours —, sans aucun identifiant ; rien de ce que tu lis ne quitte ton appareil. Tu peux couper ces comptages dans Réglages → Données.",
  usageAsk: "Autorises-tu Lingua à compter de façon anonyme quelques étapes — installation, premiers pas, retours —, sans aucun identifiant ? Rien de ce que tu lis ne quitte ton appareil, et tu pourras changer d'avis dans Réglages → Données.",
  usageYes: "Oui, compter",
  usageNo: "Non merci",
  usageYesDone: "Merci ! Tu peux couper ces comptages à tout moment dans Réglages → Données.",
  usageNoDone: "C'est noté : rien ne sera compté.",
  usagePrivacy: "Politique de confidentialité",
};
