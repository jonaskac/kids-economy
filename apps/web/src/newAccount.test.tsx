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

/** Elsa with only her Main account, as every Kid starts. */
function onlyMain(): WalletView {
  const wallet = elsaWallet();
  const main = wallet.accounts[0];
  if (!main) throw new Error("fixture has a Main account");
  return { ...wallet, total: main.balance, accounts: [main] };
}

function withAccount(wallet: WalletView, account: Account): WalletView {
  return { ...wallet, accounts: [...wallet.accounts, account] };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("opening a new Account", () => {
  it("opens a spending Account with just a name", async () => {
    const before = onlyMain();
    const funStuff: Account = { id: "acc-fun", name: "Fun stuff", main: false, balance: sek(0), savings_goal: null };
    const sent = open(before, "owner", {
      [`POST ${walletUrl}/accounts`]: () => ({ status: 201, json: withAccount(before, funStuff) }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "New account" }));
    const sheet = screen.getByRole("dialog", { name: "New account" });
    await userEvent.type(within(sheet).getByLabelText("Account name"), "Fun stuff");
    await userEvent.click(within(sheet).getByRole("button", { name: "Open Fun stuff" }));

    expect(await screen.findByRole("heading", { name: "Fun stuff" })).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByText("In 2 accounts")).toBeInTheDocument();
    expect(sent).toContainEqual({ method: "POST", path: `${walletUrl}/accounts`, body: { name: "Fun stuff" } });
  });

  it("opens a savings Account with a Savings goal", async () => {
    const before = onlyMain();
    const bike: Account = {
      id: "acc-bike",
      name: "Bike",
      main: false,
      balance: sek(0),
      savings_goal: { name: "New bike", target: sek(150000), target_date: "2027-06-01" },
    };
    const sent = open(before, "owner", {
      [`POST ${walletUrl}/accounts`]: () => ({ status: 201, json: withAccount(before, bike) }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "New account" }));
    const sheet = screen.getByRole("dialog", { name: "New account" });
    await userEvent.type(within(sheet).getByLabelText("Account name"), "Bike");
    await userEvent.click(within(sheet).getByRole("radio", { name: /Saving for something/ }));
    await userEvent.type(within(sheet).getByLabelText("Saving for"), "New bike");
    await userEvent.type(within(sheet).getByLabelText("Goal amount"), "1500");
    await userEvent.type(within(sheet).getByLabelText("By when (optional)"), "2027-06-01");
    await userEvent.click(within(sheet).getByRole("button", { name: "Open Bike" }));

    expect(await screen.findByText("Savings · New bike")).toBeInTheDocument();
    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/accounts`,
      body: { name: "Bike", savings_goal: { name: "New bike", target: sek(150000), target_date: "2027-06-01" } },
    });
  });

  it("asks for a name and a goal before it can open", async () => {
    open(onlyMain());
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "New account" }));
    const sheet = screen.getByRole("dialog", { name: "New account" });
    const submit = within(sheet).getByRole("button", { name: "Open account" });
    expect(submit).toBeDisabled();

    await userEvent.type(within(sheet).getByLabelText("Account name"), "   ");
    expect(submit).toBeDisabled();

    await userEvent.type(within(sheet).getByLabelText("Account name"), "Bike");
    await userEvent.click(within(sheet).getByRole("radio", { name: /Saving for something/ }));
    expect(within(sheet).getByRole("button", { name: "Open Bike" })).toBeDisabled();

    await userEvent.type(within(sheet).getByLabelText("Saving for"), "New bike");
    await userEvent.type(within(sheet).getByLabelText("Goal amount"), "abc");
    expect(within(sheet).getByText("Type an amount like 20 or 12,50.")).toBeInTheDocument();
    expect(within(sheet).getByRole("button", { name: "Open Bike" })).toBeDisabled();
  });

  it("keeps names within 40 letters", async () => {
    open(onlyMain());
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "New account" }));
    const sheet = screen.getByRole("dialog", { name: "New account" });
    await userEvent.type(within(sheet).getByLabelText("Account name"), "å".repeat(41));

    expect(within(sheet).getByText("Use at most 40 letters (this has 41).")).toBeInTheDocument();
    expect(within(sheet).getByRole("button", { name: /^Open/ })).toBeDisabled();
  });

  it("says why when the API refuses", async () => {
    open(onlyMain(), "owner", {
      [`POST ${walletUrl}/accounts`]: () => ({
        status: 403,
        json: { error: "forbidden", message: "Only a Parent can do that." },
      }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "New account" }));
    const sheet = screen.getByRole("dialog", { name: "New account" });
    await userEvent.type(within(sheet).getByLabelText("Account name"), "Fun stuff");
    await userEvent.click(within(sheet).getByRole("button", { name: "Open Fun stuff" }));

    expect(await within(sheet).findByRole("alert")).toHaveTextContent("Only a Parent can do that.");
  });

  it("lets money move to the new Account straight away", async () => {
    const before = onlyMain();
    const funStuff: Account = { id: "acc-fun", name: "Fun stuff", main: false, balance: sek(0), savings_goal: null };
    const sent = open(before, "owner", {
      [`POST ${walletUrl}/accounts`]: () => ({ status: 201, json: withAccount(before, funStuff) }),
      [`POST ${walletUrl}/transfers`]: () => ({ status: 201, json: withAccount(before, funStuff) }),
    });
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Elsa's money" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Transfer between accounts" })).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "New account" }));
    await userEvent.type(screen.getByLabelText("Account name"), "Fun stuff");
    await userEvent.click(screen.getByRole("button", { name: "Open Fun stuff" }));

    await userEvent.click(await screen.findByRole("button", { name: "Transfer between accounts" }));
    const sheet = screen.getByRole("dialog", { name: "Transfer" });
    await userEvent.type(within(sheet).getByLabelText("Amount"), "5");
    await userEvent.click(within(sheet).getByRole("button", { name: "Transfer 5 kr" }));

    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/transfers`,
      body: { from: "acc-main", to: "acc-fun", amount: sek(500), memo: "To Fun stuff" },
    });
  });

  it("is not offered to a Viewer", async () => {
    open(onlyMain(), "viewer");
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Elsa's money" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "New account" })).not.toBeInTheDocument();
  });
});
