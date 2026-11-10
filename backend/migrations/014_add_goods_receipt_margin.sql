-- PostgreSQL migration: margin laba (profit margin) per item penerimaan barang.
-- Kolom nullable: NULL = harga jual SKU tidak diubah saat posting (harga jual
-- diatur manual di master data); angka >= 0 = selling_price dihitung ulang dari
-- unit_cost × (1 + margin/100) saat goods receipt diposting.

BEGIN;

ALTER TABLE goods_receipt_items
    ADD COLUMN IF NOT EXISTS margin_percent NUMERIC(5, 2);

ALTER TABLE goods_receipt_items
    ADD CONSTRAINT goods_receipt_items_margin_percent_non_negative
    CHECK (margin_percent IS NULL OR margin_percent >= 0);

COMMIT;
