// The demo video's pointer (add-site-lingua-showcase D3): Playwright's recorded video shows no
// mouse, so a page opened with `?pointer` draws one that follows it. Without the parameter, nothing.
(() => {
  if (!new URLSearchParams(location.search).has("pointer")) return;
  const pointer = document.createElement("div");
  pointer.id = "capture-pointer";
  pointer.setAttribute("aria-hidden", "true");
  pointer.style.transform = "translate(-40px, -40px)";
  document.body.append(pointer);
  addEventListener(
    "mousemove",
    (e) => {
      // Last in the document, so that it stays above the card and the panel the reader adds.
      if (document.documentElement.lastElementChild !== pointer) document.documentElement.append(pointer);
      pointer.style.transform = `translate(${e.clientX}px, ${e.clientY}px)`;
    },
    { capture: true, passive: true },
  );
})();
