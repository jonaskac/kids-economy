import { formatAmount, parseAmount } from "./money";

const sek = (minor: number) => ({ minor, currency: "SEK" });

describe("formatAmount", () => {
  it("drops ,00 from whole amounts and puts kr after the number", () => {
    expect(formatAmount(sek(2000))).toBe("20 kr");
  });

  it("shows öre with a comma", () => {
    expect(formatAmount(sek(1250))).toBe("12,50 kr");
    expect(formatAmount(sek(1205))).toBe("12,05 kr");
  });

  it("groups thousands with a narrow space that never breaks", () => {
    expect(formatAmount(sek(124000))).toBe("1\u202f240 kr");
  });

  it("shows the size of a negative amount; the sign is the caller's job", () => {
    expect(formatAmount(sek(-500))).toBe("5 kr");
  });

  it("falls back to the currency code", () => {
    expect(formatAmount({ minor: 300, currency: "CHF" })).toBe("3 CHF");
  });
});

describe("parseAmount", () => {
  it("reads whole amounts and öre with a comma or a dot, without floats", () => {
    expect(parseAmount("20")).toBe(2000);
    expect(parseAmount("12,5")).toBe(1250);
    expect(parseAmount("12.05")).toBe(1205);
    expect(parseAmount(" 0,10 ")).toBe(10);
  });

  it("says no to anything else", () => {
    expect(parseAmount("")).toBeNull();
    expect(parseAmount("abc")).toBeNull();
    expect(parseAmount("1,234")).toBeNull();
    expect(parseAmount("-5")).toBeNull();
    expect(parseAmount("0")).toBeNull();
  });
});
