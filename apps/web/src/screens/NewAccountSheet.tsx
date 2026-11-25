import { useState, type FormEvent } from "react";
import { AmountField, Button, formatAmount, parseAmount, Sheet, TextField } from "@kids-economy/design-system";
import { api, ApiFailure, type NewAccount, type WalletView } from "../api";
import { ErrorText } from "./parts";

/** Same limit as the API's AccountName (glossary: Account, Savings goal). */
export const NAME_MAX = 40;

export interface NewAccountSheetProps {
  open: boolean;
  familyId: string;
  wallet: WalletView;
  onClose: () => void;
  onDone: (wallet: WalletView) => void;
}

type Purpose = "spending" | "saving";

/** Counts letters the way the API does: Unicode characters, after trimming. */
function nameError(text: string): string | undefined {
  const letters = [...text.trim()].length;
  return letters > NAME_MAX ? `Use at most ${NAME_MAX} letters (this has ${letters}).` : undefined;
}

/** Opens another Account in a Kid's Wallet, optionally with a Savings goal. */
export function NewAccountSheet({ open, familyId, wallet, onClose, onDone }: NewAccountSheetProps) {
  const [name, setName] = useState("");
  const [purpose, setPurpose] = useState<Purpose>("spending");
  const [goalName, setGoalName] = useState("");
  const [targetText, setTargetText] = useState("");
  const [targetDate, setTargetDate] = useState("");
  const [failure, setFailure] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  const saving = purpose === "saving";
  const trimmed = name.trim();
  const nameProblem = nameError(name);
  const goalProblem = nameError(goalName);
  const target = parseAmount(targetText);
  const targetProblem =
    targetText.trim() !== "" && (target === null || target <= 0) ? "Type an amount like 20 or 12,50." : undefined;

  const goalReady = goalName.trim() !== "" && !goalProblem && target !== null && target > 0;
  const ready = trimmed !== "" && !nameProblem && (!saving || goalReady);

  const reset = () => {
    setName("");
    setPurpose("spending");
    setGoalName("");
    setTargetText("");
    setTargetDate("");
    setFailure(undefined);
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    const body: NewAccount = { name: trimmed };
    if (saving && target !== null) {
      body.savings_goal = {
        name: goalName.trim(),
        target: { minor: target, currency: wallet.currency },
        ...(targetDate ? { target_date: targetDate } : {}),
      };
    }
    setBusy(true);
    try {
      const next = await api.openAccount(familyId, wallet.id, body);
      reset();
      onDone(next);
    } catch (caught) {
      setFailure(caught instanceof ApiFailure ? caught.message : "Could not save. Try again.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Sheet
      open={open}
      title="New account"
      description={`Another pot of money for ${wallet.kid.name}. It starts at ${formatAmount({ minor: 0, currency: wallet.currency })}.`}
      onClose={() => {
        reset();
        onClose();
      }}
      actions={
        <Button type="submit" form="new-account" size="lg" disabled={!ready || busy}>
          {trimmed && !nameProblem ? `Open ${trimmed}` : "Open account"}
        </Button>
      }
    >
      <form id="new-account" onSubmit={submit} className="flex flex-col gap-4">
        <TextField
          label="Account name"
          hint="Like Fun stuff or Bike."
          value={name}
          onChange={setName}
          error={nameProblem}
          autoComplete="off"
        />
        <fieldset className="m-0 flex flex-col gap-2 border-0 p-0">
          <legend className="mb-2 p-0 text-[15px] leading-5 font-bold">What is it for?</legend>
          <PurposeOption
            value="spending"
            checked={!saving}
            onPick={setPurpose}
            title="For spending"
            detail="Money to use on small things."
          />
          <PurposeOption
            value="saving"
            checked={saving}
            onPick={setPurpose}
            title="Saving for something"
            detail="Set a goal and watch it fill up."
          />
        </fieldset>
        {saving && (
          <>
            <TextField
              label="Saving for"
              hint="Like New bike or Lego set."
              value={goalName}
              onChange={setGoalName}
              error={goalProblem}
              autoComplete="off"
            />
            <AmountField
              label="Goal amount"
              currency={wallet.currency}
              value={targetText}
              onChange={setTargetText}
              error={targetProblem}
            />
            <TextField label="By when (optional)" type="date" value={targetDate} onChange={setTargetDate} />
          </>
        )}
        {failure && <ErrorText>{failure}</ErrorText>}
      </form>
    </Sheet>
  );
}

function PurposeOption({
  value,
  checked,
  onPick,
  title,
  detail,
}: {
  value: Purpose;
  checked: boolean;
  onPick: (value: Purpose) => void;
  title: string;
  detail: string;
}) {
  return (
    <label className="flex min-h-12 cursor-pointer items-center gap-3 rounded-field border-2 border-line-strong bg-surface-raised px-3 py-2 has-[:checked]:border-brand has-[:checked]:bg-brand-soft">
      <input
        type="radio"
        name="purpose"
        value={value}
        checked={checked}
        onChange={() => onPick(value)}
        className="size-5 shrink-0 accent-brand"
      />
      <span className="flex flex-col">
        <span className="text-[17px] leading-6 font-bold">{title}</span>
        <span className="text-sm font-semibold text-muted">{detail}</span>
      </span>
    </label>
  );
}
