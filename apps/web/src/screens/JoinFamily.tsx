import { useEffect, useState } from "react";
import { Button } from "@kids-economy/design-system";
import { api, ApiFailure, type InvitationPreview, type Me, type Role } from "../api";
import { SignInForm, SignUpForm } from "./AuthForms";
import { Brand, ErrorText, LinkProblem } from "./parts";
import { Screen } from "./Screen";

type Loaded = { state: "loading" } | { state: "unknown" } | { state: "found"; preview: InvitationPreview };

const ROLE_LINE: Record<Role, string> = {
  owner: "as an Owner: you can record money for every Kid and invite other adults.",
  parent: "as a Parent: you can record money for every Kid.",
  viewer: "as a Viewer: you can see every Kid's money, but not change it.",
};

const ASK_AGAIN = "Ask the Owner who sent it for a new one.";

const ACCEPT_ERRORS: Record<string, string> = {
  already_member: "You are already in this Family.",
  wrong_email: "This link is for another email address.",
  invitation_not_pending: `This link no longer works. ${ASK_AGAIN}`,
  invitation_expired: `This link has expired. ${ASK_AGAIN}`,
};

export interface JoinFamilyProps {
  secret: string;
  /** The signed-in adult, if any. */
  me: Me | null;
  onJoined: () => Promise<void>;
  onSignOut: () => Promise<void>;
}

/** `/join/<secret>`: an invited adult signs up or in, then joins the Family. */
export function JoinFamily({ secret, me, onJoined, onSignOut }: JoinFamilyProps) {
  const [loaded, setLoaded] = useState<Loaded>({ state: "loading" });
  const [haveAccount, setHaveAccount] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.invitation(secret).then(
      (preview) => setLoaded({ state: "found", preview }),
      () => setLoaded({ state: "unknown" }),
    );
  }, [secret]);

  const accept = async () => {
    setBusy(true);
    setError(undefined);
    try {
      await api.acceptInvitation(secret);
      await onJoined();
    } catch (failure) {
      setError(
        (failure instanceof ApiFailure && ACCEPT_ERRORS[failure.code]) || "Could not join right now. Try again.",
      );
    } finally {
      setBusy(false);
    }
  };

  if (loaded.state === "loading") return <p className="p-4 text-muted">Loading…</p>;
  if (loaded.state === "unknown")
    return (
      <Screen>
        <LinkProblem title="This link does not work" advice={ASK_AGAIN} />
      </Screen>
    );
  const { preview } = loaded;
  if (preview.state !== "pending") {
    const title =
      preview.state === "accepted"
        ? "This link was already used"
        : preview.state === "revoked"
          ? "This link was taken back"
          : "This link has expired";
    return (
      <Screen>
        <LinkProblem title={title} advice={ASK_AGAIN} />
      </Screen>
    );
  }

  const intro = (
    <Brand
      title={`Join ${preview.family_name}`}
      lead={
        <>
          You are invited {ROLE_LINE[preview.role]} The link is for{" "}
          <strong className="text-ink">{preview.email}</strong>.
        </>
      }
    />
  );

  if (me) {
    const sameEmail = me.user.email === preview.email;
    return (
      <Screen>
        {intro}
        {sameEmail ? (
          <Button size="lg" disabled={busy} onClick={() => void accept()}>
            Join {preview.family_name}
          </Button>
        ) : (
          <>
            <p className="m-0 rounded-control bg-surface-sunken px-4 py-3 text-[15px] leading-5 font-semibold text-muted">
              This link is for {preview.email}, but you are signed in as {me.user.email}. Sign out, then open the
              link again.
            </p>
            <Button variant="secondary" size="lg" onClick={() => void onSignOut()}>
              Sign out
            </Button>
          </>
        )}
        {error && <ErrorText>{error}</ErrorText>}
      </Screen>
    );
  }

  return (
    <Screen>
      {intro}
      {haveAccount ? (
        <SignInForm fixedEmail={preview.email} submitLabel="Sign in and join" onDone={accept} />
      ) : (
        <SignUpForm fixedEmail={preview.email} submitLabel="Create account and join" onDone={accept} />
      )}
      {error && <ErrorText>{error}</ErrorText>}
      <Button variant="quiet" onClick={() => setHaveAccount(!haveAccount)}>
        {haveAccount ? "Create an account instead" : "I already have an account"}
      </Button>
    </Screen>
  );
}
