import { beforeEach, describe, expect, it } from "vitest";
import { SelectionButton } from "@/reading/selectionbutton.ts";

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("selection button", () => {
  it("shows on demand, hides, and recognises its own host", () => {
    const sb = new SelectionButton("", () => {});
    expect(sb.visible()).toBe(false);

    sb.show({ left: 40, bottom: 80 });
    expect(sb.visible()).toBe(true);
    expect(sb.host.isConnected).toBe(true);
    expect(sb.contains(sb.host)).toBe(true);
    expect(sb.contains(document.body)).toBe(false);

    sb.hide();
    expect(sb.visible()).toBe(false);
  });
});
