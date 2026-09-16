import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { createMemoryHistory } from "vue-router";
import { canOpen, landing, NAV_SECTIONS, visibleSections, type Access } from "@/lib/navigation";
import { createAppRouter, FORMER_PATHS } from "@/router";
import { useAuthStore } from "@/stores/auth";
import { makeJwt } from "./fakes";

// Change: restructure-back-office-navigation. The sidebar and the route guard read one
// access rule per page, so what is asserted here is that the two cannot disagree: an
// entry is listed exactly when its page opens, for every operator profile.

type Roles = Record<string, string[]>;

/** Claims as the auth store derives them: the flat role set is every scoped role. */
function claims(rolesByScope: Roles) {
  return { roles: [...new Set(Object.values(rolesByScope).flat())], rolesByScope };
}

const PROFILES = {
  globalAdmin: { global: ["admin"] },
  musicAdmin: { music: ["admin"] },
  musicModerator: { music: ["moderator"] },
  linguaAdmin: { lingua: ["admin"] },
  liveModerator: { live: ["moderator"] },
} satisfies Record<string, Roles>;

function signIn(rolesByScope: Roles) {
  useAuthStore().setToken(
    makeJwt({ sub: "op-1", roles: claims(rolesByScope).roles, roles_by_scope: rolesByScope, exp: 4102444800 }),
  );
}

async function openAt(path: string, rolesByScope?: Roles) {
  if (rolesByScope) signIn(rolesByScope);
  const router = createAppRouter(createMemoryHistory());
  await router.push(path);
  await router.isReady();
  return router;
}

describe("canOpen", () => {
  const cases: [string, Access | undefined, Roles, boolean][] = [
    [
      "music moderator rule admits a music moderator",
      { role: "moderator", scope: "music" },
      { music: ["moderator"] },
      true,
    ],
    ["music moderator rule admits a music admin", { role: "moderator", scope: "music" }, { music: ["admin"] }, true],
    [
      "music moderator rule refuses a live moderator",
      { role: "moderator", scope: "music" },
      { live: ["moderator"] },
      false,
    ],
    ["music admin rule refuses a music moderator", { role: "admin", scope: "music" }, { music: ["moderator"] }, false],
    ["music admin rule refuses a lingua admin", { role: "admin", scope: "music" }, { lingua: ["admin"] }, false],
    ["a global admin counts in every scope", { role: "admin", scope: "lingua" }, { global: ["admin"] }, true],
    ["an unscoped admin rule admits an admin of any scope", { role: "admin" }, { live: ["admin"] }, true],
    ["an unscoped admin rule refuses a moderator", { role: "admin" }, { music: ["moderator"] }, false],
    [
      "an unscoped moderator rule admits a moderator of any scope",
      { role: "moderator" },
      { live: ["moderator"] },
      true,
    ],
    ["a page without a rule stays closed, even to a global admin", undefined, { global: ["admin"] }, false],
  ];
  it.each(cases)("%s", (_, access, roles, expected) => {
    expect(canOpen(access, claims(roles))).toBe(expected);
  });
});

describe("sidebar", () => {
  beforeEach(() => setActivePinia(createPinia()));

  const sidebar = (roles: Roles) => {
    const router = createAppRouter(createMemoryHistory());
    return visibleSections(router, claims(roles)).map((s) => [s.id, s.entries.map((e) => e.route)]);
  };

  it("every entry names a route that exists and declares an access rule", () => {
    const router = createAppRouter(createMemoryHistory());
    for (const entry of NAV_SECTIONS.flatMap((s) => s.entries)) {
      expect(router.hasRoute(entry.route), entry.route).toBe(true);
      expect(router.resolve({ name: entry.route }).meta.access, entry.route).toBeDefined();
    }
  });

  it("a global admin sees the three sections, in order, with every entry", () => {
    expect(sidebar(PROFILES.globalAdmin)).toEqual(NAV_SECTIONS.map((s) => [s.id, s.entries.map((e) => e.route)]));
  });

  it("a music admin sees all of Music and the admin pages, but neither Lingua nor Jobs", () => {
    expect(sidebar(PROFILES.musicAdmin)).toEqual([
      [
        "music",
        ["music-queue", "music-catalog", "music-private-scores", "music-soundfonts", "music-campaigns", "music-usage"],
      ],
      ["admin", ["admin-users", "admin-flags", "admin-notifications"]],
    ]);
  });

  it("a music moderator sees only what they moderate", () => {
    expect(sidebar(PROFILES.musicModerator)).toEqual([["music", ["music-queue", "music-catalog"]]]);
  });

  it("a lingua-only admin is offered no Music page — not sound fonts, campaigns or usage", () => {
    expect(sidebar(PROFILES.linguaAdmin)).toEqual([
      ["lingua", ["lingua-overview"]],
      ["admin", ["admin-users", "admin-flags", "admin-notifications"]],
    ]);
  });

  it("a moderator of another product has no section at all", () => {
    expect(sidebar(PROFILES.liveModerator)).toEqual([]);
  });

  it("the sidebar lists an entry exactly when the guard opens its page", async () => {
    for (const roles of Object.values(PROFILES)) {
      setActivePinia(createPinia());
      signIn(roles);
      const listed = new Set(sidebar(roles).flatMap(([, routes]) => routes as string[]));
      for (const entry of NAV_SECTIONS.flatMap((s) => s.entries)) {
        const router = createAppRouter(createMemoryHistory());
        await router.push({ name: entry.route });
        const opened = router.currentRoute.value.name === entry.route;
        expect(opened, `${JSON.stringify(roles)} → ${entry.route}`).toBe(listed.has(entry.route));
      }
    }
  });
});

