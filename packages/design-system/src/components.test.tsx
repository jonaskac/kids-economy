import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AmountText } from "./AmountText";
import { Avatar } from "./Avatar";
import { BalanceStat } from "./BalanceStat";
import { ProgressRing } from "./ProgressRing";
import { Sheet } from "./Sheet";
import { TransactionRow } from "./TransactionRow";
import { AmountField, SelectField, TextField } from "./Field";

const sek = (minor: number) => ({ minor, currency: "SEK" });

describe("AmountText", () => {
  it("shows a sign and says the word, not colour alone", () => {
    render(<AmountText amount={sek(2000)} direction="in" />);
    expect(screen.getByText("+")).toBeInTheDocument();
    expect(screen.getByText(/money in/i)).toBeInTheDocument();
  });

  it("uses a real minus for money out", () => {
    render(<AmountText amount={sek(500)} direction="out" />);
    expect(screen.getByText("−")).toBeInTheDocument();
    expect(screen.getByText(/money out/i)).toBeInTheDocument();
  });
});

describe("BalanceStat", () => {
  it("labels the Balance so it reads as one phrase", () => {
    render(<BalanceStat label="All together" amount={sek(124000)} />);
    expect(screen.getByText("All together")).toBeInTheDocument();
    expect(screen.getByText("All together").parentElement?.textContent).toBe("All together1\u202f240 kr");
  });
});

describe("Avatar", () => {
  it("shows the first letter and is named for screen readers", () => {
    render(<Avatar name="Elsa" />);
    expect(screen.getByRole("img", { name: "Elsa" })).toHaveTextContent("E");
  });
});

describe("ProgressRing", () => {
  it("says how far the Savings goal is, in words", () => {
    render(<ProgressRing label="Lego set" saved={sek(1000)} target={sek(4000)} />);
    expect(screen.getByRole("img", { name: "Lego set: 25 percent saved" })).toBeInTheDocument();
    expect(screen.getByText("30 kr to go")).toBeInTheDocument();
  });

  it("celebrates a reached goal", () => {
    render(<ProgressRing label="Lego set" saved={sek(5000)} target={sek(4000)} />);
    expect(screen.getByText("Goal reached")).toBeInTheDocument();
  });
});

describe("TransactionRow", () => {
  it("shows the Memo, when and on which Account, and the signed Amount", () => {
    render(
      <ul>
        <TransactionRow memo="Pocket money" meta="Sun 22 Nov · Main" amount={sek(2000)} direction="in" />
      </ul>,
    );
    const row = screen.getByRole("listitem");
    expect(row).toHaveTextContent("Pocket money");
    expect(row).toHaveTextContent("Sun 22 Nov · Main");
    expect(row).toHaveTextContent("+money in 20 kr");
  });
});

describe("TransactionRow, selectable", () => {
  it("becomes one button when it can be selected, and shows a note", async () => {
    const onSelect = vi.fn();
    render(
      <ul>
        <TransactionRow
          memo="Pocket money"
          meta="Sun 22 Nov · Main"
          amount={sek(2000)}
          direction="in"
          word="Deposit"
          note="Corrected"
          onSelect={onSelect}
        />
      </ul>,
    );
    const button = screen.getByRole("button", { name: /Pocket money/ });
    expect(button).toHaveTextContent("Corrected");
    await userEvent.click(button);
    expect(onSelect).toHaveBeenCalledOnce();
  });
});

describe("Sheet", () => {
  it("is a labelled dialog that closes from its Cancel button", async () => {
    const onClose = vi.fn();
    render(
      <Sheet open title="Move money" onClose={onClose}>
        <p>inside</p>
      </Sheet>,
    );
    expect(screen.getByRole("dialog", { name: "Move money" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("renders nothing when closed", () => {
    render(<Sheet open={false} title="Move money" onClose={() => undefined} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});

describe("Fields", () => {
  it("label their controls and announce errors", () => {
    render(
      <>
        <TextField label="Memo" value="" onChange={() => undefined} />
        <AmountField label="Amount" currency="SEK" value="" onChange={() => undefined} error="Type an amount like 20 or 12,50." />
        <SelectField
          label="From"
          value="a"
          onChange={() => undefined}
          options={[{ value: "a", label: "Main" }]}
        />
      </>,
    );
    expect(screen.getByLabelText("Memo")).toBeInTheDocument();
    const amount = screen.getByLabelText("Amount");
    expect(amount).toHaveAttribute("inputmode", "decimal");
    expect(amount).toHaveAttribute("aria-invalid", "true");
    expect(screen.getByRole("alert")).toHaveTextContent("Type an amount");
    expect(screen.getByRole("combobox", { name: "From" })).toBeInTheDocument();
  });
});
