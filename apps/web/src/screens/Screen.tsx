import type { ReactNode } from "react";

/** One phone-first column with safe-area gutters; grows to 640 px. */
export function Screen({ children }: { children: ReactNode }) {
  return (
    <main className="mx-auto flex min-h-dvh max-w-[640px] flex-col gap-6 px-[max(16px,env(safe-area-inset-left,0px))] pt-4 pb-[calc(32px+env(safe-area-inset-bottom,0px))]">
      {children}
    </main>
  );
}
