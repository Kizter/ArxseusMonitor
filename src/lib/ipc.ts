import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { DEFAULT_SETTINGS, type Settings, type Snapshot, type TrayLabels } from "./types";

/**
 * Satu-satunya jembatan ke Rust. Di luar Tauri (mis. membuka URL Vite langsung)
 * fungsi ini mengembalikan nilai kosong, bukan data palsu.
 */
export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function getSnapshot(): Promise<Snapshot | null> {
  if (!inTauri()) return null;
  return invoke<Snapshot | null>("get_snapshot");
}

export async function getSettings(): Promise<Settings> {
  if (!inTauri()) return DEFAULT_SETTINGS;
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<Settings> {
  if (!inTauri()) return settings;
  return invoke<Settings>("save_settings", { settings });
}

export async function configureTray(labels: TrayLabels): Promise<void> {
  if (!inTauri()) return;
  await invoke<void>("configure_tray", { labels });
}

export async function setTrayTooltip(text: string): Promise<void> {
  if (!inTauri()) return;
  await invoke<void>("set_tray_tooltip", { text });
}

export async function onSnapshot(
  handler: (snapshot: Snapshot) => void,
): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  return listen<Snapshot>("snapshot", (event) => handler(event.payload));
}

export async function onOpenSettings(
  handler: () => void,
): Promise<UnlistenFn> {
  if (!inTauri()) return () => {};
  return listen("open-settings", () => handler());
}
