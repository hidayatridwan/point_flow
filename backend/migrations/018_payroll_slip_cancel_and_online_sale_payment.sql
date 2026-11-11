-- Perbaikan temuan panduan pengguna (fase 8):
-- 1. Slip payroll kini bisa dibatalkan (kolom status + jejak pembatalan).
--    Slip batal dikeluarkan dari v_payroll_report sehingga Laporan Payroll
--    dan baris beban gaji di Laba Rugi/Posisi Keuangan tidak lagi menghitung
--    slip yang salah buat.
-- 2. Penjualan online yang lahir saat order complete selama ini tidak punya
--    payment_method (NULL) sehingga tidak terhitung di split tunai/non-tunai
--    (v_daily/v_monthly) dan tidak bisa dipilah kas vs non-kas. Backfill
--    berdasarkan pesanan: bayar di toko (pay_at_store) = tunai; transfer
--    manual = transfer; yang dilunasi via Midtrans = midtrans.
BEGIN;

ALTER TABLE payroll_slips
    ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'completed',
    ADD COLUMN cancelled_at TIMESTAMPTZ,
    ADD COLUMN cancelled_by BIGINT REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN cancel_reason TEXT;

UPDATE sales s
SET payment_method = CASE
        WHEN oo.payment_method = 'pay_at_store' THEN 'cash'
        WHEN EXISTS (
            SELECT 1
            FROM midtrans_payments mp
            WHERE mp.online_order_id = oo.id
                AND (mp.transaction_status = 'settlement'
                    OR (mp.transaction_status = 'capture'
                        AND mp.fraud_status = 'accept'))
        ) THEN 'midtrans'
        ELSE 'transfer'
    END
FROM online_orders oo
WHERE s.online_order_id = oo.id
    AND s.channel = 'online'
    AND s.payment_method IS NULL;

DROP VIEW IF EXISTS v_payroll_report;
CREATE VIEW v_payroll_report AS
SELECT
    period_start,
    period_end,
    COUNT(*) AS slip_count,
    COALESCE(SUM(base_salary), 0) AS total_base_salary,
    COALESCE(SUM(bonus), 0) AS total_bonus,
    COALESCE(SUM(deduction), 0) AS total_deduction,
    COALESCE(SUM(net_salary), 0) AS total_net_salary
FROM payroll_slips
WHERE status = 'completed'
GROUP BY period_start, period_end;

COMMIT;
