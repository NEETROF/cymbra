// A dependency-free SVG bar chart (the extension ships no charting library). Pure:
// takes numbers, returns an SVG element, so it renders the same in the page and in tests
// (built via the DOM, not a string sink — see stats/view.ts's rendering discipline).

const SVG_NS = "http://www.w3.org/2000/svg";
const WIDTH = 320;
const HEIGHT = 56;
const GAP = 1;

function svgEl<K extends keyof SVGElementTagNameMap>(tag: K): SVGElementTagNameMap[K] {
  return document.createElementNS(SVG_NS, tag);
}

/**
 * A bar chart of `values` as an SVG element sized to the panel, each bar scaled to the
 * series max (a flat/empty series draws a baseline). `fill` is a CSS colour (a token
 * `var(--…)`); `label` is the accessible name.
 */
export function barChartElement(values: number[], fill: string, label: string): SVGSVGElement {
  const n = Math.max(values.length, 1);
  const max = Math.max(1, ...values);
  const slot = WIDTH / n;
  const barW = Math.max(1, slot - GAP);

  const svg = svgEl("svg");
  svg.setAttribute("viewBox", `0 0 ${WIDTH} ${HEIGHT}`);
  svg.setAttribute("width", "100%");
  svg.setAttribute("height", String(HEIGHT));
  svg.setAttribute("preserveAspectRatio", "none");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", label);

  const baseline = svgEl("line");
  baseline.setAttribute("x1", "0");
  baseline.setAttribute("y1", String(HEIGHT - 0.5));
  baseline.setAttribute("x2", String(WIDTH));
  baseline.setAttribute("y2", String(HEIGHT - 0.5));
  baseline.setAttribute("stroke", "var(--cymbra-lingua-border)");
  baseline.setAttribute("stroke-width", "1");
  svg.append(baseline);

  values.forEach((v, i) => {
    const h = v > 0 ? Math.max(1, (v / max) * (HEIGHT - 2)) : 0;
    if (h <= 0) return;
    const x = (i * slot).toFixed(2);
    const y = (HEIGHT - h).toFixed(2);
    const rect = svgEl("rect");
    rect.setAttribute("x", x);
    rect.setAttribute("y", y);
    rect.setAttribute("width", barW.toFixed(2));
    rect.setAttribute("height", h.toFixed(2));
    rect.setAttribute("rx", "1");
    rect.setAttribute("fill", fill);
    svg.append(rect);
  });

  return svg;
}
