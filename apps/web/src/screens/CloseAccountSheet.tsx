import { useState } from "react";
import { Button, formatAmount, SelectField, Sheet } from "@kids-economy/design-system";
import { api, ApiFailure, type Account, type WalletView } from "../api";
import { ErrorText } from "./parts";

export interface CloseAccountSheetProps {
  account: Account;
  familyId: string;
  wallet: WalletView;
  onClose: () => void;
  onDone: (wallet: WalletView) => void;
}

/**
 * Closes an Account (ADR-0013). Whatever is left moves first, as one Transfer, to the
 * Main account or another open Account, so the Kid's total stays the same.
 */
export function CloseAccountSheet({ account, familyId, wallet, onClose, onDone }: CloseAccountSheetProps) {
  const targets = wallet.accounts.filter((a) => a.id !== account.id);
  const main = targets.find((a) => a.main) ?? targets[0];
  const [targetId, setTargetId] = useState(main?.id ?? "");
  const [failure, setFailure] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const left = account.balance;
  const hasMoney = left.minor > 0;
  const target = targets.find((a) => a.id === targetId);

  const close = async () => {
    setBusy(true);
    setFailure(undefined);
    try {
      const memo = `${account.name} closed`;
      const next = await api.closeAccount(
        familyId,
        wallet.id,
        account.id,
        hasMoney ? { move_to: targetId, memo } : { memo },
      );
      onDone(next);
    } catch (caught) {
      setFailure(caught instanceof ApiFailure ? caught.message : "Could not close it right now. Try again.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Sheet
      open
      title={`Close ${account.name}`}
      description={
        hasMoney
          ? `${account.name} has ${formatAmount(left)}. It moves to another account first, then ${account.name} closes.`
          : `${account.name} is empty. Once closed it takes no more money, but its history stays.`
      }
      onClose={onClose}
      actions={
        <Button size="lg" disabled={busy || (hasMoney && !target)} onClick={() => void close()}>
          {hasMoney ? `Move ${formatAmount(left)} and close` : `Close ${account.name}`}
        </Button>
      }
    >
      {hasMoney && (
        <>
          <SelectField
            label={`Move ${formatAmount(left)} to`}
            value={targetId}
            onChange={setTargetId}
            options={targets.map((a) => ({ value: a.id, label: `${a.name} · ${formatAmount(a.balance)}` }))}
          />
          <p className="m-0 rounded-control bg-surface-sunken px-4 py-3 text-sm font-semibold text-muted">
            {wallet.kid.name} still has {formatAmount(wallet.total)} in total. Closing an account does not change it.
          </p>
        </>
      )}
      {failure && <ErrorText>{failure}</ErrorText>}
    </Sheet>
  );
}
