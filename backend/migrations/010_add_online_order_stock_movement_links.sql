-- PostgreSQL migration: link stock movements to online orders.
-- Packing cuts stock before the sale row exists, so movements written at
-- pack time reference the online order; completing the order backfills the
-- sale links. All columns are nullable and the change is additive.

BEGIN;

ALTER TABLE stock_movements
    ADD COLUMN online_order_id BIGINT REFERENCES online_orders(id) ON DELETE SET NULL,
    ADD COLUMN online_order_item_id BIGINT REFERENCES online_order_items(id) ON DELETE SET NULL;

CREATE INDEX stock_movements_online_order_idx ON stock_movements (online_order_id);

COMMIT;
