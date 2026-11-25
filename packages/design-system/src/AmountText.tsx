import { formatAmount, type Amount } from "./money";

export type MoneyDirection = "in" | "out" | "none";

export interface AmountTextProps {
  amount: Amount;
  direction?: MoneyDirection;
  className?: string;
}

/** An Amount with its sign and, for screen readers, its word. Colour is never the only signal. */
export function AmountText({ amount, direction = "none", className = "" }: AmountTextProps) {
  const colour = direction === "in" ? "text-money-in" : direction === "out" ? "text-money-out" : "text-ink";
  return (
    <span className={`font-display font-semibold whitespace-nowrap tabular-nums ${colour} ${className}`.trim()}>
      {direction !== "none" && (
        <>
          <span aria-hidden="true">{direction === "in" ? "+" : "−"}</span>
          <span className="sr-only">{direction === "in" ? "money in " : "money out "}</span>
        </>
      )}
      {formatAmount(amount)}
    </span>
  );
}
