// A dependency-free SVG bar chart (the extension ships no charting library). Pure:
// takes numbers, returns an SVG string, so it renders the same in the page and in tests.

const WIDTH = 320;
const HEIGHT = 56;
const GAP = 1;

/**
 * A bar chart of `values` as an SVG string sized to the panel, each bar scaled to the
 * series max (a flat/empty series draws a baseline). `fill` is a CSS colour (a token
 * `var(--…)`); `label` is the accessible name.
 */
export function barChartSvg(values: number[], fill: string, label: string): string {
  const n = Math.max(values.length, 1);
  const max = Math.max(1, ...values);
  const slot = WIDTH / n;
  const barW = Math.max(1, slot - GAP);
  const bars = values
    .map((v, i) => {
      const h = v > 0 ? Math.max(1, (v / max) * (HEIGHT - 2)) : 0;
      const x = (i * slot).toFixed(2);
      const y = (HEIGHT - h).toFixed(2);
      return h > 0
        ? `<rect x="${x}" y="${y}" width="${barW.toFixed(2)}" height="${h.toFixed(2)}" rx="1" fill="${fill}"/>`
        : "";
    })
    .join("");
  return (
    `<svg viewBox="0 0 ${WIDTH} ${HEIGHT}" width="100%" height="${HEIGHT}" ` +
    `preserveAspectRatio="none" role="img" aria-label="${label}">` +
    `<line x1="0" y1="${HEIGHT - 0.5}" x2="${WIDTH}" y2="${HEIGHT - 0.5}" stroke="var(--cymbra-lingua-border)" stroke-width="1"/>` +
    bars +
    `</svg>`
  );
}
