import { useState, type FormEvent } from "react";
import {
  AmountField,
  Button,
  formatAmount,
  parseAmount,
  SelectField,
  Sheet,
  TextField,
} from "@kids-economy/design-system";
import { api, ApiFailure, type DepositCategory, type WalletView, type WithdrawalCategory } from "../api";
import { CATEGORY_LABEL } from "../words";

const DEPOSIT_CATEGORIES: readonly DepositCategory[] = ["allowance", "chore", "gift", "interest"];
const WITHDRAWAL_CATEGORIES: readonly WithdrawalCategory[] = ["purchase", "donation"];

export interface MoneySheetProps {
  kind: "deposit" | "withdrawal";
  open: boolean;
  familyId: string;
  wallet: WalletView;
  onClose: () => void;
  onDone: (wallet: WalletView) => void;
}

/** Records a Deposit or a Withdrawal on one of the Kid's Accounts. */
export function MoneySheet({ kind, open, familyId, wallet, onClose, onDone }: MoneySheetProps) {
  const main = wallet.accounts.find((a) => a.main) ?? wallet.accounts[0];
  const categories = kind === "deposit" ? DEPOSIT_CATEGORIES : WITHDRAWAL_CATEGORIES;
  const [accountId, setAccountId] = useState(main?.id ?? "");
  const [amountText, setAmountText] = useState("");
  const [category, setCategory] = useState<string>(categories[0] ?? "");
  const [memo, setMemo] = useState("");
  const [failure, setFailure] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  const account = wallet.accounts.find((a) => a.id === accountId) ?? main;
  const minor = parseAmount(amountText);
  const amount = minor === null ? null : { minor, currency: wallet.currency };
  let error: string | undefined;
  if (amountText.trim() !== "" && minor === null) error = "Type an amount like 20 or 12,50.";
  else if (kind === "withdrawal" && account && minor !== null && minor > account.balance.minor)
    error = `That is more than ${account.name} has (${formatAmount(account.balance)}). Try a smaller amount.`;

  const verb = kind === "deposit" ? "Deposit" : "Withdraw";
  const ready = amount !== null && !error && account !== undefined;

  const reset = () => {
    setAmountText("");
    setMemo("");
    setFailure(undefined);
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready || !account) return;
    setBusy(true);
    const text = memo.trim() || CATEGORY_LABEL[category as DepositCategory];
    try {
      const next =
        kind === "deposit"
          ? await api.deposit(familyId, wallet.id, {
              account_id: account.id,
              amount,
              category: category as DepositCategory,
              memo: text,
            })
          : await api.withdraw(familyId, wallet.id, {
              account_id: account.id,
              amount,
              category: category as WithdrawalCategory,
              memo: text,
            });
      reset();
      onDone(next);
    } catch (caught) {
      setFailure(caught instanceof ApiFailure ? caught.message : "Could not save. Try again.");
    } finally {
      setBusy(false);
    }
  };

  const formId = `money-${kind}`;
  return (
    <Sheet
      open={open}
      title={kind === "deposit" ? "Deposit" : "Withdrawal"}
      description={
        kind === "deposit" ? `Money ${wallet.kid.name} gets.` : `Money ${wallet.kid.name} spends or gives away.`
      }
      onClose={() => {
        reset();
        onClose();
      }}
      actions={
        <Button type="submit" form={formId} size="lg" disabled={!ready || busy}>
          {ready && amount ? `${verb} ${formatAmount(amount)}` : verb}
        </Button>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-4">
        <AmountField label="Amount" currency={wallet.currency} value={amountText} onChange={setAmountText} error={error} />
        {wallet.accounts.length > 1 && (
          <SelectField
            label={kind === "deposit" ? "Into" : "From"}
            value={accountId}
            onChange={setAccountId}
            options={wallet.accounts.map((a) => ({ value: a.id, label: `${a.name} · ${formatAmount(a.balance)}` }))}
          />
        )}
        <SelectField
          label="Why"
          value={category}
          onChange={setCategory}
          options={categories.map((c) => ({ value: c, label: CATEGORY_LABEL[c] }))}
        />
        <TextField label="Memo" value={memo} onChange={setMemo} placeholder="What is it for?" maxLength={80} />
        {failure && (
          <p role="alert" className="m-0 rounded-field bg-danger-soft px-3 py-2 text-sm font-bold text-danger">
            {failure}
          </p>
        )}
      </form>
    </Sheet>
  );
}
