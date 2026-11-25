import { formatAmount, type Amount } from "./money";

export interface BalanceStatProps {
  amount: Amount;
  label?: string;
  caption?: string;
  /** `display` is the one big Balance a Kid reads from across the table. */
  size?: "display" | "lg";
}

export function BalanceStat({ amount, label, caption, size = "display" }: BalanceStatProps) {
  const text = formatAmount(amount);
  const valueSize =
    size === "display"
      ? text.length > 9
        ? "text-[clamp(36px,12.5vw,72px)] leading-[1.05]"
        : "text-[clamp(48px,17vw,72px)] leading-[1.05]"
      : "text-[40px] leading-[44px]";
  return (
    <div className="flex flex-col gap-1">
      {label && <span className="text-[15px] leading-5 font-bold text-muted">{label}</span>}
      <span aria-live="polite" className={`font-display font-semibold tracking-tight whitespace-nowrap tabular-nums ${valueSize}`}>
        {text}
      </span>
      {caption && <span className="text-sm font-semibold text-muted">{caption}</span>}
    </div>
  );
}
