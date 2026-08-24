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
