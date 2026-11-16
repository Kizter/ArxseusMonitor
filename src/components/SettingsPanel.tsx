import { useEffect, useRef } from "react";

import type { Dictionary } from "../lib/i18n";
import type { Language, Settings, TempPreference } from "../lib/types";

type Props = {
  dict: Dictionary;
  settings: Settings;
  /** Berapa kali penyimpanan gagal berturut-turut; 0 = tidak ada galat. */
  saveError: number;
  onChange: (patch: Partial<Settings>) => void;
};

// `lang` dipasang pada tiap opsi supaya nama bahasa diucapkan dengan fonetik
// bahasanya sendiri, bukan fonetik bahasa UI yang sedang aktif.
const LANGUAGES: { value: Language; label: string; lang: Language }[] = [
  { value: "id", label: "Bahasa Indonesia", lang: "id" },
  { value: "en", label: "English", lang: "en" },
];

const TEMPERATURE_SOURCES: TempPreference[] = ["auto", "acpi", "lhm", "off"];

const FIELD =
  "mt-1.5 w-full rounded-md border border-line-strong bg-canvas px-2.5 py-1.5 text-sm text-ink transition-colors duration-[80ms] focus:border-accent focus:ring-2 focus:ring-accent focus:outline-none";

const ROW = "border-b border-line-muted px-3 py-3 last:border-b-0";

export function SettingsPanel({ dict, settings, saveError, onChange }: Props) {
  const errorRef = useRef<HTMLParagraphElement>(null);

  // Pesan galat ada di dasar panel yang bisa digulir; tanpa ini pengguna
  // keyboard hanya mendengar "gagal disimpan" tanpa tahu di mana pesannya.
  useEffect(() => {
    if (saveError > 0) {
      errorRef.current?.scrollIntoView({ block: "nearest" });
    }
  }, [saveError]);

  const temperatureLabel: Record<TempPreference, string> = {
    auto: dict.settings.temperatureAuto,
    acpi: dict.settings.temperatureAcpi,
    lhm: dict.settings.temperatureLhm,
    off: dict.settings.temperatureOff,
  };

  const intervalLabel: Record<number, string> = {
    1: dict.settings.intervalFast,
    2: dict.settings.intervalBalanced,
    5: dict.settings.intervalLow,
  };

  return (
    <div className="flex flex-col">
      <div className={ROW}>
        <label className="text-sm font-semibold" htmlFor="setting-language">
          {dict.settings.language}
        </label>
        <select
          id="setting-language"
          className={FIELD}
          value={settings.language}
          onChange={(event) =>
            onChange({ language: event.target.value as Language })
          }
        >
          {LANGUAGES.map((option) => (
            <option key={option.value} value={option.value} lang={option.lang}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      <div className={ROW}>
        <label className="text-sm font-semibold" htmlFor="setting-interval">
          {dict.settings.interval}
        </label>
        <select
          id="setting-interval"
          className={FIELD}
          value={settings.intervalSecs}
          onChange={(event) =>
            onChange({ intervalSecs: Number(event.target.value) })
          }
        >
          {[1, 2, 5].map((seconds) => (
            <option key={seconds} value={seconds}>
              {intervalLabel[seconds]}
            </option>
          ))}
        </select>
      </div>

      <div className={ROW}>
        <label className="text-sm font-semibold" htmlFor="setting-temperature">
          {dict.settings.temperature}
        </label>
        <select
          id="setting-temperature"
          className={FIELD}
          aria-describedby="setting-temperature-note"
          value={settings.temperatureSource}
          onChange={(event) =>
            onChange({
              temperatureSource: event.target.value as TempPreference,
            })
          }
        >
          {TEMPERATURE_SOURCES.map((source) => (
            <option key={source} value={source}>
              {temperatureLabel[source]}
            </option>
          ))}
        </select>
        <p id="setting-temperature-note" className="mt-1.5 text-sm text-muted">
          {dict.settings.temperatureNote}
        </p>
      </div>

      <div className={ROW}>
        <label
          className="flex min-h-6 items-center gap-2.5 text-sm font-semibold"
          htmlFor="setting-start-minimized"
        >
          <input
            id="setting-start-minimized"
            type="checkbox"
            className="h-5 w-5 accent-accent"
            aria-describedby="setting-start-minimized-hint"
            checked={settings.startMinimized}
            onChange={(event) =>
              onChange({ startMinimized: event.target.checked })
            }
          />
          {dict.settings.startMinimized}
        </label>
        <p
          id="setting-start-minimized-hint"
          className="mt-1.5 text-sm text-muted"
        >
          {dict.settings.startMinimizedHint}
        </p>
      </div>

      {saveError > 0 ? (
        // `key` membuat elemen dipasang ulang tiap kegagalan, sehingga
        // `role="alert"` mengumumkan ulang kegagalan kedua dan seterusnya.
        <p
          key={saveError}
          ref={errorRef}
          role="alert"
          className="m-3 rounded-md border border-danger/40 bg-danger/10 px-2.5 py-1.5 text-sm text-danger"
        >
          {dict.settings.saveError}
        </p>
      ) : null}
    </div>
  );
}
