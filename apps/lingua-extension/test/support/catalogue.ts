/// <reference types="vite/client" />
import type { InterfaceLanguage } from "@/i18n/index.ts";

// The catalogue's modules and the texts they produce — shared by test/i18n.spec.ts (the drafts'
// parity with the French, add-lingua-interface-language D6) and test/italian-glossary.spec.ts (one
// Italian vocabulary, review-lingua-italian-wording D3), which once held this alone. Not a spec: it
// declares no test, and vitest does not collect it (test/**/*.spec.ts).

export type Catalogue = Record<string, unknown>;

/** Every module of the catalogue, by path (`../../src/i18n/<language>/<surface>.ts`). */
export const modules = import.meta.glob<Catalogue>("../../src/i18n/{fr,en,es,it}/*.ts", { eager: true });

/** A surface's catalogue in a language: the one object its module exports. */
export function catalogue(language: InterfaceLanguage, surface: string): Catalogue {
  const mod = modules[`../../src/i18n/${language}/${surface}.ts`];
  if (!mod) throw new Error(`no ${language} module for ${surface}`);
  const names = Object.keys(mod);
  if (names.length !== 1) throw new Error(`${language}/${surface} exports ${names.length} objects, not one`);
  return mod[names[0]] as Catalogue;
}

/**
 * What a slot message is called with: one sentinel per part, a letter no draft writes, and a
 * different alphabet at each depth of nesting, so that a message returned by a message — an inner
 * form that dropped its own part — is caught even though its text carries the outer one.
 */
export const SENTINELS: readonly (readonly string[])[] = [[..."αβγδ"], [..."εζηθ"], [..."ικλμ"]];

export type Message = (...args: string[]) => unknown;

export interface Text {
  path: string;
  text: string;
  /** The sentinels given on the way to this text: each must appear in it. */
  given: string[];
}

/** The parts each French message takes, by path — what a draft at the same path is called with. */
export type Slots = ReadonlyMap<string, number>;

export function slotsOf(value: unknown, path: string, into = new Map<string, number>(), depth = 0): Slots {
  if (typeof value === "function") {
    into.set(path, value.length);
    slotsOf((value as Message)(...(SENTINELS[depth] ?? [])), `${path}()`, into, depth + 1);
  } else if (value && typeof value === "object") {
    for (const [key, v] of Object.entries(value as Catalogue)) slotsOf(v, `${path}.${key}`, into, depth);
  }
  return into;
}

/**
 * Every text a value can produce. A message is called with as many sentinels as the French message
 * at its path takes — its own count would let a draft that dropped a part pass — and its texts must
 * contain each one. A `many` form the French lacks takes what the French `other` takes.
 */
export function texts(value: unknown, path: string, slots: Slots, depth = 0, given: string[] = []): Text[] {
  if (typeof value === "string") return [{ path, text: value, given }];
  if (typeof value === "function") {
    const count = Math.max(1, slots.get(path) ?? slots.get(path.replace(/\.many$/, ".other")) ?? value.length);
    const alphabet = SENTINELS[depth] ?? [];
    if (count > alphabet.length) throw new Error(`${path}: more parts, or deeper, than the sentinels cover`);
    const args = alphabet.slice(0, count);
    return texts((value as Message)(...args), `${path}()`, slots, depth + 1, [...given, ...args]);
  }
  if (value && typeof value === "object") {
    return Object.entries(value as Catalogue).flatMap(([key, v]) => texts(v, `${path}.${key}`, slots, depth, given));
  }
  throw new Error(`${path}: a catalogue value is a string, a function or an object of them`);
}
