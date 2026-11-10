-- Laporan memisahkan omzet tunai vs non-tunai (fase 7 Midtrans).
-- Baris `sales` hanya dibuat saat lunas (tunai langsung, transfer manual
-- langsung, Midtrans saat settlement webhook), sehingga kedua view tetap
-- menghitung penjualan completed seperti sebelumnya; pembayaran pending
-- (`midtrans_payments`) tidak masuk omzet karena belum menjadi row `sales`.
-- Tunai = payment_method 'cash'; non-tunai = 'transfer' + 'midtrans'.
CREATE OR REPLACE VIEW v_daily_sales_report AS
SELECT
    sold_at::DATE AS sales_date,
    COUNT(*) AS transaction_count,
    COALESCE(SUM(subtotal_amount), 0) AS subtotal_amount,
    COALESCE(SUM(total_amount), 0) AS total_amount,
    COALESCE(SUM(CASE WHEN payment_method = 'cash' THEN total_amount ELSE 0 END), 0) AS cash_revenue,
    COALESCE(SUM(CASE WHEN payment_method <> 'cash' THEN total_amount ELSE 0 END), 0) AS noncash_revenue
FROM sales
WHERE status = 'completed'
GROUP BY sold_at::DATE;

CREATE OR REPLACE VIEW v_monthly_sales_report AS
SELECT
    date_trunc('month', sold_at)::DATE AS sales_month,
    COUNT(*) AS transaction_count,
    COALESCE(SUM(subtotal_amount), 0) AS subtotal_amount,
    COALESCE(SUM(total_amount), 0) AS total_amount,
    COALESCE(SUM(CASE WHEN payment_method = 'cash' THEN total_amount ELSE 0 END), 0) AS cash_revenue,
    COALESCE(SUM(CASE WHEN payment_method <> 'cash' THEN total_amount ELSE 0 END), 0) AS noncash_revenue
FROM sales
WHERE status = 'completed'
GROUP BY date_trunc('month', sold_at)::DATE;
