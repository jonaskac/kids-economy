import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./App";
import { ELSA_WALLET, FAMILY_ID, elsaWallet, fakeApi, family, me, sek } from "./test/fakeApi";

const walletUrl = `/api/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`;

function signedIn(role: "owner" | "viewer" = "owner", extra: Parameters<typeof fakeApi>[0] = {}) {
  return fakeApi({
    "GET /api/me": () => ({ status: 200, json: me }),
    [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family(role) }),
    [`GET ${walletUrl}`]: () => ({ status: 200, json: elsaWallet() }),
    ...extra,
  });
}

beforeEach(() => {
  window.history.replaceState(null, "", "/");
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("signing in", () => {
  it("asks for email and password, then shows the Family", async () => {
    let session = false;
    const sent = fakeApi({
      "GET /api/me": () => (session ? { status: 200, json: me } : { status: 401, json: { error: "not_signed_in" } }),
      "POST /api/sessions": () => {
        session = true;
        return { status: 200, json: { user: me.user } };
      },
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family() }),
    });
    render(<App />);

    await userEvent.type(await screen.findByLabelText("Email"), "jonas@example.com");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse battery");
    await userEvent.click(screen.getByRole("button", { name: "Sign in" }));

    expect(await screen.findByRole("heading", { name: "The Kacs" })).toBeInTheDocument();
    expect(sent).toContainEqual({
      method: "POST",
      path: "/api/sessions",
      body: { email: "jonas@example.com", password: "correct horse battery" },
    });
  });

  it("says so when the email or password is wrong", async () => {
    fakeApi({
      "GET /api/me": () => ({ status: 401, json: { error: "not_signed_in" } }),
      "POST /api/sessions": () => ({ status: 401, json: { error: "wrong_email_or_password" } }),
    });
    render(<App />);
    await userEvent.type(await screen.findByLabelText("Email"), "jonas@example.com");
    await userEvent.type(screen.getByLabelText("Password"), "nope nope nope");
    await userEvent.click(screen.getByRole("button", { name: "Sign in" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("That email and password do not match.");
  });
});

describe("the Family home", () => {
  it("shows every Kid with how much they have", async () => {
    signedIn();
    render(<App />);
    const elsa = await screen.findByRole("link", { name: /Elsa/ });
    expect(elsa).toHaveTextContent("50 kr");
    expect(screen.getByRole("link", { name: /Leo/ })).toHaveTextContent("8,50 kr");
  });
});

describe("a Kid's Wallet", () => {
  it("opens from the Family home with the total, Accounts and history", async () => {
    signedIn();
    render(<App />);
    await userEvent.click(await screen.findByRole("link", { name: /Elsa/ }));

    expect(await screen.findByRole("heading", { name: "Elsa's money" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Lego set: 7 percent saved" })).toBeInTheDocument();
    const history = screen.getByRole("list", { name: "Latest" });
    expect(within(history).getByRole("listitem")).toHaveTextContent("Pocket money");
    expect(within(history).getByRole("listitem")).toHaveTextContent("Allowance · Sun 22 Nov · Main");
  });

  it("records a Deposit on the Main account and shows the new total", async () => {
    const after = { ...elsaWallet(), total: sek(7000) };
    const sent = signedIn("owner", {
      [`POST ${walletUrl}/deposits`]: () => ({ status: 201, json: after }),
    });
    window.history.replaceState(null, "", `/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`);
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Deposit money" }));
    const sheet = screen.getByRole("dialog", { name: "Deposit" });
    await userEvent.type(within(sheet).getByLabelText("Amount"), "20");
    await userEvent.type(within(sheet).getByLabelText("Memo"), "Washed the car");
    await userEvent.selectOptions(within(sheet).getByLabelText("Why"), "chore");
    await userEvent.click(within(sheet).getByRole("button", { name: "Deposit 20 kr" }));

    expect(await screen.findByText("70 kr")).toBeInTheDocument();
    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/deposits`,
      body: { account_id: "acc-main", amount: sek(2000), category: "chore", memo: "Washed the car" },
    });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("will not take out more than an Account has", async () => {
    signedIn();
    window.history.replaceState(null, "", `/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`);
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Withdraw money" }));
    const sheet = screen.getByRole("dialog", { name: "Withdrawal" });
    await userEvent.type(within(sheet).getByLabelText("Amount"), "25");
    expect(within(sheet).getByRole("alert")).toHaveTextContent("That is more than Main has (20 kr).");
    expect(within(sheet).getByRole("button", { name: /Withdraw/ })).toBeDisabled();
  });

  it("previews a Transfer before and after, and keeps the total", async () => {
    const sent = signedIn("owner", {
      [`POST ${walletUrl}/transfers`]: () => ({ status: 201, json: elsaWallet() }),
    });
    window.history.replaceState(null, "", `/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`);
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Transfer between accounts" }));
    const sheet = screen.getByRole("dialog", { name: "Transfer" });
    await userEvent.selectOptions(within(sheet).getByLabelText("To"), "acc-lego");
    await userEvent.type(within(sheet).getByLabelText("Amount"), "5");
    expect(sheet).toHaveTextContent("Main20 kr → 15 kr");
    expect(sheet).toHaveTextContent("Savings30 kr → 35 kr");
    expect(sheet).toHaveTextContent("Elsa still has 50 kr in total.");
    await userEvent.click(within(sheet).getByRole("button", { name: "Transfer 5 kr" }));

    expect(sent).toContainEqual({
      method: "POST",
      path: `${walletUrl}/transfers`,
      body: { from: "acc-main", to: "acc-lego", amount: sek(500), memo: "To Savings" },
    });
  });

  it("lets a Viewer look but not change anything", async () => {
    signedIn("viewer");
    window.history.replaceState(null, "", `/families/${FAMILY_ID}/wallets/${ELSA_WALLET}`);
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Elsa's money" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Deposit money" })).not.toBeInTheDocument();
    expect(screen.getByText(/You can look along with Elsa/)).toBeInTheDocument();
  });
});
