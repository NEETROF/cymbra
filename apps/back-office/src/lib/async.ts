import type { Ref } from "vue";
import { humanError } from "./errors";

// A discriminated union for an async resource. Modelling the four states as ONE
// value (instead of separate `loading`/`error`/`data` refs) makes impossible
// states unrepresentable — you can never be `loading` AND `error` at once — and
// lets views `match(...).exhaustive()` with ts-pattern so a forgotten state is a
// compile error.
export type Async<T, E = string> =
  | { readonly status: "idle" }
  | { readonly status: "loading" }
  | { readonly status: "success"; readonly data: T }
  | { readonly status: "error"; readonly error: E };

// `never` on both parameters so the constants fit ANY `Async<T, E>`, including
// unions with a typed (non-string) error.
export const idle: Async<never, never> = { status: "idle" };
export const loading: Async<never, never> = { status: "loading" };
export const success = <T>(data: T): Async<T, never> => ({ status: "success", data });
export const failure = <E = string>(error: E): Async<never, E> => ({ status: "error", error });

/**
 * Fold a promise into an `Async` ref: `loading` while it runs, then `success` or
 * `error`. The error is a **user-facing** message via `humanError` (raw gRPC codes
 * are logged, never stored/shown). Never throws — the outcome lives in the state,
 * so callers/views branch on it rather than try/catch. Returns the settled state.
 */
export async function run<T>(ref: Ref<Async<T>>, fn: () => Promise<T>): Promise<Async<T>> {
  ref.value = loading;
  try {
    ref.value = success(await fn());
  } catch (e) {
    ref.value = failure(humanError(e));
  }
  return ref.value;
}

/**
 * Re-read into an `Async` ref **without regressing to `loading`**: whatever is on screen
 * stays until the new value lands. For a re-read that follows an action the user just
 * took, `run` is the wrong fold — the momentary `loading` unmounts everything the view
 * renders from the data, so the document collapses, the browser clamps the scroll back
 * to the top, and child components remount (re-issuing their own fetches). An INITIAL
 * load still uses `run`: there is nothing to keep, and `loading` is then the honest
 * state. A failure still lands in the union, replacing the stale value.
 */
export async function reread<T>(ref: Ref<Async<T>>, fn: () => Promise<T>): Promise<Async<T>> {
  try {
    ref.value = success(await fn());
  } catch (e) {
    ref.value = failure(humanError(e));
  }
  return ref.value;
}
