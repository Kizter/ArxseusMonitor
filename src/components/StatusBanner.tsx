import type { Ref } from "react";

import { levelBanner, levelText } from "../lib/levelStyles";
import type { Level } from "../lib/types";
import { LevelGlyph } from "./Icons";

type Props = {
  level: Level;
  statusWord: string;
  headline: string;
  advice: string | null;
  /** Target fokus setelah kartu perkenalan ditutup (lihat App.tsx). */
  ref?: Ref<HTMLElement>;
};

/**
 * Ringkasan satu pandangan: warna, kata status, kalimat sebab, dan satu saran.
 * `role="status"` sendiri sudah berarti `aria-live="polite"` + `aria-atomic`,
 * jadi atribut `aria-live` tidak diulang. Teks di sini hanya berubah saat level
 * berubah (bukan setiap 2 detik), jadi tidak berisik untuk pembaca layar.
 */
export function StatusBanner({
  level,
  statusWord,
  headline,
  advice,
  ref,
}: Props) {
  return (
    <section
      ref={ref}
      id="status-banner"
      role="status"
      // Fokus dipindahkan ke sini setelah kartu perkenalan ditutup; tanpa
      // `tabIndex=-1` elemen ini tidak bisa menerima fokus programatik.
      tabIndex={-1}
      className={`rounded-md border p-3 ${levelBanner[level]}`}
    >
      <div className="flex items-center gap-2">
        <LevelGlyph level={level} className={`h-4 w-4 ${levelText[level]}`} />
        <span className={`text-sm font-semibold ${levelText[level]}`}>
          {statusWord}
        </span>
      </div>

      <h2 className="mt-1.5 text-base leading-snug font-semibold">
        {headline}
      </h2>

      {advice ? <p className="mt-1 text-sm text-muted">{advice}</p> : null}
    </section>
  );
}
