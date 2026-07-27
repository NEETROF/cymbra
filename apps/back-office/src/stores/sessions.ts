import { ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";

// Admin session revocation (change: add-session-management). The only session action
// the back office exposes: an admin cuts off every session of a target account (a
// compromised moderator/admin). The server gates it with `require_admin` and scopes it
// to the caller's audience; the outcome is an `Async` union so the view surfaces it.
export const useSessionsStore = defineStore("sessions", () => {
  const op = ref<Async<void>>(idle);

  /** Admin: cut off every session of a target account (RolesView action). */
  async function revokeAccount(userId: string) {
    return run(op, async () => {
      await api().auth.revokeAccountSessions({ userId });
    });
  }

  return { op, revokeAccount };
});