describe("landing", () => {
  beforeEach(() => setActivePinia(createPinia()));
  const router = () => createAppRouter(createMemoryHistory());

  it("sends a visitor without a session to sign-in", () => {
    expect(landing(router(), { isAuthenticated: false, claims: claims({}) })).toEqual({ name: "signin" });
  });

  it.each([
    ["globalAdmin", "music-queue"],
    ["musicAdmin", "music-queue"],
    ["musicModerator", "music-queue"],
    ["linguaAdmin", "lingua-overview"],
    ["liveModerator", "denied"],
  ] as const)("lands a %s on %s", (profile, name) => {
    expect(landing(router(), { isAuthenticated: true, claims: claims(PROFILES[profile]) })).toEqual({ name });
  });
});

describe("router", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("every page declares an access rule unless it is public", () => {
    const router = createAppRouter(createMemoryHistory());
    const pages = router.getRoutes().filter((r) => !r.redirect);
    expect(pages.length).toBeGreaterThan(0);
    for (const page of pages) {
      expect(page.meta.public === true || page.meta.access !== undefined, page.path).toBe(true);
    }
  });

  it("every page path starts with its section prefix", () => {
    const router = createAppRouter(createMemoryHistory());
    for (const page of router.getRoutes().filter((r) => !r.redirect && !r.meta.public)) {
      const section = String(page.name).split("-")[0];
      expect(page.path.startsWith(`/${section}/`), page.path).toBe(true);
    }
  });

  it("sends a visitor without a session to sign-in", async () => {
    const router = await openAt("/music/catalog");
    expect(router.currentRoute.value.name).toBe("signin");
  });

  it("lands a signed-in operator on their first page from the root and from an unknown path", async () => {
    expect((await openAt("/", PROFILES.musicModerator)).currentRoute.value.path).toBe("/music/queue");
    expect((await openAt("/nowhere/at-all", PROFILES.linguaAdmin)).currentRoute.value.path).toBe("/lingua/overview");
  });

  it("sends a signed-in operator away from sign-in to their landing page", async () => {
    expect((await openAt("/signin", PROFILES.linguaAdmin)).currentRoute.value.name).toBe("lingua-overview");
  });

  it("shows access denied to an account with no page here, and does not loop", async () => {
    const router = await openAt("/", PROFILES.liveModerator);
    expect(router.currentRoute.value.name).toBe("denied");
    await router.push("/admin/jobs");
    expect(router.currentRoute.value.name).toBe("denied");
  });

  it("lands a refused page on the operator's landing page", async () => {
    const router = await openAt("/admin/jobs", PROFILES.musicAdmin);
    expect(router.currentRoute.value.path).toBe("/music/queue");
  });

  it("opens a section root on the section's first page", async () => {
    expect((await openAt("/admin", PROFILES.globalAdmin)).currentRoute.value.path).toBe("/admin/users");
    expect((await openAt("/lingua", PROFILES.globalAdmin)).currentRoute.value.path).toBe("/lingua/overview");
    expect((await openAt("/music", PROFILES.globalAdmin)).currentRoute.value.path).toBe("/music/queue");
  });

  it.each(Object.entries(FORMER_PATHS))("redirects the former %s to %s", async (path, name) => {
    const concrete = path.replace(":userId", "u-ada");
    const router = await openAt(concrete, PROFILES.globalAdmin);
    expect(router.currentRoute.value.name).toBe(name);
  });

  it("keeps the id and the query of a former link", async () => {
    const owner = "0190aa00-0000-7000-8000-000000000001";
    const takedown = await openAt(`/takedowns?owner=${owner}`, PROFILES.musicAdmin);
    expect(takedown.currentRoute.value.fullPath).toBe(`/music/private-scores?owner=${owner}`);

    const account = await openAt("/users/u-ada?tab=roles", PROFILES.musicAdmin);
    expect(account.currentRoute.value.fullPath).toBe("/admin/users/u-ada?tab=roles");
  });

  it("does not let a former path bypass access", async () => {
    const router = await openAt("/jobs", PROFILES.musicModerator);
    expect(router.currentRoute.value.path).toBe("/music/queue");
  });
});
