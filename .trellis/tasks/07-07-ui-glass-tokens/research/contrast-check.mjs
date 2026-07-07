// 07-07-ui-glass-tokens: WCAG 对比度 + OKLCH L 验证脚本
// 运行: bun contrast-check.mjs (或 node contrast-check.mjs)

const hex2rgb = (hex) => {
  const h = hex.replace("#", "");
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
};

// WCAG 相对亮度
const srgbLin = (c) => {
  const v = c / 255;
  return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
};
const relLum = (hex) => {
  const [r, g, b] = hex2rgb(hex);
  return 0.2126 * srgbLin(r) + 0.7152 * srgbLin(g) + 0.0722 * srgbLin(b);
};
const contrast = (fg, bg) => {
  const l1 = relLum(fg);
  const l2 = relLum(bg);
  const [hi, lo] = l1 >= l2 ? [l1, l2] : [l2, l1];
  return (hi + 0.05) / (lo + 0.05);
};

// OKLab / OKLCH L 通道 (0-100)
const oklchL = (hex) => {
  const [r, g, b] = hex2rgb(hex).map(srgbLin);
  const l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
  const m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
  const s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
  const l_ = Math.cbrt(l);
  const m_ = Math.cbrt(m);
  const s_ = Math.cbrt(s);
  return (0.2104542553 * l_ + 0.793617785 * m_ - 0.0040720468 * s_) * 100;
};

const fmt = (n) => n.toFixed(2);

const flavors = {
  "light clay (new)": {
    layers: {
      base: "#ebe1d0",
      elevated: "#f5eee1",
      surface: "#fefaf2",
      overlay: "#e2d6c3",
    },
    text: { primary: "#31241c", secondary: "#5f4d3f", muted: "#715d4c" },
    darkMode: false,
  },
  "light paper (new)": {
    layers: {
      base: "#e7e7e7",
      elevated: "#f2f2f2",
      surface: "#fdfdfd",
      overlay: "#dbdbdb",
    },
    text: { primary: "#1a1a1c", secondary: "#3f3f46", muted: "#626268" },
    darkMode: false,
  },
  "light graphite (new)": {
    layers: {
      base: "#e4e4e9",
      elevated: "#f0f0f4",
      surface: "#fbfbfd",
      overlay: "#d8d8de",
    },
    text: { primary: "#1f2024", secondary: "#43464c", muted: "#5f636c" },
    darkMode: false,
  },
  "dark clay (kept)": {
    layers: {
      base: "#17120f",
      elevated: "#221b18",
      surface: "#2a221e",
      overlay: "#342b26",
    },
    text: { primary: "#f3eadf", secondary: "#dacbbc", muted: "#b9a695" },
    darkMode: true,
  },
  "dark paper (kept)": {
    layers: {
      base: "#1a1a1c",
      elevated: "#232325",
      surface: "#2a2a2d",
      overlay: "#34343a",
    },
    text: { primary: "#f2f2f4", secondary: "#d4d4d8", muted: "#a1a1a8" },
    darkMode: true,
  },
  "dark graphite (kept)": {
    layers: {
      base: "#14161a",
      elevated: "#1d1f23",
      surface: "#25272c",
      overlay: "#2e3036",
    },
    text: { primary: "#eceef2", secondary: "#c7cad1", muted: "#989ca6" },
    darkMode: true,
  },
};

for (const [name, { layers, text, darkMode }] of Object.entries(flavors)) {
  console.log(`\n=== ${name} ===`);
  const L = Object.fromEntries(
    Object.entries(layers).map(([k, v]) => [k, oklchL(v)]),
  );
  console.log(
    `OKLCH L: base=${fmt(L.base)} elevated=${fmt(L.elevated)} surface=${fmt(L.surface)} overlay=${fmt(L.overlay)}`,
  );
  console.log(
    `ΔL base→elevated=${fmt(Math.abs(L.elevated - L.base))} elevated→surface=${fmt(Math.abs(L.surface - L.elevated))} base↔surface=${fmt(Math.abs(L.surface - L.base))} overlay→base=${fmt(Math.abs(L.base - L.overlay))}`,
  );
  const minAdj = darkMode ? 2.5 : 3;
  const okAdj =
    Math.abs(L.elevated - L.base) >= minAdj &&
    Math.abs(L.surface - L.elevated) >= minAdj;
  const okSpan = darkMode || Math.abs(L.surface - L.base) >= 6;
  console.log(
    `layer check: adjacent ΔL≥${minAdj} ${okAdj ? "PASS" : "FAIL"}; base↔surface ${okSpan ? "PASS" : "FAIL"}`,
  );
  for (const [tName, tHex] of Object.entries(text)) {
    const vsSurface = contrast(tHex, layers.surface);
    const vsBase = contrast(tHex, layers.base);
    const vsElevated = contrast(tHex, layers.elevated);
    const target = tName === "primary" ? 7 : 4.5;
    const pass =
      vsSurface >= target && vsBase >= target
        ? "PASS"
        : vsSurface >= target
          ? "PASS(surface)"
          : "FAIL";
    console.log(
      `${tName} ${tHex}: vs surface=${fmt(vsSurface)} vs elevated=${fmt(vsElevated)} vs base=${fmt(vsBase)} (target ${target}:1) ${pass}`,
    );
  }
}
