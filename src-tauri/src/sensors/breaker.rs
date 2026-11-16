//! Circuit breaker per provider (lihat `DESIGN.md` §9).
//!
//! Provider yang gagal 5 kali berturut-turut dinonaktifkan selama 5 menit, lalu
//! dicoba lagi. Tujuannya: satu sensor bermasalah tidak boleh membuat aplikasi
//! memanggil WMI yang menggantung setiap 2 detik selamanya.
//!
//! Waktu disuntikkan lewat parameter `now`, bukan dibaca langsung dari
//! `Instant::now()`, supaya perilakunya bisa diuji tanpa menunggu 5 menit.

use std::time::{Duration, Instant};

pub const FAILURE_THRESHOLD: u32 = 5;
pub const COOLDOWN: Duration = Duration::from_secs(300);

#[derive(Debug)]
pub struct Breaker {
    failures: u32,
    open_until: Option<Instant>,
    threshold: u32,
    cooldown: Duration,
}

impl Default for Breaker {
    fn default() -> Self {
        Self::new()
    }
}

impl Breaker {
    pub fn new() -> Self {
        Self::with_limits(FAILURE_THRESHOLD, COOLDOWN)
    }

    pub fn with_limits(threshold: u32, cooldown: Duration) -> Self {
        Self {
            failures: 0,
            open_until: None,
            threshold: threshold.max(1),
            cooldown,
        }
    }

    /// Apakah provider boleh dipanggil sekarang.
    pub fn allows(&self, now: Instant) -> bool {
        self.open_until.is_none_or(|until| now >= until)
    }

    /// Catat satu pembacaan. `ok == false` berarti provider error *atau*
    /// melewati anggaran waktu — keduanya sama-sama tidak bisa dipercaya.
    pub fn record(&mut self, ok: bool, now: Instant) {
        if ok {
            self.failures = 0;
            self.open_until = None;
            return;
        }

        self.failures = self.failures.saturating_add(1);
        if self.failures >= self.threshold {
            self.open_until = Some(now + self.cooldown);
            self.failures = 0;
        }
    }

    pub fn failures(&self) -> u32 {
        self.failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> Instant {
        Instant::now()
    }

    #[test]
    fn starts_closed() {
        let breaker = Breaker::new();
        assert!(breaker.allows(t0()));
        assert_eq!(breaker.failures(), 0);
    }

    #[test]
    fn opens_after_threshold_failures() {
        let start = t0();
        let mut breaker = Breaker::with_limits(3, Duration::from_secs(60));

        breaker.record(false, start);
        breaker.record(false, start);
        assert!(breaker.allows(start));

        breaker.record(false, start);
        assert!(!breaker.allows(start));
    }

    #[test]
    fn reopens_after_cooldown() {
        let start = t0();
        let mut breaker = Breaker::with_limits(2, Duration::from_secs(60));

        breaker.record(false, start);
        breaker.record(false, start);
        assert!(!breaker.allows(start));
        assert!(breaker.allows(start + Duration::from_secs(61)));
    }

    #[test]
    fn success_resets_the_counter() {
        let start = t0();
        let mut breaker = Breaker::with_limits(3, Duration::from_secs(60));

        breaker.record(false, start);
        breaker.record(false, start);
        breaker.record(true, start);
        assert_eq!(breaker.failures(), 0);

        breaker.record(false, start);
        assert!(breaker.allows(start));
    }
}
