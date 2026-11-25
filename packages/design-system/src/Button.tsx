import type { ButtonHTMLAttributes } from "react";

const VARIANTS = {
  primary: "bg-brand text-on-brand border-transparent",
  secondary: "bg-surface-raised text-ink border-line-strong",
  quiet: "bg-transparent text-brand border-transparent hover:bg-brand-soft",
} as const;

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: keyof typeof VARIANTS;
  /** `lg` (56 px) for the main action on a screen. */
  size?: "md" | "lg";
}

/**
 * An action. At least 44 px tall so it is easy to hit on a phone.
 * Defaults to `type="button"`; pass `type="submit"` explicitly inside forms.
 */
export function Button({ type = "button", variant = "primary", size = "md", className = "", ...props }: ButtonProps) {
  const height = `${size === "lg" ? "min-h-14 text-[19px] px-6" : "min-h-12 text-[17px] px-5"}`;
  return (
    <button
      type={type}
      className={`inline-flex min-h-11 items-center justify-center gap-2 rounded-control border-2 font-extrabold transition-transform active:translate-y-px disabled:opacity-45 ${height} ${VARIANTS[variant]} ${className}`.trim()}
      {...props}
    />
  );
}
