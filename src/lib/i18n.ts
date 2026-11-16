import type { Language, Level, MetricKey } from "./types";

/**
 * Semua teks yang dilihat pengguna ada di sini. Rust hanya mengirim `reasonKey`,
 * jadi menambah bahasa berarti menambah satu objek di file ini.
 */

export type ReasonText = {
  title: string;
  advice: string;
};

export type Dictionary = {
  appName: string;
  tagline: string;
  /** Heading tak terlihat untuk grup baris metrik (navigasi per heading). */
  metricsHeading: string;
  level: Record<Level, string>;
  banner: Record<Level, string>;
  metric: Record<MetricKey, { label: string; why: string }>;
  reason: Record<string, ReasonText>;
  why: { title: string; empty: string };
  settings: {
    title: string;
    back: string;
    language: string;
    interval: string;
    intervalFast: string;
    intervalBalanced: string;
    intervalLow: string;
    temperature: string;
    temperatureAuto: string;
    temperatureAcpi: string;
    temperatureLhm: string;
    temperatureOff: string;
    temperatureNote: string;
    startMinimized: string;
    startMinimizedHint: string;
    saveError: string;
  };
  footer: {
    updated: (seconds: number) => string;
    tempSource: string;
    noTemp: string;
    lhm: string;
    acpi: string;
    nvml: string;
  };
  intro: {
    title: string;
    body: string;
    points: string[];
    dismiss: string;
    openSettings: string;
  };
  tray: { open: string; settings: string; quit: string };
  units: { free: string; of: string; threads: string };
  tooltip: { cpu: string; ram: string; noTemp: string; unknown: string };
};

