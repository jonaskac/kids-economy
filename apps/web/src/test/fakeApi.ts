import type { FamilyView, Me, WalletView } from "../api";

export const sek = (minor: number) => ({ minor, currency: "SEK" });

export const FAMILY_ID = "fam-1";
export const ELSA_WALLET = "wal-elsa";

export function elsaWallet(): WalletView {
  return {
    id: ELSA_WALLET,
    kid: { id: "kid-elsa", name: "Elsa" },
    currency: "SEK",
    total: sek(5000),
    accounts: [
      { id: "acc-main", name: "Main", main: true, balance: sek(2000), savings_goal: null },
      {
        id: "acc-lego",
        name: "Savings",
        main: false,
        balance: sek(3000),
        savings_goal: { name: "Lego set", target: sek(40000), target_date: null },
      },
    ],
    closed_accounts: [],
    transactions: [
      {
        id: "tx-1",
        account_id: "acc-main",
        direction: "in",
        amount: sek(2000),
        category: "allowance",
        memo: "Pocket money",
        occurred_on: "2026-11-22",
        recorded_at: "2026-11-22T10:00:00Z",
        transfer_id: null,
        corrects: null,
      },
    ],
  };
}

export function family(role: "owner" | "viewer" = "owner"): FamilyView {
  return {
    id: FAMILY_ID,
    name: "The Kacs",
    currency: "SEK",
    you: { member_id: "mem-1", role },
    members: [],
    invitations: [],
    kids: [
      { id: "kid-elsa", name: "Elsa", wallet_id: ELSA_WALLET, hidden: false, wallet_total: sek(5000) },
      { id: "kid-leo", name: "Leo", wallet_id: "wal-leo", hidden: false, wallet_total: sek(850) },
    ],
  };
}

export const me: Me = {
  user: { id: "usr-1", email: "jonas@example.com", name: "Jonas" },
  families: [{ id: FAMILY_ID, name: "The Kacs", role: "owner" }],
};

export interface Sent {
  method: string;
  path: string;
  body: unknown;
}

type Handler = (body: unknown) => { status: number; json?: unknown };

/** Replaces `fetch` with a tiny fake API. Returns what the app sent. */
export function fakeApi(routes: Record<string, Handler>): Sent[] {
  const sent: Sent[] = [];
  vi.stubGlobal("fetch", async (path: string, init?: RequestInit) => {
    const method = init?.method ?? "GET";
    const body: unknown = typeof init?.body === "string" ? JSON.parse(init.body) : undefined;
    sent.push({ method, path, body });
    const handler = routes[`${method} ${path}`];
    const reply = handler
      ? handler(body)
      : { status: 404, json: { error: "not_found", message: `no fake for ${method} ${path}` } };
    return new Response(reply.json === undefined ? null : JSON.stringify(reply.json), {
      status: reply.status,
      headers: { "Content-Type": "application/json" },
    });
  });
  return sent;
}
