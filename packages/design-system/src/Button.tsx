import type { ButtonHTMLAttributes } from "react";

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement>;

/**
 * The primary action. At least 44 px tall so it is easy to hit on a phone.
 * Defaults to `type="button"`; pass `type="submit"` explicitly inside forms.
 */
export function Button({ type = "button", className = "", ...props }: ButtonProps) {
  return (
    <button
      type={type}
      className={`inline-flex min-h-11 items-center justify-center rounded-control bg-brand px-5 text-base font-semibold text-on-brand disabled:opacity-50 ${className}`.trim()}
      {...props}
    />
  );
}
