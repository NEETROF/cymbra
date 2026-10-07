import type { sync as fr } from "../fr/sync.ts";

// The sync block's copy in Spanish — a draft after the French (src/i18n/README.md).

export const sync: typeof fr = {
  notSynced: "Aún no sincronizado en este dispositivo.",
  syncedJustNow: "Sincronizado ahora mismo.",
  syncedMinutesAgo: (n) => `Sincronizado hace ${n} min.`,
  syncedHoursAgo: (n) => `Sincronizado hace ${n} h.`,
  lastSyncOn: (date) => `Última sincronización el ${date}.`,
  storageFull: "La memoria de la extensión está llena en este dispositivo: restablece tus datos locales en Ajustes.",
  unavailable: "Servidor inaccesible: reintenta más tarde.",
  sessionExpired: "Sesión caducada: vuelve a iniciar sesión más arriba, en Cuenta.",
  conflict: "Ya hay una operación en curso: reintenta dentro de un momento.",
  failed: "La sincronización ha fallado: reintenta más tarde.",
};
