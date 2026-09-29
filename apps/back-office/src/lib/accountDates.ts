// The dates the console shows about an account (change: add-directory-account-dates).
//
// Both come from the directory row: the sign-up date is `createdAt`, the last sign-in is
// the most recent `lastSeenAt` among the apps the account used. It is derived here rather
// than sent by the server so it can never disagree with the app icons' tooltips.

/** A unix-seconds timestamp as a date in `locale`. */
export function formatUnixDate(seconds: bigint, locale: string): string {
  return new Date(Number(seconds) * 1000).toLocaleDateString(locale);
}

/** The latest last use among `apps`, or `null` when the account never signed in to one. */
export function lastSignIn(apps: readonly { lastSeenAt: bigint }[]): bigint | null {
  return apps.reduce<bigint | null>((max, a) => (max === null || a.lastSeenAt > max ? a.lastSeenAt : max), null);
}
