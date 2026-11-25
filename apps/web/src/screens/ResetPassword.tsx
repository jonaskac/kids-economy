import { useEffect, useState, type FormEvent } from "react";
import { Button, TextField } from "@kids-economy/design-system";
import { api, ApiFailure, type PasswordResetPreview } from "../api";
import { Brand, ErrorText, LinkProblem, MIN_PASSWORD, PASSWORD_HINT } from "./parts";
import { Screen } from "./Screen";

type Loaded = { state: "loading" } | { state: "unknown" } | { state: "found"; preview: PasswordResetPreview };

const ASK_AGAIN = "Ask an Owner for a new one. They make it under Family members.";

/** `/reset/<secret>`: the Member chooses a new password from a Password reset link. */
export function ResetPassword({ secret, onSignedIn }: { secret: string; onSignedIn: () => Promise<void> }) {
  const [loaded, setLoaded] = useState<Loaded>({ state: "loading" });
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.passwordReset(secret).then(
      (preview) => setLoaded({ state: "found", preview }),
      () => setLoaded({ state: "unknown" }),
    );
  }, [secret]);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(undefined);
    try {
      await api.resetPassword(secret, password);
      await onSignedIn();
    } catch (failure) {
      const code = failure instanceof ApiFailure ? failure.code : "";
      setError(
        code === "password_reset_used" || code === "password_reset_expired"
          ? `This link no longer works. ${ASK_AGAIN}`
          : code === "password_too_long"
            ? "The password can be at most 128 characters."
            : "Could not save the password right now. Try again in a moment.",
      );
    } finally {
      setBusy(false);
    }
  };

  let body;
  if (loaded.state === "loading") body = <p className="text-muted">Loading…</p>;
  else if (loaded.state === "unknown") body = <LinkProblem title="This link does not work" advice={ASK_AGAIN} />;
  else if (loaded.preview.state === "used") body = <LinkProblem title="This link was already used" advice={ASK_AGAIN} />;
  else if (loaded.preview.state === "expired") body = <LinkProblem title="This link has expired" advice={ASK_AGAIN} />;
  else
    body = (
      <>
        <Brand
          title="Choose a new password"
          lead={
            <>
              For <strong className="text-ink">{loaded.preview.email}</strong>. Saving it signs you out on every
              other phone.
            </>
          }
        />
        <form onSubmit={submit} className="flex flex-col gap-4">
          <TextField
            label="New password"
            type="password"
            autoComplete="new-password"
            hint={PASSWORD_HINT}
            value={password}
            onChange={setPassword}
            required
          />
          {error && <ErrorText>{error}</ErrorText>}
          <Button type="submit" size="lg" disabled={password.length < MIN_PASSWORD || busy}>
            Save and sign in
          </Button>
        </form>
      </>
    );
  return <Screen>{body}</Screen>;
}
