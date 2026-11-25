import { useEffect, useState, type FormEvent } from "react";
import { Avatar, BalanceStat, Button, KID_COLOURS, SelectField, Sheet, TextField } from "@kids-economy/design-system";
import { api, ApiFailure, type FamilyView, type Me } from "../api";
import { membersPath, navigate, walletPath } from "../router";
import { AppLink, ErrorText } from "./parts";
import { Screen } from "./Screen";

export interface FamilyHomeProps {
  me: Me;
  onSignOut: () => Promise<void>;
  /** A new Family was started: load who is signed in again. */
  onFamilyStarted: () => Promise<void>;
}

export function FamilyHome({ me, onSignOut, onFamilyStarted }: FamilyHomeProps) {
  const familyId = me.families[0]?.id;
  const [family, setFamily] = useState<FamilyView | null>(null);
  const [adding, setAdding] = useState(false);

  useEffect(() => {
    if (familyId) void api.family(familyId).then(setFamily);
  }, [familyId]);

  if (!familyId) {
    return <StartFamily me={me} onSignOut={onSignOut} onStarted={onFamilyStarted} />;
  }
  if (!family) return <p className="p-4 text-muted">Loading…</p>;

  const mayManageKids = family.you.role !== "viewer";
  return (
    <Screen>
      <header className="flex items-center justify-between gap-3 pt-2">
        <div className="flex flex-col gap-0.5">
          <span className="text-[15px] leading-5 font-bold text-muted">Hi {me.user.name}</span>
          <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">{family.name}</h1>
        </div>
        <Button variant="quiet" onClick={() => void onSignOut()}>
          Sign out
        </Button>
      </header>

      {family.kids.length === 0 ? (
        <section className="flex flex-col items-center gap-3 rounded-card bg-surface-sunken px-6 py-12 text-center">
          <h2 className="m-0 font-display text-[22px] font-semibold">No kids yet</h2>
          <p className="m-0 max-w-[32ch] text-[19px] leading-7 font-semibold text-muted">
            Add a kid to open their Wallet and record their first Deposit.
          </p>
        </section>
      ) : (
        <>
          <p className="m-0 text-muted">Pick a kid to show them their money.</p>
          <ul className="m-0 flex list-none flex-col gap-4 p-0">
            {family.kids.map((kid, index) => (
              <li key={kid.id}>
                <a
                  href={walletPath(family.id, kid.wallet_id)}
                  onClick={(event) => {
                    event.preventDefault();
                    navigate(walletPath(family.id, kid.wallet_id));
                  }}
                  className="flex flex-col gap-3 rounded-card border border-line bg-surface-raised p-4 text-ink no-underline shadow-card transition-transform active:scale-[0.99]"
                >
                  <span className="flex items-center gap-4">
                    <Avatar name={kid.name} colour={KID_COLOURS[index % KID_COLOURS.length]} size="lg" />
                    <span className="grow font-display text-[22px] leading-7 font-semibold">{kid.name}</span>
                    <Chevron />
                  </span>
                  {kid.wallet_total && <BalanceStat label="All together" amount={kid.wallet_total} size="lg" />}
                </a>
              </li>
            ))}
          </ul>
        </>
      )}

      {mayManageKids && (
        <Button variant="secondary" size="lg" onClick={() => setAdding(true)}>
          Add a kid
        </Button>
      )}
      <AppLink to={membersPath(family.id)} className="justify-center">
        Family members
      </AppLink>
      <AddKidSheet
        open={adding}
        familyId={family.id}
        onClose={() => setAdding(false)}
        onAdded={(next) => {
          setFamily(next);
          setAdding(false);
        }}
      />
    </Screen>
  );
}

const CURRENCIES = ["SEK", "NOK", "DKK", "EUR", "GBP", "USD"] as const;

function StartFamily({ me, onSignOut, onStarted }: { me: Me; onSignOut: () => Promise<void>; onStarted: () => Promise<void> }) {
  const [name, setName] = useState("");
  const [currency, setCurrency] = useState<string>("SEK");
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(undefined);
    try {
      await api.createFamily(name.trim(), currency);
      await onStarted();
    } catch (failure) {
      setError(
        failure instanceof ApiFailure && failure.code === "invalid_text"
          ? "Give the Family a name."
          : "Could not start the Family right now. Try again.",
      );
    } finally {
      setBusy(false);
    }
  };
  return (
    <Screen>
      <header className="flex items-center justify-between gap-3 pt-2">
        <span className="text-[15px] leading-5 font-bold text-muted">Hi {me.user.name}</span>
        <Button variant="quiet" onClick={() => void onSignOut()}>
          Sign out
        </Button>
      </header>
      <div className="flex flex-col gap-2">
        <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">Start a Family</h1>
        <p className="m-0 text-muted">
          You become its Owner and can add your kids. Invited by someone else? Open the Invitation link they sent
          you instead.
        </p>
      </div>
      <form onSubmit={submit} className="flex flex-col gap-4">
        <TextField label="Family name" value={name} onChange={setName} placeholder="The Kacs" maxLength={60} />
        <SelectField
          label="Currency"
          hint="Every Kid's money is in this Currency."
          value={currency}
          onChange={setCurrency}
          options={CURRENCIES.map((code) => ({ value: code, label: code }))}
        />
        {error && <ErrorText>{error}</ErrorText>}
        <Button type="submit" size="lg" disabled={name.trim() === "" || busy}>
          Start {name.trim() || "a Family"}
        </Button>
      </form>
    </Screen>
  );
}

function AddKidSheet({
  open,
  familyId,
  onClose,
  onAdded,
}: {
  open: boolean;
  familyId: string;
  onClose: () => void;
  onAdded: (family: FamilyView) => void;
}) {
  const [name, setName] = useState("");
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    await api.addKid(familyId, name);
    setName("");
    onAdded(await api.family(familyId));
  };
  return (
    <Sheet
      open={open}
      title="Add a kid"
      description="They get a Wallet with a Main account."
      onClose={onClose}
      actions={
        <Button type="submit" form="add-kid" size="lg" disabled={name.trim() === ""}>
          Add {name.trim() || "a kid"}
        </Button>
      }
    >
      <form id="add-kid" onSubmit={submit}>
        <TextField label="Name" value={name} onChange={setName} autoComplete="off" />
      </form>
    </Sheet>
  );
}

function Chevron() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" className="flex-none text-muted">
      <path d="m9 18 6-6-6-6" />
    </svg>
  );
}
