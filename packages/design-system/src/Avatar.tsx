export type KidColour = "sea" | "sun" | "berry" | "leaf" | "sky";

export const KID_COLOURS: readonly KidColour[] = ["sea", "sun", "berry", "leaf", "sky"];

const BACKGROUND: Record<KidColour, string> = {
  sea: "bg-kid-sea",
  sun: "bg-kid-sun",
  berry: "bg-kid-berry",
  leaf: "bg-kid-leaf",
  sky: "bg-kid-sky",
};

const SIZE = {
  sm: "size-8 text-[15px]",
  md: "size-12 text-[22px]",
  lg: "size-[72px] text-[34px]",
} as const;

export interface AvatarProps {
  name: string;
  colour?: KidColour;
  size?: keyof typeof SIZE;
}

/** A round initial in one of the five kid colours. */
export function Avatar({ name, colour = "sea", size = "md" }: AvatarProps) {
  const initial = name.trim().charAt(0).toUpperCase() || "?";
  return (
    <span
      role="img"
      aria-label={name}
      className={`inline-grid flex-none place-items-center rounded-full font-display font-semibold text-ink ${BACKGROUND[colour]} ${SIZE[size]}`}
    >
      <span aria-hidden="true">{initial}</span>
    </span>
  );
}