const id: Dictionary = {
  appName: "ArxseusMonitor",
  tagline: "Kesehatan komputer, tanpa angka rumit",
  metricsHeading: "Ringkasan metrik",
  level: {
    safe: "Aman",
    warn: "Waspada",
    danger: "Bahaya",
    unknown: "Menunggu data",
  },
  banner: {
    safe: "Semua bagian yang terpantau dalam batas aman.",
    warn: "Ada satu bagian yang perlu dipantau.",
    danger: "Ada bagian yang perlu ditangani sekarang.",
    unknown: "Data hardware belum terbaca.",
  },
  metric: {
    cpu: {
      label: "Beban CPU",
      why: "CPU adalah otak komputer. Beban tinggi sebentar saat membuka aplikasi itu normal; yang menjadi masalah adalah bila tinggi terus-menerus.",
    },
    ram: {
      label: "Memori (RAM)",
      why: "RAM adalah meja kerja komputer. Makin banyak aplikasi terbuka, makin penuh mejanya, lalu komputer memindahkan data ke disk yang jauh lebih lambat.",
    },
    disk: {
      label: "Ruang penyimpanan",
      why: "Windows dan aplikasi butuh ruang kosong untuk bekerja. Bila terlalu penuh, penyimpanan melambat dan pembaruan Windows bisa gagal.",
    },
    cpuTemp: {
      label: "Suhu CPU",
      why: "Komponen yang terlalu panas menurunkan kecepatannya sendiri (throttling) agar tidak rusak. Itu sebabnya komputer terasa lambat saat panas.",
    },
    gpuTemp: {
      label: "Suhu kartu grafis",
      why: "Kartu grafis memanas saat mengolah gambar, terutama saat bermain game. Panas berlebih dalam waktu lama memperpendek umur komponen.",
    },
  },
  reason: {
    "cpu.ok": {
      title: "Beban CPU wajar.",
      advice: "Tidak ada tindakan yang diperlukan.",
    },
    "cpu.high": {
      title: "CPU cukup sibuk.",
      advice:
        "Tutup aplikasi yang tidak sedang dipakai kalau komputer terasa lambat.",
    },
    "cpu.critical": {
      title: "CPU bekerja sangat keras.",
      advice:
        "Tutup aplikasi berat, lalu periksa apakah ada proses yang tidak dikenal.",
    },
    "cpu.unavailable": {
      title: "Beban CPU belum terbaca.",
      advice: "Tunggu beberapa detik; bila tetap kosong, mulai ulang aplikasi.",
    },
    "ram.ok": {
      title: "Memori masih lega.",
      advice: "Tidak ada tindakan yang diperlukan.",
    },
    "ram.high": {
      title: "Memori hampir penuh.",
      advice: "Tutup tab browser atau aplikasi yang tidak dipakai.",
    },
    "ram.critical": {
      title: "Memori sangat penuh.",
      advice:
        "Simpan pekerjaan, tutup aplikasi, lalu mulai ulang komputer bila perlu.",
    },
    "ram.unavailable": {
      title: "Pemakaian memori belum terbaca.",
      advice: "Tunggu beberapa detik; bila tetap kosong, mulai ulang aplikasi.",
    },
    "disk.ok": {
      title: "Ruang penyimpanan masih cukup.",
      advice: "Tidak ada tindakan yang diperlukan.",
    },
    "disk.low": {
      title: "Ruang penyimpanan menipis.",
      advice: "Hapus file besar atau pindahkan ke drive lain.",
    },
    "disk.critical": {
      title: "Ruang penyimpanan hampir habis.",
      advice:
        "Segera kosongkan minimal 10% ruang agar Windows tidak melambat.",
    },
    "disk.unavailable": {
      title: "Ruang penyimpanan tidak terdeteksi.",
      advice: "Periksa apakah drive terpasang dengan benar.",
    },
    "temp.cpu.ok": {
      title: "Suhu CPU normal.",
      advice: "Tidak ada tindakan yang diperlukan.",
    },
    "temp.cpu.warm": {
      title: "CPU mulai hangat.",
      advice: "Pastikan ventilasi tidak tertutup dan kipas tidak berdebu.",
    },
    "temp.cpu.critical": {
      title: "CPU terlalu panas.",
      advice:
        "Hentikan tugas berat dan bersihkan kipas. Panas berlebih membuat komputer melambat.",
    },
    "temp.cpu.unavailable": {
      title: "Suhu CPU tidak terdeteksi di perangkat ini.",
      advice:
        "Ini keterbatasan Windows tanpa driver tambahan, bukan tanda kerusakan.",
    },
    "temp.gpu.ok": {
      title: "Suhu kartu grafis normal.",
      advice: "Tidak ada tindakan yang diperlukan.",
    },
    "temp.gpu.warm": {
      title: "Kartu grafis mulai hangat.",
      advice: "Kurangi pengaturan grafis berat atau beri jeda sejenak.",
    },
    "temp.gpu.critical": {
      title: "Kartu grafis terlalu panas.",
      advice:
        "Hentikan game atau aplikasi grafis berat, lalu periksa aliran udara di dalam casing.",
    },
    "temp.gpu.unavailable": {
      title: "Suhu kartu grafis tidak terdeteksi.",
      advice:
        "Tanpa kartu grafis NVIDIA, suhu ini memang tidak tersedia. Bukan tanda kerusakan.",
    },
  },
  why: {
    title: "Kenapa?",
    empty:
      "Data hardware belum terbaca. Tunggu beberapa detik; penjelasan tiap bagian akan muncul di sini.",
  },
  settings: {
    title: "Pengaturan",
    back: "Kembali",
    language: "Bahasa",
    interval: "Seberapa sering diperbarui",
    intervalFast: "Tiap 1 detik (paling cepat, sedikit lebih boros)",
    intervalBalanced: "Tiap 2 detik (seimbang)",
    intervalLow: "Tiap 5 detik (paling hemat)",
    temperature: "Sumber suhu CPU",
    temperatureAuto: "Otomatis",
    temperatureAcpi: "Windows (ACPI)",
    temperatureLhm: "LibreHardwareMonitor",
    temperatureOff: "Nonaktif",
    temperatureNote:
      "Tanpa driver tambahan, Windows sering tidak memberi suhu CPU. Pilihan ini hanya mengubah cara membacanya, bukan menambah izin administrator.",
    startMinimized: "Mulai tanpa jendela",
    startMinimizedHint:
      "Aplikasi langsung berjalan di system tray saat dibuka.",
    saveError: "Pengaturan gagal disimpan. Coba lagi.",
  },
  footer: {
    updated: (seconds) =>
      seconds < 1 ? "Baru saja diperbarui" : `Diperbarui ${seconds} detik lalu`,
    tempSource: "Sumber suhu",
    noTemp: "tidak tersedia",
    lhm: "LibreHardwareMonitor",
    acpi: "ACPI Windows",
    nvml: "NVML (GPU)",
  },
  intro: {
    title: "Selamat datang",
    body: "Aplikasi ini membaca kondisi komputer dan menerjemahkannya menjadi tiga status sederhana, supaya tidak perlu memahami angka teknis.",
    points: [
      "Hijau berarti aman, kuning waspada, merah perlu ditangani.",
      "Status hanya berubah bila kondisi bertahan beberapa detik, jadi tidak berkedip.",
      "Jendela ini bisa ditutup; aplikasi tetap berjalan di system tray.",
    ],
    dismiss: "Mengerti",
    openSettings: "Buka pengaturan",
  },
  tray: { open: "Buka", settings: "Pengaturan", quit: "Keluar" },
  units: { free: "bebas", of: "dari", threads: "thread" },
  tooltip: {
    cpu: "CPU",
    ram: "RAM",
    noTemp: "Suhu CPU tidak terdeteksi",
    unknown: "Menunggu data hardware",
  },
};

