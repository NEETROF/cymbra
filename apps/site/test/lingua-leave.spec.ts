import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import LeaveReasonIsland from "../src/components/LeaveReasonIsland.vue";
import { leaveView, useLeaveReason } from "../src/composables/useLeaveReason";
import { LANGS, t } from "../src/lib/i18n";
import {
  createLinguaLeaveClient,
  LEAVE_REASONS,
  leaveContextFromQuery,
  setLinguaLeaveClientForTest,
  SUPPORT_HREF,
  type LeaveAnswer,
  type LinguaLeaveClient,
} from "../src/lib/lingua-leave";

// The Lingua leaving page (change: add-lingua-usage-counts, D12; spec `site-lingua-page`).

function fakeClient(leave: LinguaLeaveClient["leave"] = async () => undefined) {
  const fake = { leave: vi.fn(leave) };
  setLinguaLeaveClientForTest(fake);
  return fake;
}

/** The six answer buttons, in page order. */
const answers = (w: ReturnType<typeof mount>) => w.findAll(".leave-answers button");

beforeEach(() => {
  window.history.replaceState({}, "", "/lingua/au-revoir/");
});

afterEach(() => {
  setLinguaLeaveClientForTest(null);
  vi.unstubAllGlobals();
});

describe("the page's address", () => {
  it("reads the browser family and the major.minor version the extension appends", () => {
    expect(leaveContextFromQuery("?b=firefox&v=1.10")).toEqual({ browser: "firefox", version: "1.10" });
    expect(leaveContextFromQuery("b=chromium&v=2.0")).toEqual({ browser: "chromium", version: "2.0" });
    expect(leaveContextFromQuery("?b=safari&v=123.456")).toEqual({ browser: "safari", version: "123.456" });
  });

  it("reads anything else as unknown", () => {
    expect(leaveContextFromQuery("")).toEqual({ browser: "unknown", version: "unknown" });
    expect(leaveContextFromQuery("?b=edge&v=1.10.2")).toEqual({ browser: "unknown", version: "unknown" });
    expect(leaveContextFromQuery("?b=Firefox&v=v1.1")).toEqual({ browser: "unknown", version: "unknown" });
    expect(leaveContextFromQuery("?b=firefox&v=1234.1")).toEqual({ browser: "firefox", version: "unknown" });
    expect(leaveContextFromQuery("?v=1.9")).toEqual({ browser: "unknown", version: "1.9" });
    expect(leaveContextFromQuery("?b=firefox&v=1.9%0A")).toEqual({ browser: "firefox", version: "unknown" });
  });

  it("names the six answers in the page's order, with the wire's values", () => {
    expect(LEAVE_REASONS).toEqual(["not_using", "highlighting", "missing_language", "slow", "unclear", "other"]);
  });
});

describe("the client", () => {
  it("posts the answer as JSON, with no credentials and no Authorization header", async () => {
    const fetch = vi.fn(async () => new Response("{}", { status: 200 }));
    vi.stubGlobal("fetch", fetch);
    const answer: LeaveAnswer = { reason: "slow", browser: "firefox", version: "1.10" };
    await createLinguaLeaveClient("https://api.example").leave(answer);
    expect(fetch).toHaveBeenCalledTimes(1);
    const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("https://api.example/web/lingua/leave");
    expect(init.method).toBe("POST");
    expect(init.credentials).toBe("omit");
    expect(init.headers).toEqual({ "Content-Type": "application/json" });
    expect(JSON.parse(init.body as string)).toEqual({ reason: "slow", browser: "firefox", version: "1.10" });
  });

  it("rejects a refused answer (400, 429), which the page then thanks all the same", async () => {
    for (const status of [400, 429]) {
      vi.stubGlobal("fetch", vi.fn(async () => new Response('{"error":"x"}', { status })));
      await expect(createLinguaLeaveClient("https://api.example").leave({ reason: "other", browser: "unknown", version: "unknown" })).rejects.toThrow();
    }
  });
});

describe("useLeaveReason", () => {
  it("sends nothing until an answer is chosen, then that one answer, once", async () => {
    const client = fakeClient();
    const leave = useLeaveReason();
    leave.boot("?b=chromium&v=1.9");
    expect(client.leave).not.toHaveBeenCalled();
    expect(leave.view.value.answered).toBe(false);

    await leave.choose("missing_language");
    expect(client.leave).toHaveBeenCalledTimes(1);
    expect(client.leave).toHaveBeenCalledWith({ reason: "missing_language", browser: "chromium", version: "1.9" });
    expect(leave.answer.value.status).toBe("success");
    expect(leave.chosen.value).toBe("missing_language");

    await leave.choose("other");
    expect(client.leave).toHaveBeenCalledTimes(1);
    expect(leave.chosen.value).toBe("missing_language");
  });

  it("keeps a failure in the state, thanked like a success", async () => {
    fakeClient(async () => {
      throw new Error("network down");
    });
    const leave = useLeaveReason();
    await leave.choose("slow");
    expect(leave.answer.value.status).toBe("error");
    expect(leave.view.value.answered).toBe(true);
  });

  it("sends unknown when the page was never given its parameters", async () => {
    const client = fakeClient();
    const leave = useLeaveReason();
    await leave.choose("unclear");
    expect(client.leave).toHaveBeenCalledWith({ reason: "unclear", browser: "unknown", version: "unknown" });
  });

  it("thanks from the tap on, while the answer is still on its way", () => {
    expect(leaveView({ status: "idle" })).toEqual({ answered: false });
    expect(leaveView({ status: "loading" })).toEqual({ answered: true });
    expect(leaveView({ status: "success", data: undefined })).toEqual({ answered: true });
    expect(leaveView({ status: "error", error: "x" })).toEqual({ answered: true });
  });
});

