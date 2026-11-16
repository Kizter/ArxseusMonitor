/** Cermin dari `src-tauri/src/monitor/model.rs` dan `src-tauri/src/settings.rs`. */

export type Level = "safe" | "warn" | "danger" | "unknown";

export type Metric = {
  value: number | null;
  unit: string;
  level: Level;
  reasonKey: string;
};

export type MetricKey = "cpu" | "ram" | "disk" | "cpuTemp" | "gpuTemp";

export type Metrics = Record<MetricKey, Metric>;

export type DeviceInfo = {
  cpuBrand: string | null;
  cpuCores: number | null;
  gpuName: string | null;
  ramUsedBytes: number | null;
  ramTotalBytes: number | null;
  uptimeSecs: number | null;
};

export type DiskView = {
  name: string;
  mount: string;
  totalBytes: number;
  freeBytes: number;
  freePct: number;
};

export type Snapshot = {
  at: number;
  overall: Level;
  metrics: Metrics;
  device: DeviceInfo;
  disks: DiskView[];
  cpuTempSource: "lhm" | "acpi" | "nvml" | null;
  intervalMs: number;
};

export type Language = "id" | "en";

export type TempPreference = "auto" | "acpi" | "lhm" | "off";

export type Settings = {
  language: Language;
  intervalSecs: number;
  temperatureSource: TempPreference;
  startMinimized: boolean;
  introSeen: boolean;
};

export type TrayLabels = {
  open: string;
  settings: string;
  quit: string;
};

export const DEFAULT_SETTINGS: Settings = {
  language: "id",
  intervalSecs: 2,
  temperatureSource: "auto",
  startMinimized: false,
  introSeen: false,
};

export const METRIC_ORDER: MetricKey[] = [
  "cpu",
  "ram",
  "disk",
  "cpuTemp",
  "gpuTemp",
];

export const LEVEL_RANK: Record<Level, number> = {
  unknown: 0,
  safe: 1,
  warn: 2,
  danger: 3,
};

/** Metrik terburuk — dipakai untuk memilih panel "Kenapa?" pertama. */
export function worstMetric(metrics: Metrics): MetricKey {
  let worst: MetricKey | null = null;

  for (const key of METRIC_ORDER) {
    if (metrics[key].level === "unknown") continue;
    if (
      worst === null ||
      LEVEL_RANK[metrics[key].level] > LEVEL_RANK[metrics[worst].level]
    ) {
      worst = key;
    }
  }

  return worst ?? "cpu";
}
