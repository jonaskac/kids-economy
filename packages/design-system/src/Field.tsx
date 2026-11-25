import { useId, type ChangeEvent, type InputHTMLAttributes } from "react";
import { currencySymbol } from "./money";

const CONTROL =
  "min-h-12 w-full rounded-field border-2 border-line-strong bg-surface-raised px-3 text-[19px] font-semibold text-ink aria-[invalid=true]:border-danger";

interface Common {
  label: string;
  hint?: string;
  error?: string;
}

function Messages({ id, hint, error }: { id: string; hint?: string | undefined; error?: string | undefined }) {
  return (
    <>
      {hint && (
        <span id={`${id}-hint`} className="text-sm font-semibold text-muted">
          {hint}
        </span>
      )}
      {error && (
        <span id={`${id}-error`} role="alert" className="rounded-field bg-danger-soft px-3 py-2 text-sm font-bold text-danger">
          {error}
        </span>
      )}
    </>
  );
}

function describedBy(id: string, hint?: string, error?: string): string | undefined {
  const ids = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean);
  return ids.length > 0 ? ids.join(" ") : undefined;
}

export interface TextFieldProps
  extends Common,
    Omit<InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "id"> {
  value: string;
  onChange: (value: string) => void;
}

export function TextField({ label, hint, error, value, onChange, className = "", ...input }: TextFieldProps) {
  const id = useId();
  return (
    <div className="flex flex-col gap-2">
      <label htmlFor={id} className="text-[15px] leading-5 font-bold">
        {label}
      </label>
      <input
        {...input}
        id={id}
        value={value}
        onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value)}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy(id, hint, error)}
        className={`${CONTROL} ${className}`.trim()}
      />
      <Messages id={id} hint={hint} error={error} />
    </div>
  );
}

export interface AmountFieldProps extends Common {
  currency: string;
  value: string;
  onChange: (value: string) => void;
}

/** Opens the decimal keypad; the parent parses with `parseAmount`, never `parseFloat`. */
export function AmountField({ label, hint, error, currency, value, onChange }: AmountFieldProps) {
  const id = useId();
  return (
    <div className="flex flex-col gap-2">
      <label htmlFor={id} className="text-[15px] leading-5 font-bold">
        {label}
      </label>
      <span className="relative flex items-center">
        <input
          id={id}
          inputMode="decimal"
          autoComplete="off"
          placeholder="0"
          value={value}
          onChange={(event) => onChange(event.target.value)}
          aria-invalid={error ? true : undefined}
          aria-describedby={describedBy(id, hint, error)}
          className={`${CONTROL} pr-14 font-display text-[28px] tabular-nums`}
        />
        <span aria-hidden="true" className="pointer-events-none absolute right-3 font-display text-[22px] font-semibold text-muted">
          {currencySymbol(currency)}
        </span>
      </span>
      <Messages id={id} hint={hint} error={error} />
    </div>
  );
}

export interface SelectFieldProps extends Common {
  value: string;
  onChange: (value: string) => void;
  options: readonly { value: string; label: string }[];
}

export function SelectField({ label, hint, error, value, onChange, options }: SelectFieldProps) {
  const id = useId();
  return (
    <div className="flex flex-col gap-2">
      <label htmlFor={id} className="text-[15px] leading-5 font-bold">
        {label}
      </label>
      <select
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy(id, hint, error)}
        className={CONTROL}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <Messages id={id} hint={hint} error={error} />
    </div>
  );
}
