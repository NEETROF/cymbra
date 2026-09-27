import type { NavIcon } from "@/lib/navigation";

// Minimal line icons (Lucide-style paths) so the nav reads like the mockup without
// pulling an icon dependency. Shared by the sidebar and the Users directory's app
// column (change: add-directory-app-usage), so an app wears the same icon everywhere.
export const ICONS: Record<NavIcon, string> = {
  queue: "M3 5h18M3 12h18M3 19h12",
  catalog: "M9 18V5l12-2v13M9 13l12-2M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0Zm12-2a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
  privateScores: "M5 11h14v10H5zM8 11V7a4 4 0 0 1 8 0v4",
  users:
    "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm13 10v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75",
  flags: "M4 22V4m0 0 8-2 8 3v9l-8-2-8 2",
  campaigns: "M12 2 15 8.5 22 9.3 17 14.1 18.2 21 12 17.6 5.8 21 7 14.1 2 9.3 9 8.5Z",
  soundfonts: "M9 18V5l12-2v13M9 13l12-2M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0Zm12-2a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
  usage: "M3 3v18h18M7 15l4-4 3 3 5-6",
  notifications: "M18 8a6 6 0 1 0-12 0c0 7-3 9-3 9h18s-3-2-3-9M13.7 21a2 2 0 0 1-3.4 0",
  lingua: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20M4 19.5A2.5 2.5 0 0 0 6.5 22H20V2H6.5A2.5 2.5 0 0 0 4 4.5v15z",
  jobs: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20ZM12 6v6l4 2",
};

/** The apps the directory shows, in display order, with their icon and i18n label. */
export const APP_BADGES = [
  { app: "music", icon: ICONS.catalog, label: "users.appMusic" },
  { app: "lingua", icon: ICONS.lingua, label: "users.appLingua" },
] as const;
