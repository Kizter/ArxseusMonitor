import { levelBar, levelPill } from "../lib/levelStyles";
import type { Level } from "../lib/types";

type Props = {
  label: string;
  value: string;
  detail: string | null;
  level: Level;
  levelWord: string;
  bar: number | null;
  selected: boolean;
  onSelect: () => void;
};

/**
 * Satu baris metrik bergaya daftar GitHub: padat, bergaris rambut, angka
 * monospace, dan pil status di kanan. Baris dipilih memakai garis biru di tepi
 * kiri, bukan kartu terpisah dengan bayangan.
 */
export function MetricCard({
  label,
  value,
  detail,
  level,
  levelWord,
  bar,
  selected,
  onSelect,
}: Props) {
  // Detail (mis. "16 thread", "4,2 GB dari 16 GB") sebelumnya hanya terlihat,
  // tidak pernah dibacakan. Masukkan ke nama aksesibel supaya pembaca layar
  // mendapat informasi yang sama dengan pengguna visual.
  const accessibleName = [`${label}: ${value}`, detail, levelWord]
    .filter((part): part is string => Boolean(part))
    .join(", ");

  return (
    <button
      type="button"
      onClick={onSelect}
      // `aria-pressed` tidak dipakai: pilihan tidak bisa dibatalkan (selalu ada
      // satu baris aktif), jadi `aria-current` lebih jujur. `aria-controls`
      // menautkan baris ke panel "Kenapa?" yang isinya ikut berubah.
      aria-current={selected ? "true" : undefined}
      aria-controls="why-panel"
      aria-label={accessibleName}
      className={`w-full px-3 py-2 text-left transition-colors duration-[80ms] focus-visible:outline-offset-[-2px] ${
        selected
          ? "bg-surface shadow-[inset_3px_0_0_0_var(--color-accent)]"
          : "hover:bg-surface"
      }`}
    >
      <span className="flex items-baseline justify-between gap-3">
        <span className={`text-sm ${selected ? "font-semibold" : ""}`}>
          {label}
        </span>
        <span className="font-mono text-base font-semibold tabular-nums">
          {value}
        </span>
      </span>

      <span className="mt-0.5 flex items-start justify-between gap-3">
        <span className="text-sm text-muted">{detail ?? "\u00A0"}</span>
        <span
          className={`shrink-0 rounded-full border px-2 py-px text-sm ${levelPill[level]}`}
        >
          {levelWord}
        </span>
      </span>

      {bar !== null ? (
        <span
          aria-hidden="true"
          className="mt-1.5 block h-1 w-full overflow-hidden rounded-full bg-line"
        >
          <span
            className={`block h-full rounded-full ${levelBar[level]}`}
            style={{ width: `${Math.min(100, Math.max(0, bar))}%` }}
          />
        </span>
      ) : null}
    </button>
  );
}
