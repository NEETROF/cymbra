import { IDBFactory } from "fake-indexeddb";
import { describe, expect, it } from "vitest";
import { MODEL_DB, modelDb } from "@/translate/host/model-db.ts";
import type { ModelManifest } from "@/translate/host/model-manifest.ts";

// The `lingua-model` database (add-lingua-translation-delivery D5): decompressed bytes keyed by
// their sha256, a `complete` record per model version, and a whole-database deletion.

const sha = (c: string) => c.repeat(64);
const manifest: ModelManifest = {
  version: "en-fr/base-memory/2.0",
  base: "https://models.example/",
  files: {
    model: { path: "m.gz", size: 3, sha256: sha("a") },
    lex: { path: "l.gz", size: 2, sha256: sha("b") },
    vocab: { path: "v.gz", size: 1, sha256: sha("c") },
  },
};

async function filled(factory: IDBFactory) {
  const db = modelDb(factory);
  await db.put(sha("a"), new Uint8Array([1, 2, 3]));
  await db.put(sha("b"), new Uint8Array([4, 5]));
  await db.put(sha("c"), new Uint8Array([6]));
  await db.markComplete(manifest);
  return db;
}

describe("modelDb", () => {
  it("stores bytes under their sha256 and gives them back", async () => {
    const db = modelDb(new IDBFactory());
    expect(await db.has(sha("a"))).toBe(false);
    expect(await db.get(sha("a"))).toBeNull();
    await db.put(sha("a"), new Uint8Array([1, 2, 3]));
    expect(await db.has(sha("a"))).toBe(true);
    expect([...(await db.get(sha("a")))!]).toEqual([1, 2, 3]);
  });

  it("is complete only with the record AND every file", async () => {
    const factory = new IDBFactory();
    const db = modelDb(factory);
    await db.put(sha("a"), new Uint8Array([1]));
    await db.put(sha("b"), new Uint8Array([2]));
    await db.put(sha("c"), new Uint8Array([3]));
    expect(await db.complete(manifest)).toBe(false); // every file, but no record
    await db.markComplete(manifest);
    expect(await db.complete(manifest)).toBe(true);
  });

  it("is not complete for another model version", async () => {
    const db = await filled(new IDBFactory());
    expect(await db.complete({ ...manifest, version: "en-fr/base-memory/3.0" })).toBe(false);
  });

  it("is not complete once a file has gone, whatever the record says", async () => {
    const factory = new IDBFactory();
    await filled(factory);
    // The browser evicting one object store's entry is modelled by deleting it behind our back.
    await new Promise<void>((resolve, reject) => {
      const open = factory.open(MODEL_DB);
      open.onsuccess = () => {
        const tx = open.result.transaction("files", "readwrite");
        tx.objectStore("files").delete(sha("b"));
        tx.oncomplete = () => {
          open.result.close();
          resolve();
        };
        tx.onerror = () => reject(tx.error);
      };
    });
    expect(await modelDb(factory).complete(manifest)).toBe(false);
  });

  it("deletes the whole database — turning the setting off leaves nothing behind", async () => {
    const factory = new IDBFactory();
    const db = await filled(factory);
    await db.erase();
    expect((await factory.databases()).map((d) => d.name)).not.toContain(MODEL_DB);
    expect(await db.complete(manifest)).toBe(false);
  });

  it("closes each connection, so a deletion is never held off by one of its own", async () => {
    const factory = new IDBFactory();
    const db = await filled(factory);
    await db.get(sha("a"));
    let blocked = false;
    await new Promise<void>((resolve, reject) => {
      const req = factory.deleteDatabase(MODEL_DB);
      req.onblocked = () => (blocked = true);
      req.onsuccess = () => resolve();
      req.onerror = () => reject(req.error);
    });
    expect(blocked).toBe(false);
  });

  it("fails loudly when the database cannot be opened", async () => {
    const broken = {
      open: () => {
        const req = {} as IDBOpenDBRequest & { error: Error };
        queueMicrotask(() => {
          Object.assign(req, { error: new Error("denied") });
          req.onerror?.(new Event("error"));
        });
        return req;
      },
    } as unknown as IDBFactory;
    await expect(modelDb(broken).has(sha("a"))).rejects.toThrow("denied");
  });
});
