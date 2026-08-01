import { afterEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { i18n } from "@/i18n";
import IdBadge from "@/components/IdBadge.vue";

// A UUID v7: the leading hex ("019f7092") is the crawl timestamp, shared across
// rows; the trailing hex is what actually distinguishes them.
const UUID = "019f7092-8c3a-7b21-9f4e-a1b2c3d4e5f6";

function mountBadge(id = UUID) {
  return mount(IdBadge, { props: { id }, global: { plugins: [i18n] } });
}

afterEach(() => {
  vi.restoreAllMocks();
  document.body.innerHTML = "";
});

describe("IdBadge", () => {
  it("shows the trailing hex, not the shared timestamp prefix", () => {
    const w = mountBadge();
    // last 8 hex of the id, uppercased — NOT "019F7092".
    expect(w.get(".trigger").text()).toBe("ID: C3D4E5F6");
    expect(w.get(".trigger").text()).not.toContain("019F7092");
  });

  it("reveals the full id in a popover on click and hides it again", async () => {
    const w = mountBadge();
    expect(w.find(".pop").exists()).toBe(false);

    await w.get(".trigger").trigger("click");
    expect(w.get(".pop").text()).toContain(UUID);

    // Escape closes it.
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    await flushPromises();
    expect(w.find(".pop").exists()).toBe(false);
  });

  it("copies the full id to the clipboard and confirms", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });

    const w = mountBadge();
    await w.get(".trigger").trigger("click");
    await w.get(".copy").trigger("click");
    await flushPromises();

    expect(writeText).toHaveBeenCalledWith(UUID);
    expect(w.get(".copy").classes()).toContain("done");
    expect(w.get(".copy").text()).toBe(i18n.global.t("id.copied"));
  });

  it("stays open (no crash) when the clipboard write is blocked", async () => {
    const writeText = vi.fn().mockRejectedValue(new Error("blocked"));
    vi.stubGlobal("navigator", { clipboard: { writeText } });

    const w = mountBadge();
    await w.get(".trigger").trigger("click");
    await w.get(".copy").trigger("click");
    await flushPromises();

    // The rejection is swallowed: the popover is still there, unconfirmed.
    expect(w.get(".pop").text()).toContain(UUID);
    expect(w.get(".copy").classes()).not.toContain("done");
  });
});
