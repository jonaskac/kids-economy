import { useState, type FormEvent } from "react";
import { AmountText, Button, formatAmount, Sheet, TextField } from "@kids-economy/design-system";
import { api, ApiFailure, type Transaction, type WalletView } from "../api";
import { shortDay, transactionWord } from "../words";
import { ErrorText } from "./parts";

/** The Transactions a Correction of `t` reverses: `t`, and the other side of a Transfer. */
function legsOf(t: Transaction, wallet: WalletView): Transaction[] {
  if (!t.transfer_id) return [t];
  return wallet.transactions.filter((other) => other.transfer_id === t.transfer_id && other.corrects === null);
}

/** Why the Correction would overdraw an Account (no Overdraft), if it would. */
function overdraft(t: Transaction, wallet: WalletView): string | undefined {
  for (const leg of legsOf(t, wallet)) {
    if (leg.direction !== "in") continue;
    const account = wallet.accounts.find((a) => a.id === leg.account_id);
    if (account && account.balance.minor < leg.amount.minor) {
      return `${account.name} has only ${formatAmount(account.balance)} now, so ${formatAmount(leg.amount)} cannot be taken back. Correct a later Withdrawal or Transfer first.`;
    }
  }
  return undefined;
}

export interface CorrectionSheetProps {
  transaction: Transaction;
  familyId: string;
  wallet: WalletView;
  onClose: () => void;
  onDone: (wallet: WalletView) => void;
}

/** Records a Correction: a new Transaction that reverses a mistaken one. Nothing is edited. */
export function CorrectionSheet({ transaction: t, familyId, wallet, onClose, onDone }: CorrectionSheetProps) {
  const [reason, setReason] = useState("");
  const [failure, setFailure] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const word = transactionWord(t.category);
  const problem = overdraft(t, wallet);
  const ready = reason.trim() !== "" && !problem;

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    setBusy(true);
    try {
      onDone(await api.correct(familyId, wallet.id, { transaction_id: t.id, reason: reason.trim() }));
    } catch (caught) {
      setFailure(caught instanceof ApiFailure ? caught.message : "Could not save. Try again.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Sheet
      open
      title={`Correct this ${word}?`}
      description={`A Correction takes the money back. Both stay in ${wallet.kid.name}'s history, like at a real bank.${
        t.transfer_id ? " Both sides of the Transfer are taken back." : ""
      }`}
      onClose={onClose}
      actions={
        <Button type="submit" form="correction" size="lg" disabled={!ready || busy}>
          Record Correction
        </Button>
      }
    >
      <div className="flex items-center justify-between gap-3 rounded-control bg-surface-sunken px-4 py-3">
        <span className="min-w-0">
          <span className="block text-[17px] font-bold [overflow-wrap:anywhere]">{t.memo}</span>
          <span className="text-sm font-semibold text-muted">
            {word} · {shortDay(t.occurred_on)}
          </span>
        </span>
        <span className="text-right">
          <AmountText amount={t.amount} direction={t.direction === "in" ? "out" : "in"} className="text-[22px]" />
          <span className="block text-sm font-bold text-muted">Correction</span>
        </span>
      </div>
      <form id="correction" onSubmit={submit} className="flex flex-col gap-4">
        <TextField
          label="What went wrong?"
          hint={`${wallet.kid.name} sees this as the Memo.`}
          value={reason}
          onChange={setReason}
          placeholder="Typed the wrong amount"
          maxLength={80}
        />
        {problem && <ErrorText>{problem}</ErrorText>}
        {failure && <ErrorText>{failure}</ErrorText>}
      </form>
    </Sheet>
  );
}
