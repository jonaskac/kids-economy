import { useState, type FormEvent } from "react";
import { AmountField, Button, formatAmount, parseAmount, SelectField, Sheet } from "@kids-economy/design-system";
import { api, ApiFailure, type WalletView } from "../api";

export interface TransferSheetProps {
  open: boolean;
  familyId: string;
  wallet: WalletView;
  onClose: () => void;
  onDone: (wallet: WalletView) => void;
}

/** Moves money between two of the Kid's Accounts. The total never changes. */
export function TransferSheet({ open, familyId, wallet, onClose, onDone }: TransferSheetProps) {
  const accounts = wallet.accounts;
  const [fromId, setFromId] = useState(accounts[0]?.id ?? "");
  const [toId, setToId] = useState(accounts[1]?.id ?? "");
  const [amountText, setAmountText] = useState("");
  const [failure, setFailure] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  const from = accounts.find((a) => a.id === fromId);
  const to = accounts.find((a) => a.id === toId);
  const minor = parseAmount(amountText);
  let error: string | undefined;
  if (fromId === toId) error = "Pick two different accounts.";
  else if (amountText.trim() !== "" && minor === null) error = "Type an amount like 20 or 12,50.";
  else if (from && minor !== null && minor > from.balance.minor)
    error = `That is more than ${from.name} has (${formatAmount(from.balance)}). Try a smaller amount.`;
  const ready = !error && minor !== null && from !== undefined && to !== undefined;
  const money = (value: number) => formatAmount({ minor: value, currency: wallet.currency });

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready || minor === null) return;
    setBusy(true);
    try {
      const next = await api.transfer(familyId, wallet.id, {
        from: from.id,
        to: to.id,
        amount: { minor, currency: wallet.currency },
        memo: `To ${to.name}`,
      });
      setAmountText("");
      setFailure(undefined);
      onDone(next);
    } catch (caught) {
      setFailure(caught instanceof ApiFailure ? caught.message : "Could not save. Try again.");
    } finally {
      setBusy(false);
    }
  };

  const options = accounts.map((a) => ({ value: a.id, label: `${a.name} · ${formatAmount(a.balance)}` }));
  return (
    <Sheet
      open={open}
      title="Transfer"
      description={`From one of ${wallet.kid.name}'s accounts to another.`}
      onClose={() => {
        setAmountText("");
        setFailure(undefined);
        onClose();
      }}
      actions={
        <Button type="submit" form="transfer" size="lg" disabled={!ready || busy}>
          {ready ? `Transfer ${money(minor)}` : "Transfer"}
        </Button>
      }
    >
      <form id="transfer" onSubmit={submit} className="flex flex-col gap-4">
        <SelectField label="From" value={fromId} onChange={setFromId} options={options} />
        <SelectField label="To" value={toId} onChange={setToId} options={options} />
        <AmountField label="Amount" currency={wallet.currency} value={amountText} onChange={setAmountText} error={error} />
        {ready && (
          <div className="flex flex-col gap-2 rounded-control bg-surface-sunken px-4 py-3">
            <PreviewLine name={from.name} before={money(from.balance.minor)} after={money(from.balance.minor - minor)} tone="text-money-out" />
            <PreviewLine name={to.name} before={money(to.balance.minor)} after={money(to.balance.minor + minor)} tone="text-money-in" />
            <span className="text-sm font-semibold text-muted">
              {wallet.kid.name} still has {formatAmount(wallet.total)} in total. A Transfer does not change it.
            </span>
          </div>
        )}
        {failure && (
          <p role="alert" className="m-0 rounded-field bg-danger-soft px-3 py-2 text-sm font-bold text-danger">
            {failure}
          </p>
        )}
      </form>
    </Sheet>
  );
}

function PreviewLine({ name, before, after, tone }: { name: string; before: string; after: string; tone: string }) {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <span className="text-[17px] font-bold">{name}</span>
      <span className="font-display text-[19px] font-semibold tabular-nums">
        {before} → <span className={tone}>{after}</span>
      </span>
    </div>
  );
}
