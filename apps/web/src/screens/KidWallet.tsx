import { useEffect, useState } from "react";
import {
  AmountText,
  Avatar,
  KID_COLOURS,
  BalanceStat,
  Button,
  ProgressRing,
  TransactionRow,
} from "@kids-economy/design-system";
import { api, type Account, type FamilyView, type Transaction, type WalletView } from "../api";
import { navigate } from "../router";
import { CATEGORY_LABEL, shortDay, transactionWord } from "../words";
import { CorrectionSheet } from "./CorrectionSheet";
import { MoneySheet } from "./MoneySheet";
import { Screen } from "./Screen";
import { TransferSheet } from "./TransferSheet";

type Open = "none" | "deposit" | "withdrawal" | "transfer";

export function KidWallet({ familyId, walletId }: { familyId: string; walletId: string }) {
  const [family, setFamily] = useState<FamilyView | null>(null);
  const [wallet, setWallet] = useState<WalletView | null>(null);
  const [open, setOpen] = useState<Open>("none");
  const [correcting, setCorrecting] = useState<Transaction | null>(null);

  useEffect(() => {
    void api.family(familyId).then(setFamily);
    void api.wallet(familyId, walletId).then(setWallet);
  }, [familyId, walletId]);

  if (!family || !wallet) return <p className="p-4 text-muted">Loading…</p>;

  const kid = wallet.kid.name;
  const kidIndex = family.kids.findIndex((k) => k.wallet_id === wallet.id);
  const mayChange = family.you.role !== "viewer";
  const accountName = new Map(wallet.accounts.map((a) => [a.id, a.name]));
  const changed = (next: WalletView) => {
    setWallet(next);
    setOpen("none");
    setCorrecting(null);
  };
  const corrected = new Set(wallet.transactions.flatMap((t) => (t.corrects ? [t.corrects] : [])));
  const mayCorrect = (t: Transaction) =>
    mayChange && t.category !== "correction" && !corrected.has(t.id) && accountName.has(t.account_id);
  const close = () => setOpen("none");

  return (
    <Screen>
      <header className="flex items-center justify-between gap-3">
        <a
          href="/"
          onClick={(event) => {
            event.preventDefault();
            navigate("/");
          }}
          className="inline-flex min-h-12 items-center gap-1 pr-2 text-[15px] leading-5 font-extrabold text-brand no-underline"
        >
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
            <path d="m15 18-6-6 6-6" />
          </svg>
          Family
        </a>
        <Avatar name={kid} colour={KID_COLOURS[Math.max(0, kidIndex) % KID_COLOURS.length]} />
      </header>

      <section className="flex flex-col items-center gap-4 text-center">
        <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">{kid}'s money</h1>
        <BalanceStat
          label="All together"
          amount={wallet.total}
          caption={`In ${wallet.accounts.length} ${wallet.accounts.length === 1 ? "account" : "accounts"}`}
        />
      </section>

      {mayChange ? (
        <div className="flex flex-col gap-3">
          <div className="grid grid-cols-2 gap-3">
            <Button size="lg" className="min-w-0 px-3 leading-tight" onClick={() => setOpen("deposit")}>
              Deposit money
            </Button>
            <Button
              size="lg"
              variant="secondary"
              className="min-w-0 px-3 leading-tight"
              onClick={() => setOpen("withdrawal")}
            >
              Withdraw money
            </Button>
          </div>
          {wallet.accounts.length > 1 && (
            <Button variant="quiet" onClick={() => setOpen("transfer")}>
              Transfer between accounts
            </Button>
          )}
        </div>
      ) : (
        <p className="m-0 rounded-control bg-surface-sunken px-4 py-3 text-[15px] leading-5 font-semibold text-muted">
          You can look along with {kid}. A Parent records Deposits and Withdrawals.
        </p>
      )}

      <section className="flex flex-col gap-3">
        <h2 className="m-0 font-display text-[22px] leading-7 font-semibold">Accounts</h2>
        {wallet.accounts.map((account) => (
          <AccountCard key={account.id} account={account} />
        ))}
      </section>

      <section className="flex flex-col gap-1">
        <h2 id="latest" className="m-0 font-display text-[22px] leading-7 font-semibold">
          Latest
        </h2>
        {wallet.transactions.length === 0 ? (
          <p className="m-0 text-[19px] leading-7 font-semibold text-muted">Nothing yet. The first Deposit shows up here.</p>
        ) : (
          <ul aria-labelledby="latest" className="m-0 list-none p-0">
            {wallet.transactions.map((t) => (
              <TransactionRow
                key={t.id}
                memo={t.memo}
                meta={`${CATEGORY_LABEL[t.category]} · ${shortDay(t.occurred_on)} · ${accountName.get(t.account_id) ?? "Closed account"}`}
                amount={t.amount}
                direction={t.direction}
                word={transactionWord(t.category)}
                {...(corrected.has(t.id) ? { note: "Corrected" } : {})}
                {...(mayCorrect(t) ? { onSelect: () => setCorrecting(t) } : {})}
              />
            ))}
          </ul>
        )}
      </section>

      <MoneySheet
        kind="deposit"
        open={open === "deposit"}
        familyId={familyId}
        wallet={wallet}
        onClose={close}
        onDone={changed}
      />
      <MoneySheet
        kind="withdrawal"
        open={open === "withdrawal"}
        familyId={familyId}
        wallet={wallet}
        onClose={close}
        onDone={changed}
      />
      {correcting && (
        <CorrectionSheet
          transaction={correcting}
          familyId={familyId}
          wallet={wallet}
          onClose={() => setCorrecting(null)}
          onDone={changed}
        />
      )}
      <TransferSheet open={open === "transfer"} familyId={familyId} wallet={wallet} onClose={close} onDone={changed} />
    </Screen>
  );
}

function AccountCard({ account }: { account: Account }) {
  const goal = account.savings_goal;
  const caption = account.main ? "New money lands here" : goal ? `Savings · ${goal.name}` : "For spending";
  return (
    <article className="flex flex-col gap-3 rounded-card border border-line bg-surface-raised p-4 shadow-card">
      <div className="flex items-center justify-between gap-3">
        <div className="flex flex-col gap-0.5">
          <h3 className="m-0 text-[19px] leading-[26px] font-bold">{account.name}</h3>
          <span className="text-sm font-semibold text-muted">{caption}</span>
        </div>
        <AmountText amount={account.balance} className="text-[28px]" />
      </div>
      {goal && <ProgressRing label={goal.name} saved={account.balance} target={goal.target} />}
    </article>
  );
}
