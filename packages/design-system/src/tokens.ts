/**
 * Design tokens v1 (first draft, from the Kids Economy design system canvas): warm paper
 * surfaces, one sea-teal brand colour, coin gold for saving, rounded type, big numbers.
 * Values meet WCAG AA for text in light and dark.
 *
 * `theme.css` mirrors these values as Tailwind v4 theme variables. Keep them in sync;
 * the tokens test guards the shape.
 */
export const tokens = {
  color: {
    surface: { light: "#f7f5f0", dark: "#141c1b" },
    "surface-raised": { light: "#ffffff", dark: "#1d2826" },
    "surface-sunken": { light: "#eeebe3", dark: "#0f1615" },
    line: { light: "#ddd8cc", dark: "#34423f" },
    "line-strong": { light: "#8c867a", dark: "#7d8f8b" },
    ink: { light: "#1c2826", dark: "#eef4f2" },
    muted: { light: "#596663", dark: "#a7b6b2" },
    brand: { light: "#0d7064", dark: "#5ad1bf" },
    "brand-soft": { light: "#d8efe9", dark: "#173a34" },
    "on-brand": { light: "#ffffff", dark: "#0a1d1a" },
    coin: { light: "#f0b429", dark: "#f4c44e" },
    "coin-strong": { light: "#a86c00", dark: "#f4c44e" },
    "coin-soft": { light: "#fcefd0", dark: "#3a2f12" },
    "coin-ink": { light: "#7d5300", dark: "#f4c44e" },
    "money-in": { light: "#0b6a5f", dark: "#62d6c4" },
    "money-out": { light: "#b04316", dark: "#ff9f76" },
    "money-out-soft": { light: "#fbe6db", dark: "#3d2216" },
    danger: { light: "#b3261e", dark: "#ff8b80" },
    "danger-soft": { light: "#fbe3e0", dark: "#3d1a17" },
    focus: { light: "#2257d6", dark: "#8fb0ff" },
    "kid-sea": { light: "#bfe6df", dark: "#1f4a44" },
    "kid-sun": { light: "#fbe0a0", dark: "#4a3b12" },
    "kid-berry": { light: "#f6c9d6", dark: "#4d2433" },
    "kid-leaf": { light: "#cfe8b6", dark: "#2c4220" },
    "kid-sky": { light: "#c7dcf7", dark: "#22385a" },
  },
  font: {
    display: 'Fredoka, Nunito, ui-rounded, "SF Pro Rounded", system-ui, sans-serif',
    text: 'Nunito, ui-rounded, "SF Pro Rounded", system-ui, sans-serif',
  },
  radius: { field: "10px", control: "16px", card: "24px" },
  touchTarget: "48px",
} as const satisfies {
  color: Record<string, { light: string; dark: string }>;
  font: Record<string, string>;
  radius: Record<string, string>;
  touchTarget: string;
};

export type ColorToken = keyof typeof tokens.color;
