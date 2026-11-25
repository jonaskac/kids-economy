import { useState } from "react";
import { Button } from "@kids-economy/design-system";
import { SignInForm } from "./AuthForms";
import { AppLink, Brand } from "./parts";
import { Screen } from "./Screen";

export function SignIn({ onSignedIn }: { onSignedIn: () => Promise<void> }) {
  const [forgot, setForgot] = useState(false);
  return (
    <Screen>
      <Brand title="Kids Economy" lead="Sign in to show your kids their money." />
      <SignInForm submitLabel="Sign in" onDone={onSignedIn} />
      <div className="flex flex-col items-center gap-2 text-center">
        <Button variant="quiet" aria-expanded={forgot} onClick={() => setForgot(!forgot)}>
          Forgot your password?
        </Button>
        {forgot && (
          <p className="m-0 max-w-[36ch] rounded-control bg-surface-sunken px-4 py-3 text-[15px] leading-5 font-semibold text-muted">
            Ask an Owner in your Family for a password reset link. They make it under Family members and send
            it to you.
          </p>
        )}
        <p className="m-0 text-muted">
          New here? <AppLink to="/sign-up">Create an account</AppLink>
        </p>
      </div>
    </Screen>
  );
}
