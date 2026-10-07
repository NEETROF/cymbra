// The words a card uses for a form's grammar (reading/grammar-labels.ts), in French — the source
// module (add-lingua-interface-language). The pack speaks Universal Dependencies codes; the
// reader never sees one. The names of parts of speech, gender, number and person are the
// interface's own; a tense's name depends on the studied language. The English and Spanish
// entries are drafted here and settled by add-lingua-card-wording (change 18).

export const grammar = {
  // — Parts of speech —
  posAdjective: "adjectif",
  posAdposition: "préposition",
  posAdverb: "adverbe",
  posAuxiliary: "auxiliaire",
  posCoordinatingConjunction: "conjonction",
  posDeterminer: "déterminant",
  posInterjection: "interjection",
  posNoun: "nom",
  posNumeral: "nombre",
  posParticle: "particule",
  posPronoun: "pronom",
  posProperNoun: "nom propre",
  posSubordinatingConjunction: "conjonction",
  posVerb: "verbe",
  // — Gender, person, number —
  genderMasculine: "masculin",
  genderFeminine: "féminin",
  genderNeuter: "neutre",
  genderCommon: "commun",
  person1: "1re personne",
  person2: "2e personne",
  person3: "3e personne",
  ordinal1: "1re",
  ordinal2: "2e",
  ordinal3: "3e",
  ofSingular: "du singulier",
  ofPlural: "du pluriel",
  singular: "singulier",
  plural: "pluriel",
  // — The studied languages' tenses, as French schools name them —
  englishPast: "prétérit",
  englishPresent: "présent",
  spanishIndicativePresent: "présent de l’indicatif",
  spanishIndicativeImperfect: "imparfait de l’indicatif",
  spanishPreterite: "passé simple",
  spanishFuture: "futur",
  spanishConditional: "conditionnel",
  spanishSubjunctivePresent: "présent du subjonctif",
  spanishSubjunctiveImperfect: "imparfait du subjonctif",
  spanishSubjunctiveFuture: "futur du subjonctif",
  spanishImperative: "impératif",
  // — Forms —
  comparative: "comparatif",
  superlative: "superlatif",
  infinitive: "infinitif",
  pastParticiple: "participe passé",
  /** « participe passé féminin pluriel » — `agreement` is `genderNumber` without the singular. */
  pastParticipleAgreed: (agreement: string) => `participe passé ${agreement}`,
  ingForm: "forme en -ing",
  gerund: "gérondif",
  // — How the names compose —
  /** « du passé simple », « de l’imparfait de l’indicatif »: a tense after a person and a number. */
  ofName: (name: string) => `du ${name}`,
  ofNameElided: (name: string) => `de l’${name}`,
  articleMasculine: "le",
  articleFeminine: "la",
  articleElided: "l’",
  articlePlural: "les",
  /** « féminin pluriel », « masculin singulier ». */
  genderNumber: (gender: string, number: string) => `${gender} ${number}`,
  /** A sense group's heading: « nom féminin ». */
  posGender: (pos: string, gender: string) => `${pos} ${gender}`,
  /** « 3e personne du singulier du passé simple ». */
  personNumberTense: (person: string, number: string, ofTense: string) => `${person} ${number} ${ofTense}`,
  /** "a, b et c": the separator, and the last join. */
  listSeparator: ", ",
  listLast: (head: string, last: string) => `${head} et ${last}`,
  /** « 1re et 3e personnes » — `ordinals` is the joined list. */
  persons: (ordinals: string) => `${ordinals} personnes`,
  /** "de go", "d’eat": the dictionary form a reading is of, elided before a vowel. */
  ofWord: (word: string) => `de ${word}`,
  ofWordElided: (word: string) => `d’${word}`,
  /** A name with its article: « le futur », « l’infinitif ». */
  withArticle: (article: string, name: string) => `${article} ${name}`,
  withArticleElided: (name: string) => `l’${name}`,
  /** « doesn't » = does + not: the pieces of a word the pre-pass split. */
  pieces: (written: string) => `« ${written} » = `,
  piecesSeparator: " + ",
  /** « prétérit et participe passé de walk » — `names` joined, `ofWord` as `ofWord` writes it. */
  formOf: (names: string, ofWord: string) => `${names} ${ofWord}`,
  /** « peut aussi être le prétérit de put » — `names` with their articles. */
  mayAlsoBe: (names: string, ofWord: string) => `peut aussi être ${names} ${ofWord}`,
};