describe("LeaveReasonIsland", () => {
  it("shows the six answers in French and sends nothing on load", async () => {
    window.history.replaceState({}, "", "/lingua/au-revoir/?b=firefox&v=1.10");
    const client = fakeClient();
    const fetch = vi.fn();
    vi.stubGlobal("fetch", fetch);
    const w = mount(LeaveReasonIsland, { props: { lang: "fr", communityUrl: null } });
    await flushPromises();

    expect(answers(w).map((b) => b.text())).toEqual([
      "Je ne m'en servais pas",
      "Trop de mots surlignés, ou pas assez",
      "Ma langue n'y est pas",
      "Ça ralentissait mon navigateur",
      "Je n'ai pas compris comment m'en servir",
      "Une autre raison",
    ]);
    expect(answers(w).every((b) => b.attributes("disabled") === undefined)).toBe(true);
    expect(w.find("textarea").exists()).toBe(false);
    expect(w.find("input").exists()).toBe(false);
    expect(w.find('[data-testid="leave-thanks"]').exists()).toBe(false);
    expect(client.leave).not.toHaveBeenCalled();
    expect(fetch).not.toHaveBeenCalled();
  });

  it("sends the tapped answer once, disables the six and thanks the reader", async () => {
    window.history.replaceState({}, "", "/lingua/au-revoir/?b=firefox&v=1.10");
    const client = fakeClient();
    const w = mount(LeaveReasonIsland, { props: { lang: "fr", communityUrl: "https://discord.example/invite" } });
    await flushPromises();

    await w.find('[data-reason="slow"]').trigger("click");
    await flushPromises();
    expect(client.leave).toHaveBeenCalledTimes(1);
    expect(client.leave).toHaveBeenCalledWith({ reason: "slow", browser: "firefox", version: "1.10" });
    expect(answers(w).every((b) => b.attributes("disabled") !== undefined)).toBe(true);
    expect(w.find('[data-reason="slow"]').attributes("aria-pressed")).toBe("true");
    expect(w.find('[data-reason="other"]').attributes("aria-pressed")).toBe("false");

    const thanks = w.find('[data-testid="leave-thanks"]');
    expect(thanks.text()).toContain(t("fr", "leaveThanksTitle"));
    expect(thanks.find(`a[href="${SUPPORT_HREF.fr}"]`).text()).toBe(t("fr", "leaveSupport"));
    expect(thanks.find('a[href="https://discord.example/invite"]').text()).toBe(t("fr", "leaveCommunity"));

    // A second tap, even on a button re-enabled by hand, sends nothing more.
    const other = w.find<HTMLButtonElement>('[data-reason="other"]');
    other.element.disabled = false;
    await other.trigger("click");
    await flushPromises();
    expect(client.leave).toHaveBeenCalledTimes(1);
  });

  it("thanks the reader when the backend is unreachable, showing no technical error", async () => {
    window.history.replaceState({}, "", "/en/lingua/goodbye/");
    fakeClient(async () => {
      throw new TypeError("Failed to fetch");
    });
    const w = mount(LeaveReasonIsland, { props: { lang: "en", communityUrl: null } });
    await flushPromises();
    await w.find('[data-reason="not_using"]').trigger("click");
    await flushPromises();

    expect(w.find('[data-testid="leave-thanks"]').text()).toContain(t("en", "leaveThanksTitle"));
    expect(w.text()).not.toContain("Failed to fetch");
    expect(w.find('[role="alert"]').exists()).toBe(false);
    // No invite configured: no community link, the help page all the same.
    expect(w.find(".btn-discord").exists()).toBe(false);
    expect(w.find(`a[href="${SUPPORT_HREF.en}"]`).exists()).toBe(true);
  });

  it("sends unknown for a page opened by hand, or with parameters the extension never sends", async () => {
    for (const search of ["", "?b=opera&v=latest", "?utm_source=x"]) {
      window.history.replaceState({}, "", `/en/lingua/goodbye/${search}`);
      const client = fakeClient();
      const w = mount(LeaveReasonIsland, { props: { lang: "en", communityUrl: null } });
      await flushPromises();
      await w.find('[data-reason="highlighting"]').trigger("click");
      await flushPromises();
      expect(client.leave).toHaveBeenCalledWith({ reason: "highlighting", browser: "unknown", version: "unknown" });
    }
  });

  it("speaks each site language, six answers each", async () => {
    fakeClient();
    for (const lang of LANGS) {
      const w = mount(LeaveReasonIsland, { props: { lang, communityUrl: null } });
      await flushPromises();
      const labels = answers(w).map((b) => b.text());
      expect(labels).toHaveLength(6);
      expect(new Set(labels).size).toBe(6);
      expect(w.find(".leave-answers").attributes("aria-label")).toBe(t(lang, "leaveQuestion"));
    }
  });
});
