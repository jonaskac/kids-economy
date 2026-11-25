/**
 * Glossary: Amount. Integer minor units (öre, cents) plus an ISO 4217 Currency.
 * Never a float: formatting and parsing below use integer maths only.
 */
export interface Amount {
  minor: number;
  currency: string;
}

const SYMBOLS: Record<string, { symbol: string; after: boolean }> = {
  SEK: { symbol: "kr", after: true },
  NOK: { symbol: "kr", after: true },
  DKK: { symbol: "kr", after: true },
  EUR: { symbol: "€", after: true },
  USD: { symbol: "$", after: false },
  GBP: { symbol: "£", after: false },
};

/** A narrow no-break space: groups thousands and never lets money break across lines. */
const GROUP = " ";

/** The currency's symbol as people say it ("kr"), or its code. */
export function currencySymbol(currency: string): string {
  return SYMBOLS[currency]?.symbol ?? currency;
}

/** "20 kr", "12,50 kr", "1 240 kr". Shows the size only; the sign is the caller's job. */
export function formatAmount(amount: Amount): string {
  const abs = Math.abs(amount.minor);
  const cents = abs % 100;
  const major = String((abs - cents) / 100).replace(/\B(?=(\d{3})+(?!\d))/g, GROUP);
  const number = cents === 0 ? major : `${major},${String(cents).padStart(2, "0")}`;
  const style = SYMBOLS[amount.currency] ?? { symbol: amount.currency, after: true };
  return style.after ? `${number} ${style.symbol}` : `${style.symbol}${number}`;
}

/** Reads what an adult types ("20", "12,50", "12.5") as minor units. `null` unless above zero. */
export function parseAmount(text: string): number | null {
  const match = /^(\d{1,7})(?:[.,](\d{1,2}))?$/.exec(text.trim());
  if (!match) return null;
  const major = Number.parseInt(match[1] ?? "0", 10);
  const cents = match[2] ? Number.parseInt(match[2].padEnd(2, "0"), 10) : 0;
  const minor = major * 100 + cents;
  return minor > 0 ? minor : null;
}
