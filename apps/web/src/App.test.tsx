import { render, screen } from "@testing-library/react";
import { App } from "./App";

describe("App shell", () => {
  it("names the app", () => {
    render(<App />);
    expect(screen.getByRole("heading", { level: 1, name: "Kids Economy" })).toBeInTheDocument();
  });

  it("offers the first step for an empty Family", () => {
    render(<App />);
    expect(screen.getByRole("button", { name: "Add a kid" })).toBeInTheDocument();
  });
});
