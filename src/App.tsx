import type { UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { IntroCard } from "./components/IntroCard";
import { BackIcon, GearIcon } from "./components/Icons";
import { MetricCard } from "./components/MetricCard";
import { SettingsPanel } from "./components/SettingsPanel";
import { StatusBanner } from "./components/StatusBanner";
import { WhyPanel } from "./components/WhyPanel";
import {
  configureTray,
  getSettings,
  getSnapshot,
  onOpenSettings,
  onSnapshot,
  saveSettings,
  setTrayTooltip,
} from "./lib/ipc";
import {
  dictionary,
  formatBytes,
  formatPercent,
  formatTemperature,
  type Dictionary,
} from "./lib/i18n";
import {
  DEFAULT_SETTINGS,
  METRIC_ORDER,
  worstMetric,
  type DiskView,
  type Language,
  type Level,
  type Metric,
  type MetricKey,
  type Settings,
  type Snapshot,
} from "./lib/types";

type View = "detail" | "settings";

function formatMetric(metric: Metric | undefined, language: Language): string {
  if (!metric || metric.value === null) return "—";
  return metric.unit === "%"
    ? formatPercent(language, metric.value)
    : formatTemperature(language, metric.value);
}

function fullestDisk(disks: DiskView[]): DiskView | null {
  return disks.reduce<DiskView | null>(
    (acc, disk) => (acc === null || disk.freePct < acc.freePct ? disk : acc),
    null,
  );
}

function detailFor(
  key: MetricKey,
  snapshot: Snapshot,
  dict: Dictionary,
  language: Language,
): string | null {
  switch (key) {
    case "cpu":
      return snapshot.device.cpuCores !== null
        ? `${snapshot.device.cpuCores} ${dict.units.threads}`
        : (snapshot.device.cpuBrand ?? null);
    case "ram": {
      const { ramUsedBytes: used, ramTotalBytes: total } = snapshot.device;
      return used !== null && total !== null
        ? `${formatBytes(language, used)} ${dict.units.of} ${formatBytes(language, total)}`
        : null;
    }
    case "disk": {
      const disk = fullestDisk(snapshot.disks);
      return disk
        ? `${disk.mount} · ${formatBytes(language, disk.freeBytes)} ${dict.units.free}`
        : null;
    }
    case "cpuTemp":
      return sourceLabel(snapshot.cpuTempSource, dict);
    case "gpuTemp":
      return snapshot.device.gpuName ?? dict.footer.noTemp;
  }
}

function sourceLabel(
  source: Snapshot["cpuTempSource"],
  dict: Dictionary,
): string {
  switch (source) {
    case "lhm":
      return dict.footer.lhm;
    case "acpi":
      return dict.footer.acpi;
    case "nvml":
      return dict.footer.nvml;
    default:
      return dict.footer.noTemp;
  }
}

/** Satu kalimat ringkas untuk banner dan tooltip tray. */
function statusSummary(snapshot: Snapshot, dict: Dictionary): string {
  if (snapshot.overall === "unknown") return dict.banner.unknown;
  if (snapshot.overall === "safe") return dict.banner.safe;

  const worst = worstMetric(snapshot.metrics);
  return (
    dict.reason[snapshot.metrics[worst].reasonKey]?.title ??
    dict.banner[snapshot.overall]
  );
}

export default function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [ready, setReady] = useState(false);
  const [view, setView] = useState<View>("detail");
  const [selected, setSelected] = useState<MetricKey | null>(null);
  // Jumlah kegagalan simpan berturut-turut. Angka (bukan boolean) dipakai agar
  // kegagalan kedua tetap diumumkan ulang oleh `role="alert"`.
  const [saveError, setSaveError] = useState(0);
  const scrollerRef = useRef<HTMLElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const bannerRef = useRef<HTMLElement>(null);

  const dict = useMemo(
    () => dictionary(settings.language),
    [settings.language],
  );

  // Muat kondisi awal + berlangganan event dari Rust.
  useEffect(() => {
    let disposed = false;
    let stopSnapshot: UnlistenFn | undefined;
    let stopSettings: UnlistenFn | undefined;

    void (async () => {
      const [initialSnapshot, initialSettings, snapshotUnlisten, settingsUnlisten] =
        await Promise.all([
          getSnapshot(),
          getSettings(),
          onSnapshot((next) => setSnapshot(next)),
          onOpenSettings(() => setView("settings")),
        ]);

      if (disposed) {
        snapshotUnlisten();
        settingsUnlisten();
        return;
      }

      stopSnapshot = snapshotUnlisten;
      stopSettings = settingsUnlisten;

      if (initialSnapshot) setSnapshot(initialSnapshot);
      setSettings(initialSettings);
      setReady(true);
    })();

    return () => {
      disposed = true;
      stopSnapshot?.();
      stopSettings?.();
    };
  }, []);

  useEffect(() => {
    document.documentElement.lang = settings.language;
  }, [settings.language]);

  // Pindah tampilan: gulir ke atas dan pindahkan fokus ke judul. Tanpa ini
  // pengguna keyboard/pembaca layar tertinggal di posisi gulir dan fokus lama.
  // Saat jendela hanya hidup di tray, fokus tidak dipindahkan: memindahkan
  // fokus di dokumen yang tidak terlihat tidak ada gunanya bagi pengguna.
  useEffect(() => {
    scrollerRef.current?.scrollTo({ top: 0 });
    if (document.hasFocus()) headingRef.current?.focus();
  }, [view]);

  // Jendela dimunculkan lagi dari tray: WebView2 tidak memindahkan fokus sendiri.
  // Hanya lakukan bila fokus sedang tidak di mana-mana, supaya tidak merampas
  // fokus pengguna yang sedang di baris metrik atau formulir.
  //
  // `document.hasFocus()` dipakai, bukan `visibilityState`: WebView2 tetap
  // melaporkan `"visible"` meskipun jendela disembunyikan lewat WM_CLOSE
  // (diukur pada build release, 15 Nov), sehingga penjaga berbasis
  // `visibilityState` tidak pernah menahan apa pun.
  useEffect(() => {
    const focusHeadingIfIdle = () => {
      if (!document.hasFocus()) return;
      const active = document.activeElement;
      if (!active || active === document.body) headingRef.current?.focus();
    };

    window.addEventListener("focus", focusHeadingIfIdle);
    document.addEventListener("visibilitychange", focusHeadingIfIdle);
    return () => {
      window.removeEventListener("focus", focusHeadingIfIdle);
      document.removeEventListener("visibilitychange", focusHeadingIfIdle);
    };
  }, []);

  // Label menu tray mengikuti bahasa aktif.
  useEffect(() => {
    if (!ready) return;
    void configureTray({
      open: dict.tray.open,
      settings: dict.tray.settings,
      quit: dict.tray.quit,
    });
  }, [ready, dict]);

  // Tooltip tray: tiga baris sesuai DESIGN.md §8.
  useEffect(() => {
    if (!snapshot) return;

    const language = settings.language;
    const cpu = snapshot.metrics.cpu.value;
    const ram = snapshot.metrics.ram.value;
    const cpuTemp = snapshot.metrics.cpuTemp.value;
    const gpuTemp = snapshot.metrics.gpuTemp.value;

    const usage = [
      `${dict.tooltip.cpu} ${cpu === null ? "—" : formatPercent(language, cpu)}`,
      `${dict.tooltip.ram} ${ram === null ? "—" : formatPercent(language, ram)}`,
    ].join(" · ");

    const temperature =
      cpuTemp !== null
        ? `${dict.metric.cpuTemp.label} ${formatTemperature(language, cpuTemp)}`
        : gpuTemp !== null
          ? `${dict.metric.gpuTemp.label} ${formatTemperature(language, gpuTemp)}`
          : dict.tooltip.noTemp;

    void setTrayTooltip(
      `${dict.level[snapshot.overall]} — ${statusSummary(snapshot, dict)}\n${usage}\n${temperature}`,
    );
  }, [snapshot, dict, settings.language]);

  const updateSettings = useCallback(
    async (patch: Partial<Settings>) => {
      const next = { ...settings, ...patch };
      setSettings(next);

      try {
        const saved = await saveSettings(next);
        setSettings(saved);
        setSaveError(0);
      } catch (error) {
        console.error("gagal menyimpan pengaturan", error);
        setSettings(settings);
        setSaveError((attempt) => attempt + 1);
      }
    },
    [settings],
  );

  // Menutup kartu perkenalan juga menghapus tombolnya; tanpa pemindahan fokus
  // eksplisit fokus jatuh ke <body> dan pengguna keyboard/pembaca layar
  // kehilangan posisi. Banner status adalah isi terdekat yang bermakna.
  const dismissIntro = useCallback(async () => {
    await updateSettings({ introSeen: true });
    if (document.hasFocus()) bannerRef.current?.focus();
  }, [updateSettings]);

  const metrics = snapshot?.metrics;
  const overall: Level = snapshot?.overall ?? "unknown";
  const active: MetricKey = selected ?? (metrics ? worstMetric(metrics) : "cpu");
  const activeReason = metrics ? dict.reason[metrics[active].reasonKey] : undefined;

  // Banner selalu bicara tentang metrik terburuk, bukan kartu yang sedang
  // dipilih pengguna. Kalau tidak, banner bisa berteriak "WASPADA" sementara
  // kalimatnya menjelaskan kartu aman yang sedang dibuka.
  const worstReason = metrics
    ? dict.reason[metrics[worstMetric(metrics)].reasonKey]
    : undefined;

  const headline =
    overall === "unknown"
      ? dict.banner.unknown
      : overall === "safe"
        ? dict.banner.safe
        : (worstReason?.title ?? dict.banner[overall]);

  const advice = overall === "unknown" ? null : (worstReason?.advice ?? null);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between gap-3 border-b border-line px-3 py-2.5">
        {view === "detail" ? (
          <>
            <div className="min-w-0">
              <h1
                ref={headingRef}
                tabIndex={-1}
                className="text-base leading-tight font-semibold"
              >
                {dict.appName}
              </h1>
              <p className="mt-0.5 text-sm text-muted">{dict.tagline}</p>
            </div>
            <button
              type="button"
              onClick={() => setView("settings")}
              aria-label={dict.settings.title}
              title={dict.settings.title}
              className="shrink-0 rounded-md border border-line bg-overlay p-2 text-muted transition-colors duration-[80ms] hover:bg-line hover:text-ink"
            >
              <GearIcon className="h-4 w-4" />
            </button>
          </>
        ) : (
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setView("detail")}
              className="flex items-center gap-1 rounded-md border border-line bg-overlay px-2 py-1 text-sm text-ink transition-colors duration-[80ms] hover:bg-line"
            >
              <BackIcon className="h-4 w-4" />
              {dict.settings.back}
            </button>
            <h1
              ref={headingRef}
              tabIndex={-1}
              className="text-base font-semibold"
            >
              {dict.settings.title}
            </h1>
          </div>
        )}
      </header>

      <main ref={scrollerRef} className="flex-1 overflow-y-auto">
        {view === "settings" ? (
          <SettingsPanel
            dict={dict}
            settings={settings}
            saveError={saveError}
            onChange={(patch) => void updateSettings(patch)}
          />
        ) : (
          <div className="flex flex-col gap-3 px-3 py-3">
            <StatusBanner
              ref={bannerRef}
              level={overall}
              statusWord={dict.level[overall]}
              headline={headline}
              advice={advice}
            />

            {ready && !settings.introSeen ? (
              <IntroCard
                dict={dict}
                onDismiss={() => void dismissIntro()}
                onOpenSettings={() => setView("settings")}
              />
            ) : null}

            {/* Heading tak terlihat: tanpa ini navigasi per heading melewati
                seluruh isi utama jendela (lima baris metrik). */}
            <section aria-labelledby="metrics-heading">
              <h2 id="metrics-heading" className="sr-only">
                {dict.metricsHeading}
              </h2>

              <div className="divide-y divide-line-muted overflow-hidden rounded-md border border-line bg-canvas">
                {METRIC_ORDER.map((key) => {
                  const metric = metrics?.[key];
                  const level: Level = metric?.level ?? "unknown";

                  return (
                    <MetricCard
                      key={key}
                      label={dict.metric[key].label}
                      value={formatMetric(metric, settings.language)}
                      detail={
                        snapshot
                          ? detailFor(key, snapshot, dict, settings.language)
                          : null
                      }
                      level={level}
                      levelWord={dict.level[level]}
                      bar={
                        metric?.unit === "%" && metric.value !== null
                          ? metric.value
                          : null
                      }
                      selected={active === key}
                      onSelect={() => setSelected(key)}
                    />
                  );
                })}
              </div>
            </section>

            {metrics ? (
              <WhyPanel
                dict={dict}
                metricLabel={dict.metric[active].label}
                why={dict.metric[active].why}
                reasonTitle={activeReason?.title ?? dict.banner.unknown}
                advice={activeReason?.advice ?? null}
              />
            ) : (
              <p
                id="why-panel"
                className="rounded-md border border-line bg-surface px-3 py-2.5 text-sm text-muted"
              >
                {dict.why.empty}
              </p>
            )}
          </div>
        )}
      </main>

      {view === "detail" ? (
        <footer className="border-t border-line px-3 py-1.5 text-sm leading-snug text-muted">
          <p className="text-sm">
            {snapshot
              ? dict.footer.updated(
                  Math.max(0, Math.round((Date.now() - snapshot.at) / 1000)),
                )
              : dict.banner.unknown}
          </p>
          <p>
            {dict.footer.tempSource}:{" "}
            <span className="font-mono">
              {sourceLabel(snapshot?.cpuTempSource ?? null, dict)}
            </span>
          </p>
        </footer>
      ) : null}
    </div>
  );
}
