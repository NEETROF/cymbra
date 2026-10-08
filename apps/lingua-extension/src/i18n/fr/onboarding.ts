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
    "Avec un compte Cymbra — le même que dans Cymbra Music — tes mots, ton deck et tes statistiques se synchronisent entre tes appareils. Sans compte, tout reste sur cet appareil.",
  createAccount: "Créer un compte",
  later: "Plus tard",
  // — The level question (onboarding/level-row.ts) —
  beginnerChip: "Débutant — je pars de zéro",
  /** `level` as `languages.levelName` writes it. */
  levelSaved: (level: string) => `Niveau enregistré : ${level}. Tu peux fermer cet onglet et commencer à lire.`,
  beginnerSaved: "C'est noté — on part de zéro. Tu peux fermer cet onglet et commencer à lire.",
};
