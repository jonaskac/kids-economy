import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "./App";
import { FAMILY_ID, fakeApi, family, me } from "./test/fakeApi";

const signedOut = { status: 401, json: { error: "not_signed_in" } };
const noFamily = { ...me, families: [] };

beforeEach(() => {
  window.history.replaceState(null, "", "/");
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("signing up", () => {
  it("creates an account from the sign-in screen, then starts a Family", async () => {
    let user: typeof me | null = null;
    const sent = fakeApi({
      "GET /api/me": () => (user ? { status: 200, json: user } : signedOut),
      "POST /api/users": () => {
        user = noFamily;
        return { status: 201, json: { user: me.user } };
      },
      "POST /api/families": () => {
        user = me;
        return { status: 201, json: family() };
      },
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family() }),
    });
    render(<App />);

    await userEvent.click(await screen.findByRole("link", { name: "Create an account" }));
    expect(await screen.findByRole("heading", { name: "Create an account" })).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Your name"), "Jonas");
    await userEvent.type(screen.getByLabelText("Email"), "jonas@example.com");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse battery");
    await userEvent.click(screen.getByRole("button", { name: "Create account" }));

    expect(await screen.findByRole("heading", { name: "Start a Family" })).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Family name"), "The Kacs");
    await userEvent.click(screen.getByRole("button", { name: "Start The Kacs" }));

    expect(await screen.findByRole("heading", { name: "The Kacs" })).toBeInTheDocument();
    expect(sent).toContainEqual({
      method: "POST",
      path: "/api/users",
      body: { email: "jonas@example.com", name: "Jonas", password: "correct horse battery" },
    });
    expect(sent).toContainEqual({
      method: "POST",
      path: "/api/families",
      body: { name: "The Kacs", currency: "SEK" },
    });
  });

  it("says when the email already has an account", async () => {
    fakeApi({
      "GET /api/me": () => signedOut,
      "POST /api/users": () => ({ status: 409, json: { error: "email_taken" } }),
    });
    window.history.replaceState(null, "", "/sign-up");
    render(<App />);
    await userEvent.type(await screen.findByLabelText("Your name"), "Jonas");
    await userEvent.type(screen.getByLabelText("Email"), "jonas@example.com");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse battery");
    await userEvent.click(screen.getByRole("button", { name: "Create account" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("There is already an account with this email.");
  });

  it("asks for at least 10 characters before sending", async () => {
    const sent = fakeApi({ "GET /api/me": () => signedOut });
    window.history.replaceState(null, "", "/sign-up");
    render(<App />);
    await userEvent.type(await screen.findByLabelText("Password"), "short");
    expect(screen.getByRole("button", { name: "Create account" })).toBeDisabled();
    expect(sent.filter((s) => s.method === "POST")).toEqual([]);
  });
});

describe("a forgotten password", () => {
  it("tells the Member to ask an Owner for a reset link", async () => {
    fakeApi({ "GET /api/me": () => signedOut });
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Forgot your password?" }));
    expect(screen.getByText(/Ask an Owner in your Family for a password reset link/)).toBeInTheDocument();
  });

  it("lets the Member choose a new password from the reset link and signs them in", async () => {
    let user: typeof me | null = null;
    const sent = fakeApi({
      "GET /api/me": () => (user ? { status: 200, json: user } : signedOut),
      "GET /api/password-resets/s3cret": () => ({
        status: 200,
        json: { email: "jonas@example.com", name: "Jonas", state: "pending", expires_at: "2026-11-26T13:00:00Z" },
      }),
      "POST /api/password-resets/s3cret": () => {
        user = me;
        return { status: 200, json: { user: me.user } };
      },
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family() }),
    });
    window.history.replaceState(null, "", "/reset/s3cret");
    render(<App />);

    expect(await screen.findByRole("heading", { name: "Choose a new password" })).toBeInTheDocument();
    expect(screen.getByText("jonas@example.com")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("New password"), "a brand new password");
    await userEvent.click(screen.getByRole("button", { name: "Save and sign in" }));

    expect(await screen.findByRole("heading", { name: "The Kacs" })).toBeInTheDocument();
    expect(window.location.pathname).toBe("/");
    expect(sent).toContainEqual({
      method: "POST",
      path: "/api/password-resets/s3cret",
      body: { password: "a brand new password" },
    });
  });

  it("explains a used or expired reset link", async () => {
    fakeApi({
      "GET /api/me": () => signedOut,
      "GET /api/password-resets/old": () => ({
        status: 200,
        json: { email: "jonas@example.com", name: "Jonas", state: "expired", expires_at: "2026-11-20T13:00:00Z" },
      }),
    });
    window.history.replaceState(null, "", "/reset/old");
    render(<App />);
    expect(await screen.findByRole("heading", { name: "This link has expired" })).toBeInTheDocument();
    expect(screen.getByText(/Ask an Owner for a new one/)).toBeInTheDocument();
  });
});

describe("joining by Invitation link", () => {
  const preview = {
    family_name: "The Kacs",
    email: "mormor@example.com",
    role: "viewer",
    state: "pending",
    expires_at: "2026-12-02T13:00:00Z",
  };
  const mormor = {
    user: { id: "usr-2", email: "mormor@example.com", name: "Mormor" },
    families: [] as typeof me.families,
  };

  it("shows the Family and Role, signs the invited adult up and joins", async () => {
    let user: typeof mormor | null = null;
    const sent = fakeApi({
      "GET /api/me": () => (user ? { status: 200, json: user } : signedOut),
      "GET /api/invitations/inv-secret": () => ({ status: 200, json: preview }),
      "POST /api/users": () => {
        user = mormor;
        return { status: 201, json: { user: mormor.user } };
      },
      "POST /api/invitations/inv-secret/accept": () => {
        user = { ...mormor, families: [{ id: FAMILY_ID, name: "The Kacs", role: "viewer" }] };
        return { status: 200, json: { family_id: FAMILY_ID, member_id: "mem-2", role: "viewer" } };
      },
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family("viewer") }),
    });
    window.history.replaceState(null, "", "/join/inv-secret");
    render(<App />);

    expect(await screen.findByRole("heading", { name: "Join The Kacs" })).toBeInTheDocument();
    expect(screen.getByText(/as a Viewer/)).toBeInTheDocument();
    expect(screen.getByLabelText("Email")).toHaveValue("mormor@example.com");
    await userEvent.type(screen.getByLabelText("Your name"), "Mormor");
    await userEvent.type(screen.getByLabelText("Password"), "correct horse battery");
    await userEvent.click(screen.getByRole("button", { name: "Create account and join" }));

    expect(await screen.findByRole("heading", { name: "The Kacs" })).toBeInTheDocument();
    expect(window.location.pathname).toBe("/");
    expect(sent).toContainEqual({
      method: "POST",
      path: "/api/users",
      body: { email: "mormor@example.com", name: "Mormor", password: "correct horse battery" },
    });
    expect(sent.some((s) => s.path === "/api/invitations/inv-secret/accept")).toBe(true);
  });

  it("lets someone with an account sign in and join", async () => {
    let user: typeof mormor | null = null;
    const sent = fakeApi({
      "GET /api/me": () => (user ? { status: 200, json: user } : signedOut),
      "GET /api/invitations/inv-secret": () => ({ status: 200, json: preview }),
      "POST /api/sessions": () => {
        user = mormor;
        return { status: 200, json: { user: mormor.user } };
      },
      "POST /api/invitations/inv-secret/accept": () => {
        user = { ...mormor, families: [{ id: FAMILY_ID, name: "The Kacs", role: "viewer" }] };
        return { status: 200, json: { family_id: FAMILY_ID, member_id: "mem-2", role: "viewer" } };
      },
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: family("viewer") }),
    });
    window.history.replaceState(null, "", "/join/inv-secret");
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "I already have an account" }));
    await userEvent.type(screen.getByLabelText("Password"), "correct horse battery");
    await userEvent.click(screen.getByRole("button", { name: "Sign in and join" }));

    expect(await screen.findByRole("heading", { name: "The Kacs" })).toBeInTheDocument();
    expect(sent).toContainEqual({
      method: "POST",
      path: "/api/sessions",
      body: { email: "mormor@example.com", password: "correct horse battery" },
    });
  });

  it("asks a signed-in adult with another email to sign out first", async () => {
    fakeApi({
      "GET /api/me": () => ({ status: 200, json: me }),
      "GET /api/invitations/inv-secret": () => ({ status: 200, json: preview }),
    });
    window.history.replaceState(null, "", "/join/inv-secret");
    render(<App />);
    expect(await screen.findByText(/This link is for mormor@example.com/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign out" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /join/i })).not.toBeInTheDocument();
  });

  it("explains a link that no longer works", async () => {
    fakeApi({
      "GET /api/me": () => signedOut,
      "GET /api/invitations/gone": () => ({ status: 404, json: { error: "unknown_invitation" } }),
    });
    window.history.replaceState(null, "", "/join/gone");
    render(<App />);
    expect(await screen.findByRole("heading", { name: "This link does not work" })).toBeInTheDocument();
  });
});

