import type { Ref } from "vue";

// A discriminated union for an async resource — the same shape as the back office's
// `lib/async.ts` (structurally identical, so the two interoperate). Modelling the four
// states as ONE value makes impossible states unrepresentable.
export type Async<T, E = string> =
  | { readonly status: "idle" }
  | { readonly status: "loading" }
  | { readonly status: "success"; readonly data: T }
  | { readonly status: "error"; readonly error: E };

export const idle: Async<never> = { status: "idle" };
export const loading: Async<never> = { status: "loading" };
export const success = <T>(data: T): Async<T, never> => ({ status: "success", data });
export const failure = <E = string>(error: E): Async<never, E> => ({ status: "error", error });

/** Default error → message: the `Error` message, else the stringified value. */
export function messageOf(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/**
 * Fold a promise into an `Async` ref: `loading` while it runs, then `success` or
 * `error` (message via `mapError`, default `messageOf`; the back office passes its
 * localizing `humanError`). Never throws — callers branch on the returned state.
 */
export async function run<T>(
  ref: Ref<Async<T>>,
  fn: () => Promise<T>,
  mapError: (e: unknown) => string = messageOf,
): Promise<Async<T>> {
  ref.value = loading;
  try {
    ref.value = success(await fn());
  } catch (e) {
    ref.value = failure(mapError(e));
  }
  return ref.value;
}
