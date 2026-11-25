import { tokens } from "./tokens";

describe("tokens", () => {
  it("names the colours the apps rely on", () => {
    expect(Object.keys(tokens.color)).toEqual(
      expect.arrayContaining(["surface", "ink", "muted", "brand", "on-brand", "money-in", "money-out", "line"]),
    );
  });

  it("defines every colour for both light and dark", () => {
    for (const [name, value] of Object.entries(tokens.color)) {
      expect(value.light, `${name} light`).toMatch(/^#[0-9a-f]{6}$/i);
      expect(value.dark, `${name} dark`).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });
});