describe("Family settings: members", () => {
  const withMembers = () => ({
    ...family(),
    members: [
      { id: "mem-1", name: "Jonas", email: "jonas@example.com", role: "owner" as const, joined_at: "2026-11-25T13:00:00Z" },
      { id: "mem-2", name: "Mormor", email: "mormor@example.com", role: "viewer" as const, joined_at: "2026-11-25T14:00:00Z" },
    ],
    invitations: [{ id: "inv-1", email: "farfar@example.com", role: "viewer" as const, expires_at: "2026-12-02T13:00:00Z" }],
  });

  it("lets an Owner invite an adult and shows the link once", async () => {
    const sent = fakeApi({
      "GET /api/me": () => ({ status: 200, json: me }),
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: withMembers() }),
      [`POST /api/families/${FAMILY_ID}/invitations`]: () => ({
        status: 201,
        json: { invitation: {}, link: "/join/new-secret" },
      }),
    });
    render(<App />);
    await userEvent.click(await screen.findByRole("link", { name: "Family settings" }));

    const list = await screen.findByRole("list", { name: "Members" });
    expect(within(list).getByText("Mormor")).toBeInTheDocument();
    expect(screen.getByText(/farfar@example.com/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Invite an adult" }));
    const sheet = screen.getByRole("dialog", { name: "Invite an adult" });
    await userEvent.type(within(sheet).getByLabelText("Their email"), "pappa@example.com");
    await userEvent.selectOptions(within(sheet).getByLabelText("Role"), "parent");
    await userEvent.click(within(sheet).getByRole("button", { name: "Make Invitation link" }));

    const link = await within(sheet).findByLabelText("Invitation link");
    expect(link).toHaveValue(`${window.location.origin}/join/new-secret`);
    expect(sent).toContainEqual({
      method: "POST",
      path: `/api/families/${FAMILY_ID}/invitations`,
      body: { email: "pappa@example.com", role: "parent" },
    });
  });

  it("lets an Owner make a Password reset link for a Member", async () => {
    const sent = fakeApi({
      "GET /api/me": () => ({ status: 200, json: me }),
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: withMembers() }),
      [`POST /api/families/${FAMILY_ID}/members/mem-2/password-reset`]: () => ({
        status: 201,
        json: { link: "/reset/reset-secret", expires_at: "2026-11-26T13:00:00Z" },
      }),
    });
    window.history.replaceState(null, "", `/families/${FAMILY_ID}/members`);
    render(<App />);

    await userEvent.click(await screen.findByRole("button", { name: "Reset password for Mormor" }));
    const sheet = screen.getByRole("dialog", { name: "Reset Mormor's password" });
    await userEvent.click(within(sheet).getByRole("button", { name: "Make reset link" }));
    expect(await within(sheet).findByLabelText("Password reset link")).toHaveValue(
      `${window.location.origin}/reset/reset-secret`,
    );
    expect(within(sheet).getByText(/Works once, for 24 hours/)).toBeInTheDocument();
    expect(sent.some((s) => s.path.endsWith("/members/mem-2/password-reset"))).toBe(true);
    expect(screen.queryByRole("button", { name: "Reset password for Jonas" })).not.toBeInTheDocument();
  });

  it("shows a Viewer the members but no actions", async () => {
    fakeApi({
      "GET /api/me": () => ({ status: 200, json: me }),
      [`GET /api/families/${FAMILY_ID}`]: () => ({ status: 200, json: { ...withMembers(), you: { member_id: "mem-2", role: "viewer" } } }),
    });
    window.history.replaceState(null, "", `/families/${FAMILY_ID}/members`);
    render(<App />);
    expect(await screen.findByRole("list", { name: "Members" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Invite an adult" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Reset password/ })).not.toBeInTheDocument();
  });
});
