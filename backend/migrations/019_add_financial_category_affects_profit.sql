-- Pendapatan/beban manual kini ikut dihitung di Laba Rugi (baris "Pendapatan
-- Lain"). Beberapa kategori bukan laba/rugi — misalnya setoran modal, prive,
-- atau pinjaman — sehingga hanya menggeser kas. Kategori seperti itu bisa
-- ditandai `affects_profit = FALSE` agar dikecualikan dari laba.
BEGIN;

ALTER TABLE financial_categories
    ADD COLUMN affects_profit BOOLEAN NOT NULL DEFAULT TRUE;

COMMIT;