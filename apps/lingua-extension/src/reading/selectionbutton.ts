// A small floating "+ Deck" button shown next to a multi-word mouse selection (the
// Medium/Genius pattern), so a phrase can be captured without the Alt+L shortcut. Like
// the word popup it lives in a CLOSED shadow root, isolated from the page's CSS and JS,
// and its host carries the skip marker so the reader never analyses it.

export class SelectionButton {
  /** The host element in the page (excluded from scanning by its id / skip attr). */
  readonly host: HTMLElement;
  private readonly btn: HTMLButtonElement;

  constructor(css: string, onPick: () => void) {
    this.host = document.createElement("div");
    this.host.id = "cymbra-lingua-selbtn";
    this.host.setAttribute("data-cymbra-lingua-skip", "");
    const root = this.host.attachShadow({ mode: "closed" });
    const style = document.createElement("style");
    style.textContent = css;

    this.btn = document.createElement("button");
    this.btn.className = "selbtn";
    this.btn.textContent = "+ Deck";
    this.btn.hidden = true;
    // Keep the page selection alive: a plain click would collapse it before `onPick`
    // reads it, so suppress the default mousedown selection-clear.
    this.btn.addEventListener("mousedown", (e) => e.preventDefault());
    this.btn.addEventListener("click", (e) => {
      e.stopPropagation();
      onPick();
    });
    root.append(style, this.btn);
  }

  /** True when an event target is inside this button (so callers can ignore self-hits). */
  contains(target: EventTarget | null): boolean {
    return target === this.host || (target instanceof Node && this.host.contains(target));
  }

  visible(): boolean {
    return !this.btn.hidden;
  }

  /** Show the button anchored just below the selection's end. */
  show(rect: { left: number; bottom: number }): void {
    if (!this.host.isConnected) document.documentElement.appendChild(this.host);
    this.btn.style.left = `${Math.max(8, rect.left)}px`;
    this.btn.style.top = `${rect.bottom + 6}px`;
    this.btn.hidden = false;
  }

  hide(): void {
    this.btn.hidden = true;
  }
}
