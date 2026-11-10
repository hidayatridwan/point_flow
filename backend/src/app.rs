use std::sync::Arc;
use std::time::Duration;

use axum::http::HeaderValue;
use sqlx::PgPool;

use crate::{
    common::models::PaginationMeta,
    config::MidtransSettings,
    infra::{cache::TtlCache, s3::ObjectStorage},
    modules::online_orders::dto::{PublicCategory, PublicProductItem, StoreInfoResponse},
};

/// Nilai cache katalog publik: (item halaman, meta paginasi).
type CatalogCacheValue = (Vec<PublicProductItem>, PaginationMeta);

/// Cache respons publik storefront (in-process, tanpa Redis).
/// TTL produk lebih pendek karena harga bisa berubah; perubahan **stok** dari
/// penjualan sengaja mengandalkan TTL (bukan invalidasi) agar alur kasir yang
/// ramai tidak mengosongkan cache terus-menerus — staleness maksimal 30 detik.
pub struct PublicCache {
    pub catalog: TtlCache<CatalogCacheValue>,
    pub categories: TtlCache<Vec<PublicCategory>>,
    pub store_info: TtlCache<StoreInfoResponse>,
}

impl Default for PublicCache {
    fn default() -> Self {
        Self {
            catalog: TtlCache::new(Duration::from_secs(30)),
            categories: TtlCache::new(Duration::from_secs(300)),
            store_info: TtlCache::new(Duration::from_secs(300)),
        }
    }
}

impl PublicCache {
    /// Kosongkan seluruh cache publik. Dipanggil handler mutasi master data
    /// (produk, SKU, unit, kategori) dan pengaturan toko/QRIS.
    pub fn invalidate(&self) {
        self.catalog.clear();
        self.categories.clear();
        self.store_info.clear();
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cors_allowed_origins: Vec<HeaderValue>,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub refresh_token_ttl_seconds: i64,
    pub refresh_token_cookie_name: String,
    pub refresh_token_cookie_secure: bool,
    pub refresh_token_cookie_same_site: String,
    pub refresh_token_cookie_domain: Option<String>,
    pub s3: Option<Arc<ObjectStorage>>,
    pub midtrans: Option<MidtransSettings>,
    pub public_cache: Arc<PublicCache>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: PgPool,
        cors_allowed_origins: Vec<HeaderValue>,
        jwt_secret: String,
        jwt_ttl_seconds: i64,
        refresh_token_ttl_seconds: i64,
        refresh_token_cookie_name: String,
        refresh_token_cookie_secure: bool,
        refresh_token_cookie_same_site: String,
        refresh_token_cookie_domain: Option<String>,
        s3: Option<Arc<ObjectStorage>>,
        midtrans: Option<MidtransSettings>,
    ) -> Self {
        Self {
            db,
            cors_allowed_origins,
            jwt_secret,
            jwt_ttl_seconds,
            refresh_token_ttl_seconds,
            refresh_token_cookie_name,
            refresh_token_cookie_secure,
            refresh_token_cookie_same_site,
            refresh_token_cookie_domain,
            s3,
            midtrans,
            public_cache: Arc::new(PublicCache::default()),
        }
    }
}
