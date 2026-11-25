import { tokens } from "./tokens";

describe("tokens", () => {
  it("names the colours the apps rely on", () => {
    expect(Object.keys(tokens.color)).toEqual(
      expect.arrayContaining(["surface", "ink", "muted", "brand", "on-brand", "money-in", "money-out", "line"]),
    );
  });

  it("names the surfaces, saving and kid colours of the v1 look", () => {
    expect(Object.keys(tokens.color)).toEqual(
      expect.arrayContaining([
        "surface-raised",
        "surface-sunken",
        "line-strong",
        "brand-soft",
        "coin-strong",
        "coin-soft",
        "coin-ink",
        "money-out-soft",
        "danger",
        "danger-soft",
        "focus",
        "kid-sea",
        "kid-sun",
        "kid-berry",
        "kid-leaf",
        "kid-sky",
      ]),
    );
  });

  it("uses a rounded display face for Balances and a friendly text face", () => {
    expect(tokens.font.display).toMatch(/^Fredoka/);
    expect(tokens.font.text).toMatch(/^Nunito/);
  });

  it("defines every colour for both light and dark", () => {
    for (const [name, value] of Object.entries(tokens.color)) {
      expect(value.light, `${name} light`).toMatch(/^#[0-9a-f]{6}$/i);
      expect(value.dark, `${name} dark`).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });
});
