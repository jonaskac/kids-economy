import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./App";
import type { FamilyView } from "./api";
import { FAMILY_ID, fakeApi, family, me } from "./test/fakeApi";

const familyUrl = `/api/families/${FAMILY_ID}`;

/** Elsa and Leo, with Leo hidden from the Family home. */
function leoHidden(role: "owner" | "viewer" = "owner"): FamilyView {
  const view = family(role);
  return { ...view, kids: view.kids.map((kid) => (kid.name === "Leo" ? { ...kid, hidden: true } : kid)) };
}

/** Serves `views` in turn on each `GET` of the Family, the last one from then on. */
function serve(views: FamilyView[], extra: Parameters<typeof fakeApi>[0] = {}) {
  let next = 0;
  return fakeApi({
    "GET /api/me": () => ({ status: 200, json: me }),
    [`GET ${familyUrl}`]: () => ({ status: 200, json: views[Math.min(next++, views.length - 1)] }),
    ...extra,
  });
}

function openSettings() {
  window.history.replaceState(null, "", `/families/${FAMILY_ID}/settings`);
  render(<App />);
}

afterEach(() => {
  vi.unstubAllGlobals();
  window.history.replaceState(null, "", "/");
});

describe("hidden Kids on the Family home", () => {
  it("leaves a hidden Kid off and says where to find them", async () => {
    serve([leoHidden()]);
    window.history.replaceState(null, "", "/");
    render(<App />);

    expect(await screen.findByRole("link", { name: /Elsa/ })).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /Leo/ })).not.toBeInTheDocument();
    expect(screen.getByText("1 kid is hidden. Show them again in Family settings.")).toBeInTheDocument();
  });
});

describe("Family settings: Kids", () => {
  it("lists every Kid, hidden ones marked", async () => {
    serve([leoHidden()]);
    openSettings();
    const kids = await screen.findByRole("list", { name: "Kids" });
    expect(within(kids).getByText("Elsa")).toBeInTheDocument();
    expect(within(kids).getByText("Hidden from Family home")).toBeInTheDocument();
  });

  it("fixes a Kid's name entered by mistake", async () => {
    const before = family();
    const after: FamilyView = {
      ...before,
      kids: before.kids.map((kid) => (kid.name === "Elsa" ? { ...kid, name: "Elsa M" } : kid)),
    };
    const sent = serve([before, after], {
      [`PATCH ${familyUrl}/kids/kid-elsa`]: () => ({ status: 204 }),
    });
    openSettings();

    await userEvent.click(await screen.findByRole("button", { name: "Edit Elsa" }));
    const sheet = screen.getByRole("dialog", { name: "Edit Elsa" });
    const name = within(sheet).getByLabelText("Name");
    expect(name).toHaveValue("Elsa");
    await userEvent.clear(name);
    await userEvent.type(name, "Elsa M");
    await userEvent.click(within(sheet).getByRole("button", { name: "Save name" }));

    expect(await screen.findByRole("button", { name: "Edit Elsa M" })).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(sent).toContainEqual({ method: "PATCH", path: `${familyUrl}/kids/kid-elsa`, body: { name: "Elsa M" } });
  });

  it("hides a Kid from the Family home", async () => {
    const before = family();
    const after: FamilyView = {
      ...before,
      kids: before.kids.map((kid) => (kid.name === "Elsa" ? { ...kid, hidden: true } : kid)),
    };
    const sent = serve([before, after], {
      [`PATCH ${familyUrl}/kids/kid-elsa`]: () => ({ status: 204 }),
    });
    openSettings();

    await userEvent.click(await screen.findByRole("button", { name: "Edit Elsa" }));
    const sheet = screen.getByRole("dialog", { name: "Edit Elsa" });
    expect(within(sheet).getByText(/Their money stays and keeps working/)).toBeInTheDocument();
    await userEvent.click(within(sheet).getByRole("button", { name: "Hide from Family home" }));

    expect(await screen.findByText("Hidden from Family home")).toBeInTheDocument();
    expect(sent).toContainEqual({ method: "PATCH", path: `${familyUrl}/kids/kid-elsa`, body: { hidden: true } });
  });

  it("shows a hidden Kid again", async () => {
    const sent = serve([leoHidden(), family()], {
      [`PATCH ${familyUrl}/kids/kid-leo`]: () => ({ status: 204 }),
    });
    openSettings();

    await userEvent.click(await screen.findByRole("button", { name: "Edit Leo" }));
    const sheet = screen.getByRole("dialog", { name: "Edit Leo" });
    await userEvent.click(within(sheet).getByRole("button", { name: "Show on Family home" }));

    await screen.findByRole("button", { name: "Edit Leo" });
    expect(screen.queryByText("Hidden from Family home")).not.toBeInTheDocument();
    expect(sent).toContainEqual({ method: "PATCH", path: `${familyUrl}/kids/kid-leo`, body: { hidden: false } });
  });

  it("removes a Kid only after asking, and says the history is kept", async () => {
    const before = family();
    const after: FamilyView = { ...before, kids: before.kids.filter((kid) => kid.name !== "Elsa") };
    const sent = serve([before, after], {
      [`POST ${familyUrl}/kids/kid-elsa/archive`]: () => ({ status: 204 }),
    });
    openSettings();

    await userEvent.click(await screen.findByRole("button", { name: "Edit Elsa" }));
    await userEvent.click(screen.getByRole("button", { name: "Remove Elsa" }));
    expect(sent.some((s) => s.path.endsWith("/archive"))).toBe(false);

    const ask = screen.getByRole("dialog", { name: "Remove Elsa?" });
    expect(within(ask).getByText(/kept, but no one in the Family sees them/)).toBeInTheDocument();
    await userEvent.click(within(ask).getByRole("button", { name: "Yes, remove Elsa" }));

    const kids = await screen.findByRole("list", { name: "Kids" });
    await vi.waitFor(() => expect(within(kids).queryByText("Elsa")).not.toBeInTheDocument());
    expect(sent).toContainEqual({ method: "POST", path: `${familyUrl}/kids/kid-elsa/archive`, body: {} });
  });

  it("shows a Viewer the Kids but no way to change them", async () => {
    serve([leoHidden("viewer")]);
    openSettings();
    expect(await screen.findByRole("list", { name: "Kids" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Edit/ })).not.toBeInTheDocument();
  });
});