const en: Dictionary = {
  appName: "ArxseusMonitor",
  tagline: "Computer health without the jargon",
  metricsHeading: "Metric summary",
  level: {
    safe: "Safe",
    warn: "Warning",
    danger: "Danger",
    unknown: "Waiting for data",
  },
  banner: {
    safe: "Everything being monitored is within safe limits.",
    warn: "One part needs watching.",
    danger: "One part needs attention now.",
    unknown: "Hardware data is not available yet.",
  },
  metric: {
    cpu: {
      label: "CPU load",
      why: "The CPU is the computer's brain. A short spike when opening an app is normal; staying high all the time is the problem.",
    },
    ram: {
      label: "Memory (RAM)",
      why: "RAM is the computer's workbench. The more apps you open, the fuller it gets, and the computer starts moving data to a much slower disk.",
    },
    disk: {
      label: "Storage space",
      why: "Windows and apps need free space to work. When it runs low, storage slows down and Windows updates can fail.",
    },
    cpuTemp: {
      label: "CPU temperature",
      why: "An overheated component slows itself down (throttling) to avoid damage. That is why the computer feels sluggish when hot.",
    },
    gpuTemp: {
      label: "Graphics card temperature",
      why: "The graphics card heats up while rendering images, especially in games. Sustained heat shortens component life.",
    },
  },
  reason: {
    "cpu.ok": { title: "CPU load is normal.", advice: "Nothing to do." },
    "cpu.high": {
      title: "The CPU is fairly busy.",
      advice: "Close apps you are not using if the computer feels slow.",
    },
    "cpu.critical": {
      title: "The CPU is working very hard.",
      advice: "Close heavy apps, then check for processes you do not recognise.",
    },
    "cpu.unavailable": {
      title: "CPU load is not readable yet.",
      advice: "Wait a few seconds; if it stays empty, restart the app.",
    },
    "ram.ok": { title: "Memory is comfortable.", advice: "Nothing to do." },
    "ram.high": {
      title: "Memory is nearly full.",
      advice: "Close browser tabs or apps you are not using.",
    },
    "ram.critical": {
      title: "Memory is critically full.",
      advice: "Save your work, close apps, and restart the computer if needed.",
    },
    "ram.unavailable": {
      title: "Memory usage is not readable yet.",
      advice: "Wait a few seconds; if it stays empty, restart the app.",
    },
    "disk.ok": { title: "Storage space is sufficient.", advice: "Nothing to do." },
    "disk.low": {
      title: "Storage space is running low.",
      advice: "Delete large files or move them to another drive.",
    },
    "disk.critical": {
      title: "Storage space is almost full.",
      advice: "Free up at least 10% of the drive so Windows stays responsive.",
    },
    "disk.unavailable": {
      title: "Storage space is not detected.",
      advice: "Check that the drive is connected properly.",
    },
    "temp.cpu.ok": { title: "CPU temperature is normal.", advice: "Nothing to do." },
    "temp.cpu.warm": {
      title: "The CPU is getting warm.",
      advice: "Make sure vents are clear and fans are not dusty.",
    },
    "temp.cpu.critical": {
      title: "The CPU is too hot.",
      advice: "Stop heavy tasks and clean the fan. Overheating slows the computer.",
    },
    "temp.cpu.unavailable": {
      title: "CPU temperature is not available on this device.",
      advice: "This is a Windows limitation without extra drivers, not a fault.",
    },
    "temp.gpu.ok": { title: "GPU temperature is normal.", advice: "Nothing to do." },
    "temp.gpu.warm": {
      title: "The graphics card is getting warm.",
      advice: "Lower heavy graphics settings or take a short break.",
    },
    "temp.gpu.critical": {
      title: "The graphics card is too hot.",
      advice: "Stop games or heavy graphics apps and check airflow inside the case.",
    },
    "temp.gpu.unavailable": {
      title: "GPU temperature is not detected.",
      advice: "Without an NVIDIA card this reading is unavailable. Not a fault.",
    },
  },
  why: {
    title: "Why?",
    empty:
      "Hardware data is not readable yet. Wait a few seconds; an explanation for each part will appear here.",
  },
  settings: {
    title: "Settings",
    back: "Back",
    language: "Language",
    interval: "Refresh rate",
    intervalFast: "Every 1 second (fastest, slightly heavier)",
    intervalBalanced: "Every 2 seconds (balanced)",
    intervalLow: "Every 5 seconds (lightest)",
    temperature: "CPU temperature source",
    temperatureAuto: "Automatic",
    temperatureAcpi: "Windows (ACPI)",
    temperatureLhm: "LibreHardwareMonitor",
    temperatureOff: "Off",
    temperatureNote:
      "Without extra drivers Windows often does not expose CPU temperature. These options only change how it is read; the app never asks for administrator rights.",
    startMinimized: "Start without a window",
    startMinimizedHint: "The app goes straight to the system tray when launched.",
    saveError: "Settings could not be saved. Please try again.",
  },
  footer: {
    updated: (seconds) =>
      seconds < 1 ? "Updated just now" : `Updated ${seconds}s ago`,
    tempSource: "Temperature source",
    noTemp: "unavailable",
    lhm: "LibreHardwareMonitor",
    acpi: "Windows ACPI",
    nvml: "NVML (GPU)",
  },
  intro: {
    title: "Welcome",
    body: "This app reads your computer's condition and translates it into three plain statuses, so you never have to read raw numbers.",
    points: [
      "Green is safe, yellow needs watching, red needs attention.",
      "A status only changes after it holds for several seconds, so it does not flicker.",
      "You can close this window; the app keeps running in the system tray.",
    ],
    dismiss: "Got it",
    openSettings: "Open settings",
  },
  tray: { open: "Open", settings: "Settings", quit: "Quit" },
  units: { free: "free", of: "of", threads: "threads" },
  tooltip: {
    cpu: "CPU",
    ram: "RAM",
    noTemp: "CPU temperature unavailable",
    unknown: "Waiting for hardware data",
  },
};

export const dictionaries: Record<Language, Dictionary> = { id, en };

export function dictionary(language: Language): Dictionary {
  return dictionaries[language] ?? id;
}

export function locale(language: Language): string {
  return language === "id" ? "id-ID" : "en-US";
}

export function formatNumber(
  language: Language,
  value: number,
  digits = 1,
): string {
  return new Intl.NumberFormat(locale(language), {
    maximumFractionDigits: digits,
    minimumFractionDigits: 0,
  }).format(value);
}

export function formatPercent(language: Language, value: number): string {
  return `${formatNumber(language, value, 1)}%`;
}

export function formatTemperature(
  language: Language,
  celsius: number,
): string {
  return `${formatNumber(language, celsius, 1)} °C`;
}

export function formatBytes(language: Language, bytes: number): string {
  const gb = bytes / 1024 ** 3;
  if (gb >= 1) return `${formatNumber(language, gb, 1)} GB`;

  const mb = bytes / 1024 ** 2;
  return `${formatNumber(language, Math.max(mb, 0), 0)} MB`;
}
