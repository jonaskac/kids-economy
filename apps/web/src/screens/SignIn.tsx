import { useState, type FormEvent } from "react";
import { Button, TextField } from "@kids-economy/design-system";
import { api, ApiFailure } from "../api";
import { Screen } from "./Screen";

export function SignIn({ onSignedIn }: { onSignedIn: () => Promise<void> }) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(undefined);
    try {
      await api.signIn(email, password);
      await onSignedIn();
    } catch (failure) {
      setError(
        failure instanceof ApiFailure && failure.code === "wrong_email_or_password"
          ? "That email and password do not match. Try again."
          : "Could not sign in right now. Try again in a moment.",
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <Screen>
      <div className="flex flex-col gap-2 pt-12">
        <span aria-hidden="true" className="flex gap-1.5">
          <span className="h-3 w-10 rounded-full bg-coin-strong" />
          <span className="h-3 w-10 rounded-full bg-coin" />
          <span className="size-3 rounded-full bg-brand" />
        </span>
        <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">Kids Economy</h1>
        <p className="m-0 text-muted">Sign in to show your kids their money.</p>
      </div>
      <form onSubmit={submit} className="flex flex-col gap-4">
        <TextField label="Email" type="email" autoComplete="email" value={email} onChange={setEmail} required />
        <TextField
          label="Password"
          type="password"
          autoComplete="current-password"
          value={password}
          onChange={setPassword}
          required
        />
        {error && (
          <p role="alert" className="m-0 rounded-field bg-danger-soft px-3 py-2 text-sm font-bold text-danger">
            {error}
          </p>
        )}
        <Button type="submit" size="lg" disabled={busy}>
          Sign in
        </Button>
      </form>
    </Screen>
  );
}
