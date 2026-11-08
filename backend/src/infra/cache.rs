use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Cache in-process TTL sederhana (tanpa Redis).
/// Dipilih daripada Redis/Upstash karena: 0 hop jaringan (lebih cepat),
/// tidak menambah biaya per request (Upstash menagih per command), dan
/// volume baca publik app ini kecil sehingga konsistensi antar-machine
/// cukup dijamin TTL + invalidasi eksplisit saat data master berubah.
pub struct TtlCache<V> {
    ttl: Duration,
    entries: Mutex<HashMap<String, (Instant, V)>>,
}

impl<V: Clone> TtlCache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Ambil nilai jika masih segar; entri kedaluwarsa dibuang saat disentuh.
    pub fn get(&self, key: &str) -> Option<V> {
        let mut entries = self.entries.lock().unwrap_or_else(|err| err.into_inner());
        let (stored_at, value) = entries.get(key)?;
        if stored_at.elapsed() >= self.ttl {
            entries.remove(key);
            return None;
        }
        Some(value.clone())
    }

    pub fn insert(&self, key: impl Into<String>, value: V) {
        self.entries
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(key.into(), (Instant::now(), value));
    }

    /// Buang semua entri (invalidasi saat data berubah).
    pub fn clear(&self) {
        self.entries
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_inserted_value() {
        let cache: TtlCache<u32> = TtlCache::new(Duration::from_secs(60));
        assert_eq!(cache.get("a"), None);
        cache.insert("a", 1);
        assert_eq!(cache.get("a"), Some(1));
    }

    #[test]
    fn get_drops_expired_value() {
        let cache: TtlCache<u32> = TtlCache::new(Duration::from_secs(0));
        cache.insert("a", 1);
        assert_eq!(cache.get("a"), None);
    }

    #[test]
    fn clear_removes_all_entries() {
        let cache: TtlCache<u32> = TtlCache::new(Duration::from_secs(60));
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.clear();
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), None);
    }
}
