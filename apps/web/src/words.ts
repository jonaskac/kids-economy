import type { Category } from "./api";

/** The glossary word for what kind of Transaction a Category belongs to. */
export function transactionWord(category: Category): string {
  switch (category) {
    case "allowance":
    case "chore":
    case "gift":
    case "interest":
      return "Deposit";
    case "purchase":
    case "donation":
      return "Withdrawal";
    case "transfer":
      return "Transfer";
    case "correction":
      return "Correction";
  }
}

export const CATEGORY_LABEL: Record<Category, string> = {
  allowance: "Allowance",
  chore: "Chore",
  gift: "Gift",
  interest: "Interest",
  purchase: "Purchase",
  donation: "Donation",
  transfer: "Transfer",
  correction: "Correction",
};

/** "Sun 22 Nov" for a `YYYY-MM-DD` date, read as a calendar day (no time zone shift). */
export function shortDay(isoDate: string): string {
  const [year, month, day] = isoDate.split("-").map((part) => Number.parseInt(part, 10));
  if (!year || !month || !day) return isoDate;
  return new Intl.DateTimeFormat("en-GB", {
    weekday: "short",
    day: "numeric",
    month: "short",
    timeZone: "UTC",
  }).format(new Date(Date.UTC(year, month - 1, day)));
}
