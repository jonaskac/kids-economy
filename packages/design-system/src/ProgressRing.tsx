import { formatAmount, type Amount } from "./money";

export interface ProgressRingProps {
  label: string;
  saved: Amount;
  target: Amount;
  size?: number;
}

/** How far an Account is towards its Savings goal. */
export function ProgressRing({ label, saved, target, size = 72 }: ProgressRingProps) {
  const stroke = Math.round(size / 9);
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const percent = target.minor > 0 ? Math.min(100, Math.floor((Math.max(0, saved.minor) * 100) / target.minor)) : 0;
  const reached = target.minor > 0 && saved.minor >= target.minor;
  const left = { minor: Math.max(0, target.minor - saved.minor), currency: target.currency };
  const centre = size / 2;
  return (
    <div role="img" aria-label={`${label}: ${percent} percent saved`} className="inline-flex items-center gap-4">
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true" className="flex-none">
        <circle cx={centre} cy={centre} r={radius} strokeWidth={stroke} className="fill-none stroke-surface-sunken" />
        <circle
          cx={centre}
          cy={centre}
          r={radius}
          strokeWidth={stroke}
          strokeDasharray={circumference}
          strokeDashoffset={circumference * (1 - percent / 100)}
          strokeLinecap="round"
          transform={`rotate(-90 ${centre} ${centre})`}
          className="fill-none stroke-coin-strong transition-[stroke-dashoffset] duration-600 motion-reduce:transition-none"
        />
        <text
          x="50%"
          y="50%"
          dominantBaseline="central"
          textAnchor="middle"
          fontSize={Math.round(size / 4)}
          className="fill-ink font-display font-semibold"
        >
          {percent}%
        </text>
      </svg>
      <div className="flex flex-col gap-0.5">
        <span className="text-[19px] leading-[26px] font-bold">{label}</span>
        <span className="text-sm font-semibold text-muted">
          {formatAmount(saved)} of {formatAmount(target)}
        </span>
        {reached ? (
          <span className="mt-1 inline-block w-fit rounded-full bg-coin-soft px-3 py-0.5 text-sm font-extrabold text-coin-ink">
            Goal reached
          </span>
        ) : (
          <span className="text-sm font-semibold text-muted">{formatAmount(left)} to go</span>
        )}
      </div>
    </div>
  );
}
