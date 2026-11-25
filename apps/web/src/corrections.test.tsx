import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./App";
import type { Transaction, WalletView } from "./api";
import { ELSA_WALLET, FAMILY_ID, elsaWallet, fakeApi, family, me, sek } from "./test/fakeApi";

const walletUrl = `/api/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`;

function correction(of: Transaction): Transaction {
  return {
    ...of,
    id: `corr-${of.id}`,
    direction: of.direction === "in" ? "out" : "in",
    category: "correction",
    memo: "Typed the wrong amount",
    corrects: of.id,
  };
}

function open(wallet: WalletView, role: "owner" | "viewer" = "owner", extra: Parameters<typeof fakeApi>[0] = {}) {
  window.history.replaceState(null, "", `/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`);
  return fakeApi({
    "GET /api/me": () => ({ status: 200, json: me }),
    [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family(role) }),
    [`GET ${walletUrl}`]: () => ({ status: 200, json: wallet }),
    ...extra,
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("correcting a mistake", () => {
  it("records a Correction that takes the money back and keeps the original", async () => {
    const before = elsaWallet();
    const original = before.transactions[0];
    if (!original) throw new Error("fixture has a Transaction");
    const after: WalletView = {
      ...before,
      total: sek(3000),
      transactions: [correction(original), original],
    };
    const sent = open(before, "owner", {
      [`POST ${walletUrl}/corrections`]: () => ({ status: 201, json: after }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: /Pocket money/ }));
    const sheet = screen.getByRole("dialog", { name: "Correct this Deposit?" });
    expect(sheet).toHaveTextContent("Both stay in Elsa's history");
    expect(sheet).toHaveTextContent("−money out 20 kr");
    await userEvent.type(within(sheet).getByLabelText("What went wrong?"), "Typed the wrong amount");
    await userEvent.click(within(sheet).getByRole("button", { name: "Record Correction" }));

    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/corrections`,
      body: { transaction_id: original.id, reason: "Typed the wrong amount" },
    });
    const history = await screen.findByRole("list", { name: "Latest" });
    expect(within(history).getAllByRole("listitem")).toHaveLength(2);
    expect(within(history).getByText("Corrected")).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("offers no Correction for a corrected Transaction or a Correction", async () => {
    const before = elsaWallet();
    const original = before.transactions[0];
    if (!original) throw new Error("fixture has a Transaction");
    open({ ...before, transactions: [correction(original), original] });
    render(<App />);
    const history = await screen.findByRole("list", { name: "Latest" });
    expect(within(history).queryAllByRole("button")).toEqual([]);
  });

  it("will not take back more than the Account has now", async () => {
    const before = elsaWallet();
    const main = before.accounts[0];
    if (!main) throw new Error("fixture has Main");
    open({ ...before, accounts: [{ ...main, balance: sek(500) }, ...before.accounts.slice(1)] });
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: /Pocket money/ }));
    const sheet = screen.getByRole("dialog", { name: "Correct this Deposit?" });
    expect(within(sheet).getByRole("alert")).toHaveTextContent("Main has only 5 kr now");
    expect(within(sheet).getByRole("button", { name: "Record Correction" })).toBeDisabled();
  });

  it("lets a Viewer see the history but not correct it", async () => {
    open(elsaWallet(), "viewer");
    render(<App />);
    const history = await screen.findByRole("list", { name: "Latest" });
    expect(within(history).queryAllByRole("button")).toEqual([]);
  });
});
