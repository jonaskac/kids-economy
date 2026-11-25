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
  /** Left off the Family home; still listed in Family settings (ADR-0013). */
  hidden: boolean;
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

/** Body of `POST …/accounts` (docs/api.md). */
export interface NewAccount {
  name: string;
  savings_goal?: { name: string; target: Amount; target_date?: string };
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
  /** Closed Accounts, kept so their Transactions can still name them. */
  closed_accounts: { id: string; name: string }[];
  transactions: Transaction[];
}

export type LinkState = "pending" | "accepted" | "revoked" | "expired";

export interface InvitationPreview {
  family_name: string;
  email: string;
  role: Role;
  state: LinkState;
  expires_at: string;
}

export interface PasswordResetPreview {
  email: string;
  name: string;
  state: "pending" | "used" | "expired";
  expires_at: string;
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
  signUp: (body: { email: string; name: string; password: string }) =>
    call<{ user: User }>("POST", "/api/users", body),
  signOut: () => call<undefined>("DELETE", "/api/sessions/current"),
  createFamily: (name: string, currency: string) => call<FamilyView>("POST", "/api/families", { name, currency }),
  invite: (familyId: string, email: string, role: Role) =>
    call<{ link: string }>("POST", `/api/families/${familyId}/invitations`, { email, role }),
  invitation: (secret: string) => call<InvitationPreview>("GET", `/api/invitations/${encodeURIComponent(secret)}`),
  acceptInvitation: (secret: string) =>
    call<{ family_id: string; member_id: string; role: Role }>(
      "POST",
      `/api/invitations/${encodeURIComponent(secret)}/accept`,
      {},
    ),
  passwordResetLink: (familyId: string, memberId: string) =>
    call<{ link: string; expires_at: string }>(
      "POST",
      `/api/families/${familyId}/members/${memberId}/password-reset`,
      {},
    ),
  passwordReset: (secret: string) =>
    call<PasswordResetPreview>("GET", `/api/password-resets/${encodeURIComponent(secret)}`),
  resetPassword: (secret: string, password: string) =>
    call<{ user: User }>("POST", `/api/password-resets/${encodeURIComponent(secret)}`, { password }),
  family: (familyId: string) => call<FamilyView>("GET", `/api/families/${familyId}`),
  addKid: (familyId: string, name: string) => call<KidSummary>("POST", `/api/families/${familyId}/kids`, { name }),
  /** Fixes a Kid's name, or hides or shows them on the Family home. */
  changeKid: (familyId: string, kidId: string, body: { name?: string; hidden?: boolean }) =>
    call<undefined>("PATCH", `/api/families/${familyId}/kids/${kidId}`, body),
  /** "Remove kid": archives them; their Wallet and History are kept. */
  removeKid: (familyId: string, kidId: string) =>
    call<undefined>("POST", `/api/families/${familyId}/kids/${kidId}/archive`, {}),
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
  openAccount: (familyId: string, walletId: string, body: NewAccount) =>
    call<WalletView>("POST", `${wallet(familyId, walletId)}/accounts`, body),
  /** Moves what is left to `move_to` (default: the Main account), then closes the Account. */
  closeAccount: (familyId: string, walletId: string, accountId: string, body: { move_to?: string; memo: string }) =>
    call<WalletView>("POST", `${wallet(familyId, walletId)}/accounts/${accountId}/close`, body),
  correct: (familyId: string, walletId: string, body: { transaction_id: string; reason: string }) =>
    call<WalletView>("POST", `${wallet(familyId, walletId)}/corrections`, body),
};
