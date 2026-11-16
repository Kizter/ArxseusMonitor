import type { Dictionary } from "../lib/i18n";

type Props = {
  dict: Dictionary;
  metricLabel: string;
  why: string;
  reasonTitle: string;
  advice: string | null;
};

/**
 * Panel penjelasan bergaya kotak GitHub: strip judul di atas, isi padat.
 * Di sinilah materi Arsitektur Komputer disampaikan dengan bahasa sehari-hari.
 * `id="why-panel"` adalah target `aria-controls` dari baris metrik, sehingga
 * hubungan baris ↔ penjelasan terbaca oleh pembaca layar.
 */
export function WhyPanel({ dict, metricLabel, why, reasonTitle, advice }: Props) {
  return (
    <section
      id="why-panel"
      aria-labelledby="why-heading"
      className="overflow-hidden rounded-md border border-line"
    >
      <h2
        id="why-heading"
        className="border-b border-line bg-surface px-3 py-1.5 text-sm font-semibold"
      >
        {dict.why.title}
      </h2>

      <div className="px-3 py-2.5">
        <p className="text-sm font-semibold">{metricLabel}</p>
        <p className="mt-1 text-sm text-muted">{why}</p>

        <div className="mt-2.5 border-t border-line-muted pt-2.5">
          <p className="text-sm">{reasonTitle}</p>
          {advice ? <p className="mt-0.5 text-sm text-muted">{advice}</p> : null}
        </div>
      </div>
    </section>
  );
}
