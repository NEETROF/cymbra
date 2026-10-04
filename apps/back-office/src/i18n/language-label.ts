// The name of a Lingua studied language, from its ISO 639-1 code, in the console's language
// (change: add-admin-lingua-language-labels). A language the console has no name for keeps its
// code, so one that reports before it is named still shows — never a missing-key marker.
export function languageLabel(code: string, t: (key: string) => string, te: (key: string) => boolean): string {
  const key = `lingua.languages.${code}`;
  return te(key) ? t(key) : code;
}
