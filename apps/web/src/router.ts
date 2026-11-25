import { useEffect, useState } from "react";

/** The current path; changes on `navigate` and on the browser's back button. */
export function usePath(): string {
  const [path, setPath] = useState(() => window.location.pathname);
  useEffect(() => {
    const update = () => setPath(window.location.pathname);
    window.addEventListener("popstate", update);
    return () => window.removeEventListener("popstate", update);
  }, []);
  return path;
}

export function navigate(path: string): void {
  window.history.pushState(null, "", path);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

/** `/families/{familyId}/kids/{walletId}`: one Kid's Wallet. */
export function walletPath(familyId: string, walletId: string): string {
  return `/families/${familyId}/wallets/${walletId}`;
}

export function matchWallet(path: string): { familyId: string; walletId: string } | null {
  const match = /^\/families\/([^/]+)\/wallets\/([^/]+)\/?$/.exec(path);
  if (!match?.[1] || !match[2]) return null;
  return { familyId: match[1], walletId: match[2] };
}
