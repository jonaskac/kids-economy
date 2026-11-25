import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Button } from "./Button";

describe("Button", () => {
  it("is a button labelled by its text", () => {
    render(<Button>Record deposit</Button>);
    expect(screen.getByRole("button", { name: "Record deposit" })).toBeInTheDocument();
  });

  it("calls onClick when pressed", async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Save</Button>);
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("does not submit forms unless asked to", () => {
    render(<Button>Cancel</Button>);
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveAttribute("type", "button");
  });

  it("meets the 44 px touch target on phones", () => {
    render(<Button>Tap me</Button>);
    expect(screen.getByRole("button", { name: "Tap me" })).toHaveClass("min-h-11");
  });

  it("comes in quieter variants for secondary actions", () => {
    render(<Button variant="secondary">Cancel</Button>);
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveClass("border-line-strong");
  });

  it("has a danger variant for actions that cannot be undone from the app", () => {
    render(<Button variant="danger">Remove Elsa</Button>);
    const button = screen.getByRole("button", { name: "Remove Elsa" });
    expect(button).toHaveClass("text-danger");
    expect(button).toHaveClass("border-danger");
  });
});
