import type { Level } from "./types";

/**
 * Kelas Tailwind ditulis lengkap (bukan disusun dari string) agar tetap
 * terdeteksi saat build. Hijau/kuning/merah dipakai hanya untuk status
 * kesehatan, tidak untuk tombol atau tautan.
 */
export const levelText: Record<Level, string> = {
  safe: "text-safe",
  warn: "text-warn",
  danger: "text-danger",
  unknown: "text-muted",
};

export const levelBar: Record<Level, string> = {
  safe: "bg-safe",
  warn: "bg-warn",
  danger: "bg-danger",
  unknown: "bg-muted",
};

/** Pil status bergaya label GitHub: latar tipis, teks berwarna, garis tepi. */
export const levelPill: Record<Level, string> = {
  safe: "border-safe/40 bg-safe/10 text-safe",
  warn: "border-warn/40 bg-warn/10 text-warn",
  danger: "border-danger/40 bg-danger/10 text-danger",
  unknown: "border-line bg-overlay text-muted",
};

/** Kotak peringatan bergaya alert GitHub. */
export const levelBanner: Record<Level, string> = {
  safe: "border-safe/40 bg-safe/10",
  warn: "border-warn/40 bg-warn/10",
  danger: "border-danger/40 bg-danger/10",
  unknown: "border-line bg-surface",
};
