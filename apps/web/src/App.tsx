import { useCallback, useEffect, useState } from "react";
import { api, ApiFailure, type Me } from "./api";
import { matchSecret, matchSettings, matchWallet, navigate, usePath } from "./router";
import { FamilyHome } from "./screens/FamilyHome";
import { FamilySettings } from "./screens/FamilySettings";
import { JoinFamily } from "./screens/JoinFamily";
import { KidWallet } from "./screens/KidWallet";
import { ResetPassword } from "./screens/ResetPassword";
import { SignIn } from "./screens/SignIn";
import { SignUp } from "./screens/SignUp";

type Session = { state: "loading" } | { state: "signed-out" } | { state: "signed-in"; me: Me };

/**
 * The Parent app: sign in or up, then a Family's Kids, then one Kid's Wallet.
 * Invitation links (`/join/…`) and Password reset links (`/reset/…`) open with or without a Session.
 */
export function App() {
  const path = usePath();
  const [session, setSession] = useState<Session>({ state: "loading" });

  const load = useCallback(async () => {
    try {
      setSession({ state: "signed-in", me: await api.me() });
    } catch (error) {
      if (error instanceof ApiFailure && error.status === 401) setSession({ state: "signed-out" });
      else throw error;
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  if (session.state === "loading") {
    return <p className="p-4 text-muted">Loading…</p>;
  }

  const endSession = async () => {
    await api.signOut();
    setSession({ state: "signed-out" });
  };
  const goHome = async () => {
    navigate("/");
    await load();
  };

  const reset = matchSecret(path, "reset");
  if (reset) return <ResetPassword secret={reset} onSignedIn={goHome} />;
  const join = matchSecret(path, "join");
  if (join) {
    return (
      <JoinFamily
        secret={join}
        me={session.state === "signed-in" ? session.me : null}
        onJoined={goHome}
        onSignOut={endSession}
      />
    );
  }

  if (session.state === "signed-out") {
    return path === "/sign-up" ? <SignUp onSignedIn={load} /> : <SignIn onSignedIn={load} />;
  }

  const signOut = async () => {
    await endSession();
    navigate("/");
  };

  const wallet = matchWallet(path);
  if (wallet) {
    return <KidWallet familyId={wallet.familyId} walletId={wallet.walletId} />;
  }
  const settings = matchSettings(path);
  if (settings) {
    return <FamilySettings familyId={settings.familyId} />;
  }
  return <FamilyHome me={session.me} onSignOut={signOut} onFamilyStarted={load} />;
}
