import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";

/** One active session (a refresh-token family) — id, the app it was minted for, and
 * when it was created (unix seconds). */
export interface SessionRow {
  id: string;
  audience: string;
  createdAt: number;
}

// Self-service + admin session management (change: add-session-management). Every
// call goes through the authenticated gRPC `AuthService`; the server scopes self ops
// to the caller and gates the admin revoke with `require_admin`. The list and the last
// action outcome are `Async` unions so views match on them — a denied action lands in
// `op` as `{ status: "error" }`, never a throw.
export const useSessionsStore = defineStore("sessions", () => {
  const list = ref<Async<SessionRow[]>>(idle);
  const op = ref<Async<void>>(idle);

  /** Load the caller's active sessions. */
  async function load() {
    await run(list, async () => {
      const resp = await api().auth.listSessions({});
      return resp.sessions.map((s) => ({ id: s.id, audience: s.audience, createdAt: Number(s.createdAt) }));
    });
  }

  /** Revoke one of the caller's own sessions by id, then re-list. */
  async function revoke(sessionId: string) {
    const outcome = await run(op, async () => {
      await api().auth.revokeSession({ sessionId });
    });
    if (outcome.status === "success") await load();
    return outcome;
  }

  /** Revoke ALL of the caller's sessions (sign out everywhere). The caller then also
   * clears its own cookie via the auth store's sign-out. */
  async function revokeAll() {
    return run(op, async () => {
      await api().auth.revokeAllSessions({});
    });
  }

  /** Admin: cut off every session of a target account (RolesView action). */
  async function revokeAccount(userId: string) {
    return run(op, async () => {
      await api().auth.revokeAccountSessions({ userId });
    });
  }

  return { list, op, load, revoke, revokeAll, revokeAccount };
});
