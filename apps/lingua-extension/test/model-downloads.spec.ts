import { describe, expect, it, vi } from "vitest";
import {
  type DownloadEvent,
  DownloadHost,
  type DownloadWorkerLike,
  type ModelWorkerEvent,
  type ModelWorkerRequest,
} from "@/translate/host/downloads.ts";

// The download's owner in the engine's host (add-lingua-translation-delivery D4): one download at
// a time, in a worker that lives only while it runs.

class FakeWorker implements DownloadWorkerLike {
  readonly sent: ModelWorkerRequest[] = [];
  terminated = false;
  onmessage: ((event: { data: ModelWorkerEvent }) => void) | null = null;
  onerror: ((event: unknown) => void) | null = null;
  postMessage(message: ModelWorkerRequest): void {
    this.sent.push(message);
  }
  terminate(): void {
    this.terminated = true;
  }
  say(data: ModelWorkerEvent): void {
    this.onmessage?.({ data });
  }
}

function setup() {
  const workers: FakeWorker[] = [];
  const events: DownloadEvent[] = [];
  const ended = vi.fn();
  const log = vi.fn();
  const host = new DownloadHost(
    () => {
      const w = new FakeWorker();
      workers.push(w);
      return w;
    },
    (e) => events.push(e),
    { onEnded: ended, log },
  );
  return { host, workers, events, ended, log };
}

describe("DownloadHost", () => {
  it("starts one worker and tells it to download", () => {
    const { host, workers } = setup();
    host.start();
    host.start(); // already running: nothing more
    expect(workers).toHaveLength(1);
    expect(workers[0]!.sent).toEqual([{ op: "download" }]);
    expect(host.running()).toBe(true);
  });

  it("relays progress, then completion — and puts the worker down", () => {
    const { host, workers, events, ended } = setup();
    host.start();
    workers[0]!.say({ kind: "progress", received: 5, total: 10 });
    workers[0]!.say({ kind: "done" });
    expect(events).toEqual([{ kind: "progress", received: 5, total: 10 }, { kind: "done" }]);
    expect(workers[0]!.terminated).toBe(true);
    expect(host.running()).toBe(false);
    expect(ended).toHaveBeenCalledOnce();
  });

  it("reports a failure by its reason alone, and logs the detail", () => {
    const { host, workers, events, log } = setup();
    host.start();
    workers[0]!.say({ kind: "failed", reason: "not-the-model", detail: "lex: sha256 differs" });
    expect(events).toEqual([{ kind: "failed", reason: "not-the-model" }]);
    expect(log).toHaveBeenCalledWith(expect.stringContaining("not-the-model"), "lex: sha256 differs");
  });

  it("reports nothing for a download it was told to cancel", () => {
    const { host, workers, events, ended } = setup();
    host.start();
    host.cancel();
    expect(workers[0]!.terminated).toBe(true);
    workers[0]!.say({ kind: "done" }); // a message already on its way
    expect(events).toEqual([]);
    expect(ended).toHaveBeenCalledOnce();
    host.cancel(); // nothing to cancel any more
    expect(ended).toHaveBeenCalledOnce();
  });

  it("reports a worker that says it was cancelled as nothing at all", () => {
    const { host, workers, events } = setup();
    host.start();
    workers[0]!.say({ kind: "failed", reason: "cancelled", detail: "cancelled" });
    expect(events).toEqual([]);
    expect(host.running()).toBe(false);
  });

  it("reports a crashed worker as a failure", () => {
    const { host, workers, events, ended } = setup();
    host.start();
    workers[0]!.onerror?.(new Event("error"));
    expect(events).toEqual([{ kind: "failed", reason: "unknown" }]);
    expect(ended).toHaveBeenCalledOnce();
    workers[0]!.onerror?.(new Event("error")); // late, from a worker already put down
    expect(events).toHaveLength(1);
  });

  it("reports a worker that cannot be made as a failure", () => {
    const events: DownloadEvent[] = [];
    const host = new DownloadHost(
      () => {
        throw new Error("no Worker here");
      },
      (e) => events.push(e),
      { log: vi.fn() },
    );
    host.start();
    expect(events).toEqual([{ kind: "failed", reason: "unknown" }]);
    expect(host.running()).toBe(false);
  });

  it("starts a fresh worker for the next download", () => {
    const { host, workers } = setup();
    host.start();
    workers[0]!.say({ kind: "failed", reason: "network", detail: "offline" });
    host.start();
    expect(workers).toHaveLength(2);
  });
});
