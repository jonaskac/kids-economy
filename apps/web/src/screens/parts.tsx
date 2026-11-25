import { useId, useState, type ReactNode } from "react";
import { Button } from "@kids-economy/design-system";
import { navigate } from "../router";

/** A message the app needs the adult to read now. */
export function ErrorText({ children }: { children: ReactNode }) {
  return (
    <p role="alert" className="m-0 rounded-field bg-danger-soft px-3 py-2 text-sm font-bold text-danger">
      {children}
    </p>
  );
}

/** A link inside the app: changes the path without reloading. */
export function AppLink({ to, children, className = "" }: { to: string; children: ReactNode; className?: string }) {
  return (
    <a
      href={to}
      onClick={(event) => {
        event.preventDefault();
        navigate(to);
      }}
      className={`inline-flex min-h-11 items-center font-extrabold text-brand ${className}`.trim()}
    >
      {children}
    </a>
  );
}

/** "‹ Family": back to the Family home. */
export function BackToFamily() {
  return (
    <AppLink to="/" className="gap-1 pr-2 text-[15px] leading-5 no-underline">
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
        <path d="m15 18-6-6 6-6" />
      </svg>
      Family
    </AppLink>
  );
}

/** The app's name with its coin mark, at the top of the screens before sign-in. */
export function Brand({ title, lead }: { title: string; lead?: ReactNode }) {
  return (
    <div className="flex flex-col gap-2 pt-12">
      <span aria-hidden="true" className="flex gap-1.5">
        <span className="h-3 w-10 rounded-full bg-coin-strong" />
        <span className="h-3 w-10 rounded-full bg-coin" />
        <span className="size-3 rounded-full bg-brand" />
      </span>
      <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">{title}</h1>
      {lead && <p className="m-0 text-muted">{lead}</p>}
    </div>
  );
}

/** A link to share, shown once, with a button that copies it. */
export function CopyLink({ label, url }: { label: string; url: string }) {
  const id = useId();
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  };
  return (
    <div className="flex flex-col gap-2">
      <label htmlFor={id} className="text-[15px] leading-5 font-bold">
        {label}
      </label>
      <input
        id={id}
        readOnly
        value={url}
        onFocus={(event) => event.target.select()}
        className="min-h-12 w-full rounded-field border-2 border-line-strong bg-surface-sunken px-3 text-[15px] font-semibold text-ink"
      />
      <Button variant="secondary" onClick={() => void copy()}>
        {copied ? "Copied" : "Copy link"}
      </Button>
    </div>
  );
}

/** A screen that only says why a link cannot be used. */
export function LinkProblem({ title, advice }: { title: string; advice: string }) {
  return (
    <>
      <Brand title={title} lead={advice} />
      <AppLink to="/">Go to sign in</AppLink>
    </>
  );
}

export const MIN_PASSWORD = 10;
export const PASSWORD_HINT = `At least ${MIN_PASSWORD} characters. A few words in a row work well.`;
