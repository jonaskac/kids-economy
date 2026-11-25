import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./App";
import type { Account, WalletView } from "./api";
import { ELSA_WALLET, FAMILY_ID, elsaWallet, fakeApi, family, me, sek } from "./test/fakeApi";

const walletUrl = `/api/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`;

function open(wallet: WalletView, role: "owner" | "viewer" = "owner", extra: Parameters<typeof fakeApi>[0] = {}) {
  window.history.replaceState(null, "", `/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`);
  return fakeApi({
    "GET /api/me": () => ({ status: 200, json: me }),
    [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family(role) }),
    [`GET ${walletUrl}`]: () => ({ status: 200, json: wallet }),
    ...extra,
  });
}

/** Elsa's Wallet with a third, empty Account. */
function withBike(): WalletView {
  const wallet = elsaWallet();
  const bike: Account = { id: "acc-bike", name: "Bike", main: false, balance: sek(0), savings_goal: null };
  return { ...wallet, accounts: [...wallet.accounts, bike] };
}

/** What the server answers after closing Savings (30 kr) into `into`. */
function savingsClosedInto(into: string): WalletView {
  const before = withBike();
  const savings = before.accounts.find((a) => a.id === "acc-lego");
  if (!savings) throw new Error("fixture has Savings");
  return {
    ...before,
    accounts: before.accounts
      .filter((a) => a.id !== "acc-lego")
      .map((a) => (a.id === into ? { ...a, balance: sek(a.balance.minor + 3000) } : a)),
    closed_accounts: [{ id: "acc-lego", name: "Savings" }],
    transactions: [
      {
        id: "tx-in",
        account_id: into,
        direction: "in",
        amount: sek(3000),
        category: "transfer",
        memo: "Savings closed",
        occurred_on: "2026-11-25",
        recorded_at: "2026-11-25T12:00:00Z",
        transfer_id: "tr-1",
        corrects: null,
      },
      {
        id: "tx-out",
        account_id: "acc-lego",
        direction: "out",
        amount: sek(3000),
        category: "transfer",
        memo: "Savings closed",
        occurred_on: "2026-11-25",
        recorded_at: "2026-11-25T12:00:00Z",
        transfer_id: "tr-1",
        corrects: null,
      },
      ...before.transactions,
    ],
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("closing an Account", () => {
  it("moves what is left to the Main account, then closes it", async () => {
    const sent = open(withBike(), "owner", {
      [`POST ${walletUrl}/accounts/acc-lego/close`]: () => ({ status: 201, json: savingsClosedInto("acc-main") }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Close Savings" }));
    const sheet = screen.getByRole("dialog", { name: "Close Savings" });
    expect(within(sheet).getByLabelText(/Move .* to/)).toHaveValue("acc-main");
    expect(within(sheet).getByText(/Elsa still has 50 kr in total/)).toBeInTheDocument();
    await userEvent.click(within(sheet).getByRole("button", { name: "Move 30 kr and close" }));

    await vi.waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(screen.queryByRole("heading", { name: "Savings" })).not.toBeInTheDocument();
    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/accounts/acc-lego/close`,
      body: { move_to: "acc-main", memo: "Savings closed" },
    });
    // The closed Account's side of the Transfer still names it in the history.
    expect(screen.getByText(/Savings \(closed\)/)).toBeInTheDocument();
  });

  it("can move what is left to another Account", async () => {
    const sent = open(withBike(), "owner", {
      [`POST ${walletUrl}/accounts/acc-lego/close`]: () => ({ status: 201, json: savingsClosedInto("acc-bike") }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Close Savings" }));
    const sheet = screen.getByRole("dialog", { name: "Close Savings" });
    const target = within(sheet).getByLabelText(/Move .* to/);
    expect(within(target).queryByRole("option", { name: /Savings/ })).not.toBeInTheDocument();
    await userEvent.selectOptions(target, "acc-bike");
    await userEvent.click(within(sheet).getByRole("button", { name: "Move 30 kr and close" }));

    await vi.waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/accounts/acc-lego/close`,
      body: { move_to: "acc-bike", memo: "Savings closed" },
    });
  });

  it("closes an empty Account without asking where money goes", async () => {
    const after = { ...withBike(), accounts: elsaWallet().accounts, closed_accounts: [{ id: "acc-bike", name: "Bike" }] };
    const sent = open(withBike(), "owner", {
      [`POST ${walletUrl}/accounts/acc-bike/close`]: () => ({ status: 201, json: after }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Close Bike" }));
    const sheet = screen.getByRole("dialog", { name: "Close Bike" });
    expect(within(sheet).queryByLabelText(/Move .* to/)).not.toBeInTheDocument();
    await userEvent.click(within(sheet).getByRole("button", { name: "Close Bike" }));

    await vi.waitFor(() => expect(screen.queryByRole("heading", { name: "Bike" })).not.toBeInTheDocument());
    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/accounts/acc-bike/close`,
      body: { memo: "Bike closed" },
    });
  });

  it("never offers to close the Main account, and never to a Viewer", async () => {
    open(withBike());
    const { unmount } = render(<App />);
    expect(await screen.findByRole("button", { name: "Close Savings" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Close Main" })).not.toBeInTheDocument();
    unmount();
    vi.unstubAllGlobals();

    open(withBike(), "viewer");
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Savings" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Close / })).not.toBeInTheDocument();
  });
});
