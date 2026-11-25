/**
 * Design tokens v0. Placeholder values that meet WCAG AA; the real look and feel
 * (palette, font, mood) is still being designed and will replace these values,
 * not the token names.
 *
 * `theme.css` mirrors these values as Tailwind v4 theme variables. Keep them in sync;
 * the tokens test guards the shape.
 */
export const tokens = {
  color: {
    surface: { light: "#ffffff", dark: "#15161a" },
    ink: { light: "#1d1f24", dark: "#f4f4f5" },
    muted: { light: "#5b5f6a", dark: "#a9acb5" },
    line: { light: "#e4e5e9", dark: "#2c2e35" },
    brand: { light: "#2f6fde", dark: "#7aa7ff" },
    "on-brand": { light: "#ffffff", dark: "#0d1b33" },
    "money-in": { light: "#1d7a3a", dark: "#6fd38f" },
    "money-out": { light: "#b3261e", dark: "#f2a39d" },
  },
  radius: { control: "12px", card: "20px" },
  touchTarget: "44px",
} as const satisfies {
  color: Record<string, { light: string; dark: string }>;
  radius: Record<string, string>;
  touchTarget: string;
};

export type ColorToken = keyof typeof tokens.color;
