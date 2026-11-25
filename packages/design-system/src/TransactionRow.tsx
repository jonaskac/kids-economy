import { AmountText, type MoneyDirection } from "./AmountText";
import type { Amount } from "./money";

export interface TransactionRowProps {
  memo: string;
  /** When, and on which Account: "Sun 22 Nov · Main". */
  meta: string;
  amount: Amount;
  direction: Exclude<MoneyDirection, "none">;
  /** The glossary word shown under the Amount: Deposit, Withdrawal, Transfer, Correction. */
  word?: string;
  /** A short status under the meta line, such as "Corrected". */
  note?: string;
  /** Makes the whole row one button (at least 44 px tall), e.g. to correct it. */
  onSelect?: () => void;
}

function Glyph({ direction }: { direction: "in" | "out" }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="22"
      height="22"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.6"
      strokeLinecap="round"
      aria-hidden="true"
    >
      {direction === "in" ? <path d="M12 5v14M5 12h14" /> : <path d="M5 12h14" />}
    </svg>
  );
}

/** One row of a bank-style history. Render inside a `<ul>`. */
export function TransactionRow({ memo, meta, amount, direction, word, note, onSelect }: TransactionRowProps) {
  const tone = direction === "in" ? "bg-brand-soft text-money-in" : "bg-money-out-soft text-money-out";
  const grid = "grid w-full grid-cols-[auto_1fr_auto] items-center gap-3 py-3 text-left";
  const content = (
    <>
      <span className={`grid size-11 place-items-center rounded-control ${tone}`}>
        <Glyph direction={direction} />
      </span>
      <span className="min-w-0">
        <span className="block text-[19px] leading-[26px] font-bold [overflow-wrap:anywhere]">{memo}</span>
        <span className="block text-sm font-semibold text-muted">{meta}</span>
        {note && (
          <span className="mt-1 inline-block rounded-full bg-surface-sunken px-2 py-0.5 text-sm font-bold text-muted">
            {note}
          </span>
        )}
      </span>
      <span className="text-right">
        <AmountText amount={amount} direction={direction} className="text-[22px] leading-7" />
        {word && (
          <span className={`block text-sm font-bold ${direction === "in" ? "text-money-in" : "text-money-out"}`}>
            {word}
          </span>
        )}
      </span>
    </>
  );
  return (
    <li className="border-b border-line last:border-b-0">
      {onSelect ? (
        <button
          type="button"
          onClick={onSelect}
          className={`${grid} min-h-11 cursor-pointer rounded-control border-0 bg-transparent text-ink hover:bg-surface-sunken`}
        >
          {content}
        </button>
      ) : (
        <div className={grid}>{content}</div>
      )}
    </li>
  );
}
