import type { Dictionary } from "../lib/i18n";

type Props = {
  dict: Dictionary;
  onDismiss: () => void;
  onOpenSettings: () => void;
};

/**
 * Penjelasan singkat saat pertama kali dibuka. Bukan wizard: satu layar,
 * satu tombol, dan bisa dibuka lagi lewat pengaturan bila perlu.
 */
export function IntroCard({ dict, onDismiss, onOpenSettings }: Props) {
  return (
    <section className="rounded-md border border-accent/40 bg-accent/10 p-3">
      <h2 className="text-base font-semibold">{dict.intro.title}</h2>
      <p className="mt-1 text-sm text-muted">{dict.intro.body}</p>

      <ul className="mt-2.5 list-disc space-y-1 pl-5 marker:text-muted">
        {dict.intro.points.map((point) => (
          <li key={point} className="text-sm">
            {point}
          </li>
        ))}
      </ul>

      <div className="mt-3 flex flex-wrap gap-2">
        <button
          type="button"
          onClick={onDismiss}
          className="rounded-md border border-accent-emphasis bg-accent-emphasis px-3 py-1.5 text-sm font-semibold text-white transition-colors duration-[80ms] hover:border-accent-emphasis-hover hover:bg-accent-emphasis-hover"
        >
          {dict.intro.dismiss}
        </button>
        <button
          type="button"
          onClick={onOpenSettings}
          className="rounded-md border border-line bg-overlay px-3 py-1.5 text-sm text-ink transition-colors duration-[80ms] hover:bg-line"
        >
          {dict.intro.openSettings}
        </button>
      </div>
    </section>
  );
}
