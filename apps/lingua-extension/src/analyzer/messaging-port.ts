import type { CardOp, LinguaPort, NewCard, Rating, ReviewCard, StatusChangeIn, StatusOp } from "./port.ts";
import type { LemmaStatus, PageAnalysis } from "./types.ts";
import { sendRpc } from "./rpc.ts";

// The Firefox AnalyzerPort implementation: a thin LinguaPort that forwards every call
// to the WASM engine hosted in the event page (background.ts), over runtime messaging.
// The reading code and side panel consume the same LinguaPort seam, unaware of where
// the engine runs. `send` is injected so the RPC transport can be faked in tests.

export type RpcSend = (method: string, args: unknown[]) => Promise<unknown>;

export class MessagingLinguaPort implements LinguaPort {
  constructor(private readonly send: RpcSend = sendRpc) {}

  private rpc<T>(method: string, args: unknown[] = []): Promise<T> {
    return this.send(method, args) as Promise<T>;
  }

  analyse(blocks: string[]): Promise<PageAnalysis> {
    return this.rpc("analyse", [blocks]);
  }
  setCalibration(threshold: number): Promise<void> {
    return this.rpc("setCalibration", [threshold]);
  }
  calibration(): Promise<number> {
    return this.rpc("calibration");
  }
  setStatus(lemma: string, status: Parameters<LinguaPort["setStatus"]>[1]): Promise<void> {
    return this.rpc("setStatus", [lemma, status]);
  }
  gloss(lemma: string): Promise<string | undefined> {
    return this.rpc("gloss", [lemma]);
  }
  trackedCount(): Promise<number> {
    return this.rpc("trackedCount");
  }
  addCard(card: NewCard): Promise<void> {
    return this.rpc("addCard", [card]);
  }
  deckCount(): Promise<number> {
    return this.rpc("deckCount");
  }
  dueCount(now: number): Promise<number> {
    return this.rpc("dueCount", [now]);
  }
  startReview(now: number): Promise<number> {
    return this.rpc("startReview", [now]);
  }
  reviewCurrent(): Promise<ReviewCard | null> {
    return this.rpc("reviewCurrent");
  }
  reviewReveal(): Promise<void> {
    return this.rpc("reviewReveal");
  }
  reviewGrade(rating: Rating, now: number): Promise<void> {
    return this.rpc("reviewGrade", [rating, now]);
  }
  reviewMarkKnown(now: number): Promise<void> {
    return this.rpc("reviewMarkKnown", [now]);
  }
  backup(): Promise<string> {
    return this.rpc("backup");
  }
  restore(json: string): Promise<void> {
    return this.rpc("restore", [json]);
  }
  reset(): Promise<void> {
    return this.rpc("reset");
  }
  notice(): Promise<string> {
    return this.rpc("notice");
  }
  licences(): Promise<string[]> {
    return this.rpc("licences");
  }
  setStatusAt(lemma: string, status: LemmaStatus | null, atMs: number): Promise<void> {
    return this.rpc("setStatusAt", [lemma, status, atMs]);
  }
  exportStatusOps(): Promise<StatusOp[]> {
    return this.rpc("exportStatusOps");
  }
  applyStatusChanges(changes: StatusChangeIn[]): Promise<number> {
    return this.rpc("applyStatusChanges", [changes]);
  }
  exportCardOps(): Promise<CardOp[]> {
    return this.rpc("exportCardOps");
  }
  applyCardOps(ops: CardOp[]): Promise<number> {
    return this.rpc("applyCardOps", [ops]);
  }
}
