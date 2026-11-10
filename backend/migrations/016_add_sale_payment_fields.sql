-- Kasir non-tunai manual: catat metode + referensi transfer pada sale.
-- Tunai tetap default 'cash' (baris lama otomatis 'cash'); settlement
-- Midtrans fase 2+ mengisi 'midtrans' + order_id sebagai referensi.
-- Kolom ini yang memisahkan omzet tunai vs non-tunai di laporan.
ALTER TABLE sales
    ADD COLUMN payment_method VARCHAR(20) NOT NULL DEFAULT 'cash',
    ADD COLUMN payment_reference TEXT;

ALTER TABLE sales
    ADD CONSTRAINT sales_payment_method_valid CHECK (payment_method IN ('cash', 'transfer', 'midtrans'));

CREATE INDEX sales_payment_method_idx ON sales (payment_method);
