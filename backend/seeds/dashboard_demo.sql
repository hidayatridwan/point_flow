-- Dashboard demo seed
-- ---------------------------------------------------------------------------
-- Populates ~90 days of realistic, randomized transactions so the dashboard
-- charts (sales trend, hourly sales, top products, sales by category, product
-- scatter, cashier performance, procurement funnel, low-stock table) have data.
--
-- Safe to run against an existing database: it only ADDS demo rows and is
-- guarded so it will not duplicate itself on re-run. Everything it inserts is
-- tagged with notes = 'demo-seed' (and demo cashier emails) for easy removal.
--
-- Run:
--   docker compose exec -T postgres psql -U pos -d pos < backend/seeds/dashboard_demo.sql
--
-- Re-seed from scratch: uncomment and run the RESET block below first.
-- ---------------------------------------------------------------------------

-- === RESET (optional) ======================================================
-- BEGIN;
-- DELETE FROM stock_adjustments WHERE notes = 'demo-seed';
-- DELETE FROM goods_receipts    WHERE notes = 'demo-seed';
-- DELETE FROM purchase_orders   WHERE notes = 'demo-seed';
-- DELETE FROM sales             WHERE notes = 'demo-seed'; -- cascades to sale_items
-- DELETE FROM cash_register_sessions WHERE opening_notes = 'demo-seed';
-- DELETE FROM cash_registers    WHERE lower(code) = 'reg-01';
-- DELETE FROM users             WHERE email IN ('dewi@example.com','budi@example.com','sari@example.com');
-- COMMIT;
-- ===========================================================================

BEGIN;

DO $$
DECLARE
	v_admin_id      bigint;
	v_pw_hash       text;
	v_register_id   bigint;
	v_session_id    bigint;
	v_cashiers      bigint[];
	v_opened_at     timestamptz := (CURRENT_DATE - INTERVAL '89 days');
	v_day           date;
	v_num_sales     int;
	v_sale_id       bigint;
	v_sold_at       timestamptz;
	v_cashier       bigint;
	v_num_items     int;
	v_subtotal      numeric(14, 2);
	v_total         numeric(14, 2);
	v_paid          numeric(14, 2);
	v_qty           numeric(14, 3);
	v_sku           record;
	v_any_po        bigint;
	i               int;
	j               int;
