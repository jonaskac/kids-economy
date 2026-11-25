import { useEffect, useId, type ReactNode } from "react";

export interface SheetProps {
  open: boolean;
  title: string;
  description?: string;
  onClose: () => void;
  children?: ReactNode;
  /** The main action, shown above Cancel. */
  actions?: ReactNode;
}

/** A dialog that rises from the bottom on phones and sits centred on wider screens. */
export function Sheet({ open, title, description, onClose, children, actions }: SheetProps) {
  const titleId = useId();
  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);
  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-10 flex items-end justify-center bg-scrim sm:items-center"
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="flex max-h-[92dvh] w-full max-w-[480px] flex-col gap-4 overflow-y-auto rounded-t-card bg-surface-raised px-4 pt-6 pb-[calc(16px+env(safe-area-inset-bottom,0px))] text-ink shadow-dialog sm:rounded-card sm:p-6"
      >
        <h2 id={titleId} className="m-0 font-display text-[22px] leading-7 font-semibold">
          {title}
        </h2>
        {description && <p className="m-0 text-muted">{description}</p>}
        {children}
        <div className="flex flex-col gap-3">
          {actions}
          <button
            type="button"
            onClick={onClose}
            className="inline-flex min-h-12 items-center justify-center rounded-control border-2 border-line-strong bg-surface-raised px-6 text-[17px] font-extrabold text-ink"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
