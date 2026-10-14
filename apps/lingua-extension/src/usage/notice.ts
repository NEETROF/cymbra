import type { NoticeKind } from "./counts.ts";
import type { NoticeControls } from "./messages.ts";

// The anonymous counts' first-run sentence (change: add-lingua-usage-counts, D7), one builder for
// its two hosts: the welcome tab and the toolbar popup. Never a notification, a badge or anything
// over a page being read. On a build that informs (`on`), showing it is what the background is told;
// on a build that asks (Chromium), it carries two equal buttons and says nothing more until one is
// pressed. Its copy is the host's (`onboarding.ts` / `popup.ts` of the catalogue).

export interface NoticeCopy {
  /** The sentence on a build that informs: what is counted, nothing read, where to turn it off. */
  inform: string;
  /** The same, as the question of a build that asks. */
  ask: string;
  /** « Oui, compter » — the gate opens. */
  yes: string;
  /** « Non merci » — the switch turned off. */
  no: string;
  /** Said after « Oui, compter ». */
  yesDone: string;
  /** Said after « Non merci ». */
  noDone: string;
  /** The privacy policy's link text. */
  privacy: string;
}

/** The keys a host's catalogue module carries for the sentence (`onboarding`, `popup`). */
export interface NoticeCopyKeys {
  usageInform: string;
  usageAsk: string;
  usageYes: string;
  usageNo: string;
  usageYesDone: string;
  usageNoDone: string;
  usagePrivacy: string;
}

/** The sentence's copy out of a host's catalogue module. */
export function noticeCopyOf(m: NoticeCopyKeys): NoticeCopy {
  return {
    inform: m.usageInform,
    ask: m.usageAsk,
    yes: m.usageYes,
    no: m.usageNo,
    yesDone: m.usageYesDone,
    noDone: m.usageNoDone,
    privacy: m.usagePrivacy,
  };
}

export interface NoticeOptions {
  copy: NoticeCopy;
  controls: NoticeControls;
  /** The privacy policy in the interface language. */
  privacyUrl: string;
}

function el<K extends keyof HTMLElementTagNameMap>(
  doc: Document,
  tag: K,
  className: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = doc.createElement(tag);
  node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/**
 * Fill `container` with the sentence the background says is due, if any, and return what it was.
 * The container stays hidden when nothing is due.
 */
export async function mountUsageNotice(container: HTMLElement, opts: NoticeOptions): Promise<NoticeKind> {
  const { copy, controls } = opts;
  const doc = container.ownerDocument;
  const kind = await controls.status();
  container.replaceChildren();
  container.hidden = kind === "none";
  if (kind === "none") return kind;

  const text = el(doc, "p", "usage-notice-text", kind === "ask" ? copy.ask : copy.inform);
  const privacy = el(doc, "a", "usage-notice-privacy", copy.privacy);
  privacy.href = opts.privacyUrl;
  privacy.target = "_blank";
  privacy.rel = "noopener";
  container.append(text, privacy);

  if (kind === "inform") {
    await controls.shown();
    return kind;
  }

  // Two buttons of the same size and weight: neither is the default.
  const row = el(doc, "div", "usage-notice-answers");
  const yes = el(doc, "button", "usage-notice-answer", copy.yes);
  const no = el(doc, "button", "usage-notice-answer", copy.no);
  for (const b of [yes, no]) b.type = "button";
  const done = el(doc, "p", "usage-notice-done");
  done.hidden = true;
  row.append(yes, no);
  container.append(row, done);
  const answer = async (agreed: boolean): Promise<void> => {
    yes.disabled = true;
    no.disabled = true;
    await controls.answer(agreed);
    row.hidden = true;
    done.textContent = agreed ? copy.yesDone : copy.noDone;
    done.hidden = false;
  };
  yes.addEventListener("click", () => void answer(true));
  no.addEventListener("click", () => void answer(false));
  return kind;
}
