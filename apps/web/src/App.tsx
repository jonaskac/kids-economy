import { useCallback, useEffect, useState } from "react";
import { api, ApiFailure, type Me } from "./api";
import { matchWallet, navigate, usePath } from "./router";
import { FamilyHome } from "./screens/FamilyHome";
import { KidWallet } from "./screens/KidWallet";
import { SignIn } from "./screens/SignIn";

type Session = { state: "loading" } | { state: "signed-out" } | { state: "signed-in"; me: Me };

/** The Parent app: sign in, then a Family's Kids, then one Kid's Wallet. */
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
  if (session.state === "signed-out") {
    return <SignIn onSignedIn={load} />;
  }

  const signOut = async () => {
    await api.signOut();
    navigate("/");
    setSession({ state: "signed-out" });
  };

  const wallet = matchWallet(path);
  if (wallet) {
    return <KidWallet familyId={wallet.familyId} walletId={wallet.walletId} />;
  }
  return <FamilyHome me={session.me} onSignOut={signOut} />;
}
