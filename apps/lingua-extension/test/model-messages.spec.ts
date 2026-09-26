import { describe, expect, it, vi } from "vitest";
import { askModel, asModelStatus, isModelMessage, MODEL_MESSAGE, NOT_OFFERED } from "@/translate/model-messages.ts";

describe("model messages", () => {
  it("recognises the setting's commands only", () => {
    for (const op of ["status", "enable", "disable", "resume"]) {
      expect(isModelMessage({ type: MODEL_MESSAGE, op })).toBe(true);
    }
    expect(isModelMessage({ type: MODEL_MESSAGE, op: "download-everything" })).toBe(false);
    expect(isModelMessage({ type: "lingua-translate", op: "status" })).toBe(false);
    expect(isModelMessage(null)).toBe(false);
  });

  it("reads a reply defensively: anything unreadable offers nothing", () => {
    expect(asModelStatus({ offered: true, host: "local", state: { phase: "ready" } })).toEqual({
      offered: true,
      host: "local",
      state: { phase: "ready" },
    });
    expect(asModelStatus({ offered: true, host: "remote", state: { phase: "nope" } })).toEqual({
      offered: true,
      host: "none",
      state: { phase: "absent" },
    });
    expect(asModelStatus(undefined)).toEqual(NOT_OFFERED);
    expect(asModelStatus({ host: "local" })).toEqual(NOT_OFFERED);
  });

  it("asks the background, and treats a background that cannot answer as offering nothing", async () => {
    const send = vi.fn(async () => ({ offered: true, host: "none", state: { phase: "absent" } }));
    await expect(askModel("enable", send)).resolves.toMatchObject({ offered: true });
    expect(send).toHaveBeenCalledWith({ type: MODEL_MESSAGE, op: "enable" });

    const gone = vi.fn(async () => Promise.reject(new Error("Receiving end does not exist.")));
    await expect(askModel("status", gone)).resolves.toEqual(NOT_OFFERED);
  });
});
