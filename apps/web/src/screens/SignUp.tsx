import { navigate } from "../router";
import { SignUpForm } from "./AuthForms";
import { AppLink, Brand } from "./parts";
import { Screen } from "./Screen";

export function SignUp({ onSignedIn }: { onSignedIn: () => Promise<void> }) {
  return (
    <Screen>
      <Brand
        title="Create an account"
        lead="For the adults. Next you start a Family, or join one with the link you got."
      />
      <SignUpForm
        submitLabel="Create account"
        onDone={async () => {
          navigate("/");
          await onSignedIn();
        }}
      />
      <p className="m-0 text-center text-muted">
        Already have an account? <AppLink to="/">Sign in</AppLink>
      </p>
    </Screen>
  );
}