BEGIN
	IF EXISTS (SELECT 1 FROM sales WHERE notes = 'demo-seed') THEN
		RAISE NOTICE 'Demo data already present — skipping. Run the RESET block to re-seed.';
		RETURN;
	END IF;

	SELECT id, password_hash INTO v_admin_id, v_pw_hash
	FROM users WHERE role_id = 1 AND deleted_at IS NULL ORDER BY id LIMIT 1;

	IF v_admin_id IS NULL THEN
		RAISE EXCEPTION 'No admin user found to anchor demo data.';
	END IF;

	-- 0. Assign any uncategorized products to a category by name, so the
	--    "sales by category" pie chart shows a meaningful breakdown.
	UPDATE products SET category_id = 1
	WHERE category_id IS NULL
		AND (name ILIKE 'Mineral Water%' OR name ILIKE 'UHT Milk%'
			OR name ILIKE 'Coffee%' OR name ILIKE 'Tea%');     -- Beverages
	UPDATE products SET category_id = 2
	WHERE category_id IS NULL AND name ILIKE 'Instant Noodles%'; -- Snacks
	UPDATE products SET category_id = 3
	WHERE category_id IS NULL AND name ILIKE 'Toothpaste%';      -- Personal Care
	UPDATE products SET category_id = 4 WHERE category_id IS NULL; -- Household Supplies

	-- 1. Cashier users (reuse the admin password hash so they can also log in) ---
	INSERT INTO users (name, email, password_hash, role, role_id, is_active, last_login_at)
	SELECT x.name, x.email, v_pw_hash, 'cashier', 2, TRUE, now()
	FROM (VALUES
		('Dewi Lestari', 'dewi@example.com'),
		('Budi Santoso', 'budi@example.com'),
		('Sari Wahyuni', 'sari@example.com')
	) AS x(name, email)
	WHERE NOT EXISTS (SELECT 1 FROM users u WHERE lower(u.email) = lower(x.email));

	-- Cashier pool used for sales: the 3 demo cashiers + the admin, for variety.
	SELECT array_agg(id) INTO v_cashiers
	FROM users
	WHERE email IN ('dewi@example.com', 'budi@example.com', 'sari@example.com');
	v_cashiers := v_cashiers || v_admin_id;

	-- 2. A cash register with one open session to anchor the sales -------------
	SELECT id INTO v_register_id
	FROM cash_registers WHERE lower(code) = 'reg-01' AND deleted_at IS NULL;
	IF v_register_id IS NULL THEN
		INSERT INTO cash_registers (code, name, description, status)
		VALUES ('REG-01', 'Main Register', 'Primary front counter register', 'active')
		RETURNING id INTO v_register_id;
	END IF;

	SELECT id INTO v_session_id
	FROM cash_register_sessions WHERE cash_register_id = v_register_id AND status = 'open';
	IF v_session_id IS NULL THEN
		INSERT INTO cash_register_sessions
			(cash_register_id, opened_at, opened_by, opening_cash, opening_notes, status, created_at)
		VALUES
			(v_register_id, v_opened_at, v_admin_id, 500000, 'demo-seed', 'open', v_opened_at)
		RETURNING id INTO v_session_id;
	END IF;

	-- 3. ~90 days of completed sales -------------------------------------------
	FOR v_day IN
		SELECT generate_series(CURRENT_DATE - INTERVAL '89 days', CURRENT_DATE, INTERVAL '1 day')::date
	LOOP
		-- Busier on weekends (Sun=0, Sat=6).
		v_num_sales := 3 + floor(random() *
			(CASE WHEN extract(dow FROM v_day) IN (0, 6) THEN 12 ELSE 8 END))::int;

		FOR i IN 1..v_num_sales LOOP
			-- Spread across business hours 08:00–21:59. Demo data may include
			-- same-day "future" timestamps so the hourly chart shows a full
			-- daily curve even when seeded early in the day.
			v_sold_at := v_day
				+ (8 + floor(random() * 14)) * INTERVAL '1 hour'
				+ floor(random() * 60) * INTERVAL '1 minute';

			v_cashier := v_cashiers[1 + floor(random() * array_length(v_cashiers, 1))::int];

			INSERT INTO sales
				(sold_at, cashier_id, cash_register_session_id, status, notes,
				 subtotal_amount, total_amount, paid_amount, change_amount, created_at, updated_at)
			VALUES
				(v_sold_at, v_cashier, v_session_id, 'completed', 'demo-seed',
				 0, 0, 0, 0, v_sold_at, v_sold_at)
			RETURNING id INTO v_sale_id;

			v_num_items := 1 + floor(random() * 4)::int; -- 1..4 line items
			v_subtotal := 0;

			FOR j IN 1..v_num_items LOOP
				SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name,
					ps.sku, u.code AS unit, ps.selling_price
				INTO v_sku
				FROM product_skus ps
				JOIN products p ON p.id = ps.product_id
				JOIN units u ON u.id = ps.unit_id
				WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
					AND p.deleted_at IS NULL AND p.is_active = TRUE
				ORDER BY random()
				LIMIT 1;

				v_qty := (1 + floor(random() * 5))::numeric; -- 1..5 units

				INSERT INTO sale_items
					(sale_id, product_id, product_sku_id, product_name, product_sku,
					 unit, quantity, unit_price, created_at)
				VALUES
					(v_sale_id, v_sku.product_id, v_sku.sku_id, v_sku.product_name, v_sku.sku,
					 v_sku.unit, v_qty, v_sku.selling_price, v_sold_at);

				v_subtotal := v_subtotal + (v_qty * v_sku.selling_price);
			END LOOP;

			v_total := v_subtotal;
			v_paid := ceil(v_total / 5000.0) * 5000; -- round cash up to nearest 5k

			UPDATE sales SET
				subtotal_amount = v_subtotal,
				total_amount = v_total,
				paid_amount = v_paid,
				change_amount = v_paid - v_total
			WHERE id = v_sale_id;
		END LOOP;
	END LOOP;

	-- 4. Make a handful of SKUs low on stock (low-stock table + KPI) ------------
	UPDATE product_skus SET stock = 10 WHERE id = 1;  -- min 24
	UPDATE product_skus SET stock = 12 WHERE id = 3;  -- min 30
	UPDATE product_skus SET stock = 2  WHERE id = 9;  -- min 5
	UPDATE product_skus SET stock = 0  WHERE id = 16; -- min 1
	UPDATE product_skus SET stock = 1  WHERE id = 18; -- min 2

	-- 5. Enrich the procurement funnel (header-only demo documents) -------------
	INSERT INTO purchase_orders (supplier_id, order_date, status, subtotal_amount, notes, created_by, created_at)
	SELECT 1 + floor(random() * 4)::int, CURRENT_DATE - floor(random() * 30)::int,
		'draft', 0, 'demo-seed', v_admin_id, now()
	FROM generate_series(1, 4);

	INSERT INTO purchase_orders (supplier_id, order_date, status, subtotal_amount, notes, created_by, ordered_by, ordered_at, created_at)
	SELECT 1 + floor(random() * 4)::int, CURRENT_DATE - floor(random() * 20)::int,
		'ordered', 0, 'demo-seed', v_admin_id, v_admin_id, now(), now()
	FROM generate_series(1, 3);

	SELECT min(id) INTO v_any_po FROM purchase_orders;

	INSERT INTO goods_receipts (purchase_order_id, received_at, status, received_by, notes, created_at)
	SELECT v_any_po, now() - floor(random() * 15)::int * INTERVAL '1 day',
		'draft', v_admin_id, 'demo-seed', now()
	FROM generate_series(1, 2);

	INSERT INTO stock_adjustments (adjusted_at, status, reason, notes, created_by, posted_at, posted_by, created_at)
	SELECT now() - floor(random() * 60)::int * INTERVAL '1 day',
		'posted', 'Stock opname', 'demo-seed', v_admin_id, now(), v_admin_id, now()
	FROM generate_series(1, 2);

	INSERT INTO stock_adjustments (adjusted_at, status, reason, notes, created_by, cancelled_at, cancelled_by, cancel_reason, created_at)
	SELECT now() - floor(random() * 60)::int * INTERVAL '1 day',
		'cancelled', 'Data entry error', 'demo-seed', v_admin_id, now(), v_admin_id, 'demo', now()
	FROM generate_series(1, 1);

	RAISE NOTICE 'Demo seed complete.';
END $$;

COMMIT;
