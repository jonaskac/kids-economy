/**
 * Typed client for the Kids Economy API (docs/api.md). Same origin: Vite proxies `/api`
 * in development, the server serves both in production. The Session travels as a cookie.
 */
import type { Amount } from "@kids-economy/design-system";

export type Role = "owner" | "parent" | "viewer";

export interface User {
  id: string;
  email: string;
  name: string;
}

export interface Me {
  user: User;
  families: { id: string; name: string; role: Role }[];
}

export interface KidSummary {
  id: string;
  name: string;
  wallet_id: string;
  wallet_total?: Amount;
}

export interface FamilyView {
  id: string;
  name: string;
  currency: string;
  you: { member_id: string; role: Role };
  members: { id: string; name: string; email: string; role: Role; joined_at: string }[];
  invitations: { id: string; email: string; role: Role; expires_at: string }[];
  kids: KidSummary[];
}

export interface SavingsGoal {
  name: string;
  target: Amount;
  target_date: string | null;
}

export interface Account {
  id: string;
  name: string;
  main: boolean;
  balance: Amount;
  savings_goal: SavingsGoal | null;
}

export type Category =
  | "allowance"
  | "chore"
  | "gift"
  | "interest"
  | "purchase"
  | "donation"
  | "transfer"
  | "correction";

export type DepositCategory = Extract<Category, "allowance" | "chore" | "gift" | "interest">;
export type WithdrawalCategory = Extract<Category, "purchase" | "donation">;

export interface Transaction {
  id: string;
  account_id: string;
  direction: "in" | "out";
  amount: Amount;
  category: Category;
  memo: string;
  occurred_on: string;
  recorded_at: string;
  transfer_id: string | null;
  corrects: string | null;
}

export interface WalletView {
  id: string;
  kid: { id: string; name: string };
  currency: string;
  total: Amount;
  accounts: Account[];
  transactions: Transaction[];
}

/** The API said no: its status and stable error code (docs/api.md, Conventions). */
export class ApiFailure extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiFailure";
  }
}

async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const init: RequestInit = { method, credentials: "same-origin" };
  if (body !== undefined) {
    init.headers = { "Content-Type": "application/json" };
    init.body = JSON.stringify(body);
  }
  const response = await fetch(path, init);
  const text = await response.text();
  const json: unknown = text ? JSON.parse(text) : undefined;
  if (!response.ok) {
    const error = (json ?? {}) as { error?: string; message?: string };
    throw new ApiFailure(response.status, error.error ?? "unknown", error.message ?? response.statusText);
  }
  return json as T;
}

const wallet = (familyId: string, walletId: string) => `/api/families/${familyId}/wallets/${walletId}`;

export const api = {
  me: () => call<Me>("GET", "/api/me"),
  signIn: (email: string, password: string) => call<{ user: User }>("POST", "/api/sessions", { email, password }),
  signOut: () => call<undefined>("DELETE", "/api/sessions/current"),
  family: (familyId: string) => call<FamilyView>("GET", `/api/families/${familyId}`),
  addKid: (familyId: string, name: string) => call<KidSummary>("POST", `/api/families/${familyId}/kids`, { name }),
  wallet: (familyId: string, walletId: string) => call<WalletView>("GET", wallet(familyId, walletId)),
  deposit: (
    familyId: string,
    walletId: string,
    body: { account_id: string; amount: Amount; category: DepositCategory; memo: string },
  ) => call<WalletView>("POST", `${wallet(familyId, walletId)}/deposits`, body),
  withdraw: (
    familyId: string,
    walletId: string,
    body: { account_id: string; amount: Amount; category: WithdrawalCategory; memo: string },
  ) => call<WalletView>("POST", `${wallet(familyId, walletId)}/withdrawals`, body),
  transfer: (
    familyId: string,
    walletId: string,
    body: { from: string; to: string; amount: Amount; memo: string },
  ) => call<WalletView>("POST", `${wallet(familyId, walletId)}/transfers`, body),
};
