import { describe, expect, it } from "vitest";
import { formatUnixDate, lastSignIn } from "@/lib/accountDates";

describe("lastSignIn", () => {
  it("is the most recent last use among the apps", () => {
    expect(lastSignIn([{ lastSeenAt: 100n }, { lastSeenAt: 300n }, { lastSeenAt: 200n }])).toBe(300n);
  });

  it("is null for an account that never signed in to an app", () => {
    expect(lastSignIn([])).toBeNull();
  });
});

describe("formatUnixDate", () => {
  it("formats unix seconds as a date in the given locale", () => {
    // Noon UTC, so the calendar day is the same in every test-runner time zone.
    const s = BigInt(Date.UTC(2026, 2, 2, 12) / 1000);
    expect(formatUnixDate(s, "en-US")).toBe("3/2/2026");
    expect(formatUnixDate(s, "fr-FR")).toBe("02/03/2026");
  });
});
