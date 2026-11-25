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

/** `/families/{familyId}/settings`: the Family's Kids and Members. */
export function settingsPath(familyId: string): string {
  return `/families/${familyId}/settings`;
}

/** Family settings; `/members` was its earlier path and still works. */
export function matchSettings(path: string): { familyId: string } | null {
  const match = /^\/families\/([^/]+)\/(?:settings|members)\/?$/.exec(path);
  return match?.[1] ? { familyId: match[1] } : null;
}

/** The secret of an Invitation link (`/join/<secret>`) or a Password reset link (`/reset/<secret>`). */
export function matchSecret(path: string, prefix: "join" | "reset"): string | null {
  const match = new RegExp(`^/${prefix}/([^/]+)/?$`).exec(path);
  return match?.[1] ? decodeURIComponent(match[1]) : null;
}

/** A link the server returned (`/join/…`), as a full URL to share. */
export function shareableUrl(link: string): string {
  return `${window.location.origin}${link}`;
}
