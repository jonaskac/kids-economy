import { useState, type FormEvent } from "react";
import { Button, TextField } from "@kids-economy/design-system";
import { api, ApiFailure } from "../api";
import { ErrorText, MIN_PASSWORD, PASSWORD_HINT } from "./parts";

interface FormProps {
  /** The email the form is for, shown but not changed (an Invitation link). */
  fixedEmail?: string;
  submitLabel: string;
  onDone: () => Promise<void>;
}

/** Email and password; signs in. */
export function SignInForm({ fixedEmail, submitLabel, onDone }: FormProps) {
  const [email, setEmail] = useState(fixedEmail ?? "");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(undefined);
    try {
      await api.signIn(email, password);
      await onDone();
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
    <form onSubmit={submit} className="flex flex-col gap-4">
      <TextField
        label="Email"
        type="email"
        autoComplete="email"
        value={email}
        onChange={setEmail}
        readOnly={fixedEmail !== undefined}
        required
      />
      <TextField
        label="Password"
        type="password"
        autoComplete="current-password"
        value={password}
        onChange={setPassword}
        required
      />
      {error && <ErrorText>{error}</ErrorText>}
      <Button type="submit" size="lg" disabled={busy}>
        {submitLabel}
      </Button>
    </form>
  );
}

const SIGN_UP_ERRORS: Record<string, string> = {
  email_taken: "There is already an account with this email. Sign in instead.",
  invalid_email: "That does not look like an email address.",
  password_too_short: `The password needs at least ${MIN_PASSWORD} characters.`,
  password_too_long: "The password can be at most 128 characters.",
  invalid_text: "Type your name.",
};

/** Name, email and a new password; creates a User and signs them in. */
export function SignUpForm({ fixedEmail, submitLabel, onDone }: FormProps) {
  const [name, setName] = useState("");
  const [email, setEmail] = useState(fixedEmail ?? "");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const ready = name.trim() !== "" && email.trim() !== "" && password.length >= MIN_PASSWORD;

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) return;
    setBusy(true);
    setError(undefined);
    try {
      await api.signUp({ email, name: name.trim(), password });
      await onDone();
    } catch (failure) {
      setError(
        (failure instanceof ApiFailure && SIGN_UP_ERRORS[failure.code]) ||
          "Could not create the account right now. Try again in a moment.",
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={submit} className="flex flex-col gap-4">
      <TextField label="Your name" autoComplete="name" value={name} onChange={setName} maxLength={60} required />
      <TextField
        label="Email"
        type="email"
        autoComplete="email"
        value={email}
        onChange={setEmail}
        readOnly={fixedEmail !== undefined}
        required
      />
      <TextField
        label="Password"
        type="password"
        autoComplete="new-password"
        hint={PASSWORD_HINT}
        value={password}
        onChange={setPassword}
        required
      />
      {error && <ErrorText>{error}</ErrorText>}
      <Button type="submit" size="lg" disabled={!ready || busy}>
        {submitLabel}
      </Button>
    </form>
  );
}
