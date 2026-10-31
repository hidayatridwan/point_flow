-- Dashboard / full-coverage demo seed
-- ---------------------------------------------------------------------------
-- Populates ~90 days of realistic, randomized data that exercises EVERY
-- transaction type and EVERY lifecycle state in the system, so that the
-- dashboard, every report, and every list page have representative data:
--
--   * Sales ............... completed + cancelled
--   * Sale returns ........ completed + cancelled (with items)
--   * Cash sessions ....... one open (anchors POS) + several closed
--   * Cash drawer moves ... cash_in / cash_out / cash_drop / petty_cash + cancelled
--   * Purchase orders ..... draft / ordered / partially_received / received / cancelled (with items)
--   * Goods receipts ...... draft / posted / cancelled (with items)
--   * Stock adjustments ... draft / posted / cancelled (with items)
--   * Stock conversions ... draft / posted / cancelled (with items)
--   * Stock movements ..... stock_in / stock_out / sale / purchase_receipt /
--                           adjustment / conversion_in / conversion_out /
--                           correction / sale_return
--   * Employees ........... active + inactive
--   * Payroll slips ....... last 3 months per active employee
--
-- Self-contained: if the database has no usable master data it creates a small
-- demo catalog (categories, suppliers, products + SKUs, a multi-SKU product for
-- conversions, employees). When real master data already exists it is reused,
-- and only the minimum demo scaffolding is added.
--
-- Safe to run against an existing database: it only ADDS demo rows and is
-- guarded so it will not duplicate itself on re-run. Demo rows are tagged for
-- easy removal:
--   * notes / opening_notes = 'demo-seed' on documents that have such a column
--   * employees.phone       = 'demo-seed'
--   * suppliers.name        LIKE 'Demo Supplier%'
--   * products.name         LIKE '[DEMO]%'  (only the fallback catalog)
--   * product_skus.sku      LIKE 'DEMO-%'   (only the fallback catalog)
--   * cash_registers.code   = 'REG-01'
--   * demo cashier emails   dewi@/budi@/sari@example.com
--
-- Run:
--   docker compose exec -T postgres psql -U pos -d pos < backend/seeds/dashboard_demo.sql
--
-- Re-seed from scratch: run the RESET block below first (uncomment), then re-run.
-- ---------------------------------------------------------------------------

-- === RESET (optional) ======================================================
-- FK-safe order. Uncomment, run, then re-run the seed.
-- BEGIN;
--   DELETE FROM stock_movements      WHERE notes = 'demo-seed';
--   DELETE FROM sale_returns         WHERE notes = 'demo-seed'; -- cascades sale_return_items
--   DELETE FROM sales                WHERE notes = 'demo-seed'; -- cascades sale_items
--   DELETE FROM cash_drawer_movements WHERE notes = 'demo-seed';
--   DELETE FROM goods_receipts       WHERE notes = 'demo-seed'; -- cascades goods_receipt_items
--   DELETE FROM purchase_orders      WHERE notes = 'demo-seed'; -- cascades purchase_order_items
--   DELETE FROM stock_adjustments    WHERE notes = 'demo-seed'; -- cascades items
--   DELETE FROM stock_conversions    WHERE notes = 'demo-seed'; -- cascades items
--   DELETE FROM payroll_slips        WHERE employee_id IN (SELECT id FROM employees WHERE phone = 'demo-seed');
--   DELETE FROM cash_register_sessions WHERE opening_notes = 'demo-seed';
--   DELETE FROM cash_registers       WHERE lower(code) = 'reg-01';
--   DELETE FROM employees            WHERE phone = 'demo-seed';
--   DELETE FROM product_skus         WHERE sku ILIKE 'DEMO-%';
--   DELETE FROM products             WHERE name ILIKE '[DEMO]%';
--   DELETE FROM suppliers            WHERE name ILIKE 'Demo Supplier%';
--   DELETE FROM users                WHERE email IN ('dewi@example.com','budi@example.com','sari@example.com');
-- COMMIT;
-- ===========================================================================

BEGIN;

DO $$
DECLARE
	v_admin_id       bigint;
	v_pw_hash        text;
	v_register_id    bigint;
	v_session_id     bigint;
	v_cashiers       bigint[];
	v_supplier_ids   bigint[];
	v_emp_ids        bigint[];
	v_cat_bev        bigint;
	v_cat_snack      bigint;
	v_cat_pers       bigint;
	v_cat_house      bigint;
	v_opened_at      timestamptz := (CURRENT_DATE - INTERVAL '89 days');
	v_day            date;
	v_num_sales      int;
	v_sale_id        bigint;
	v_sold_at        timestamptz;
	v_cashier        bigint;
	v_num_items      int;
	v_subtotal       numeric(14, 2);
	v_total          numeric(14, 2);
	v_paid           numeric(14, 2);
	v_qty            numeric(14, 3);
	v_sku            record;
	v_rec            record;
	v_po_id          bigint;
	v_gr_id          bigint;
	v_sa_id          bigint;
	v_sc_id          bigint;
	v_ret_id         bigint;
	v_unit_id        bigint;
	v_pid            bigint;
	v_conv_product   bigint;
	v_from_sku       record;
	v_to_sku         record;
	v_status         text;
	v_received_pos   bigint[] := ARRAY[]::bigint[];
	v_partial_pos    bigint[] := ARRAY[]::bigint[];
	v_ordered_pos    bigint[] := ARRAY[]::bigint[];
	v_period_start   date;
	v_period_end     date;
	v_count          int;
	i                int;
	j                int;
	k                int;
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

	-- 0. Categories ----------------------------------------------------------
	INSERT INTO product_categories (name, description)
	SELECT x.name, 'demo-seed'
	FROM (VALUES ('Beverages'), ('Snacks'), ('Personal Care'), ('Household Supplies')) AS x(name)
	WHERE NOT EXISTS (
		SELECT 1 FROM product_categories c
		WHERE lower(c.name) = lower(x.name) AND c.deleted_at IS NULL
	);

	SELECT id INTO v_cat_bev   FROM product_categories WHERE lower(name) = 'beverages'          AND deleted_at IS NULL ORDER BY id LIMIT 1;
	SELECT id INTO v_cat_snack FROM product_categories WHERE lower(name) = 'snacks'             AND deleted_at IS NULL ORDER BY id LIMIT 1;
	SELECT id INTO v_cat_pers  FROM product_categories WHERE lower(name) = 'personal care'      AND deleted_at IS NULL ORDER BY id LIMIT 1;
	SELECT id INTO v_cat_house FROM product_categories WHERE lower(name) = 'household supplies' AND deleted_at IS NULL ORDER BY id LIMIT 1;

	-- 1. Suppliers -----------------------------------------------------------
	INSERT INTO suppliers (name, contact_name, phone, email, is_active)
	SELECT x.name, x.cn, x.ph, x.em, TRUE
	FROM (VALUES
		('Demo Supplier Sentosa', 'Andi', '0811000001', 'sentosa@demo.example'),
		('Demo Supplier Makmur',  'Beni', '0811000002', 'makmur@demo.example'),
		('Demo Supplier Jaya',    'Cici', '0811000003', 'jaya@demo.example'),
		('Demo Supplier Abadi',   'Dodi', '0811000004', 'abadi@demo.example')
	) AS x(name, cn, ph, em)
	WHERE NOT EXISTS (
		SELECT 1 FROM suppliers s WHERE lower(s.name) = lower(x.name) AND s.deleted_at IS NULL
	);

	SELECT array_agg(id) INTO v_supplier_ids
	FROM suppliers WHERE deleted_at IS NULL AND is_active = TRUE;

	-- 2. Fallback product catalog (only when there is no usable SKU) ----------
	IF NOT EXISTS (
		SELECT 1 FROM product_skus ps
		JOIN products p ON p.id = ps.product_id
		WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
			AND p.deleted_at IS NULL AND p.is_active = TRUE
	) THEN
		FOR v_rec IN
			SELECT * FROM (VALUES
				('Mineral Water 600ml', 'bev',   'pcs',  1500, 3000, 400),
				('UHT Milk 1L',         'bev',   'pcs',  9000, 14000, 220),
				('Coffee Sachet',       'bev',   'pcs',  1200, 2500, 600),
				('Tea Box',             'bev',   'box',  8000, 13000, 90),
				('Instant Noodles',     'snack', 'pcs',  2300, 3500, 800),
				('Potato Chips',        'snack', 'pack', 7000, 11000, 180),
				('Toothpaste',          'pers',  'pcs',  9000, 15000, 140),
				('Shampoo Sachet',      'pers',  'pcs',  1000, 2000, 500),
				('Dish Soap',           'house', 'pcs',  8000, 13000, 160),
				('Floor Cleaner 800ml', 'house', 'pcs', 12000, 19000, 120)
			) AS t(pname, cat, unitcode, pprice, sprice, stk)
		LOOP
			INSERT INTO products (category_id, name)
			VALUES (
				CASE v_rec.cat
					WHEN 'bev'   THEN v_cat_bev
					WHEN 'snack' THEN v_cat_snack
					WHEN 'pers'  THEN v_cat_pers
					ELSE v_cat_house
				END,
				v_rec.pname
			)
			RETURNING id INTO v_pid;

			SELECT id INTO v_unit_id FROM units WHERE lower(code) = v_rec.unitcode AND deleted_at IS NULL LIMIT 1;

			INSERT INTO product_skus
				(product_id, sku, unit_id, contents_qty, purchase_price, selling_price, stock, minimum_stock, is_active)
			VALUES
				(v_pid, 'DEMO-' || v_pid || '-1', v_unit_id, 1, v_rec.pprice, v_rec.sprice, v_rec.stk, 20, TRUE);
		END LOOP;
	END IF;

	-- 2b. Ensure a product with >= 2 active SKUs exists (for conversions) ------
	SELECT ps.product_id INTO v_conv_product
	FROM product_skus ps
	JOIN products p ON p.id = ps.product_id
	WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
		AND p.deleted_at IS NULL AND p.is_active = TRUE
	GROUP BY ps.product_id
	HAVING COUNT(*) >= 2
	LIMIT 1;

	IF v_conv_product IS NULL THEN
		INSERT INTO products (category_id, name)
		VALUES (v_cat_bev, 'Bundle Drink (Box & Pcs)')
		RETURNING id INTO v_conv_product;

		SELECT id INTO v_unit_id FROM units WHERE lower(code) = 'box' AND deleted_at IS NULL LIMIT 1;
		INSERT INTO product_skus (product_id, sku, unit_id, contents_qty, purchase_price, selling_price, stock, minimum_stock, is_active)
		VALUES (v_conv_product, 'DEMO-' || v_conv_product || '-BOX', v_unit_id, 24, 60000, 84000, 50, 5, TRUE);

		SELECT id INTO v_unit_id FROM units WHERE lower(code) = 'pcs' AND deleted_at IS NULL LIMIT 1;
		INSERT INTO product_skus (product_id, sku, unit_id, contents_qty, purchase_price, selling_price, stock, minimum_stock, is_active)
		VALUES (v_conv_product, 'DEMO-' || v_conv_product || '-PCS', v_unit_id, 1, 2500, 3500, 240, 24, TRUE);
	END IF;

	-- 3. Assign any uncategorized products so "sales by category" is meaningful -
	UPDATE products SET category_id = v_cat_bev
		WHERE category_id IS NULL AND deleted_at IS NULL
			AND (name ILIKE '%water%' OR name ILIKE '%milk%' OR name ILIKE '%coffee%' OR name ILIKE '%tea%' OR name ILIKE '%drink%');
	UPDATE products SET category_id = v_cat_snack
		WHERE category_id IS NULL AND deleted_at IS NULL
			AND (name ILIKE '%noodle%' OR name ILIKE '%chip%' OR name ILIKE '%snack%');
	UPDATE products SET category_id = v_cat_pers
		WHERE category_id IS NULL AND deleted_at IS NULL
			AND (name ILIKE '%toothpaste%' OR name ILIKE '%shampoo%' OR name ILIKE '%soap%');
	UPDATE products SET category_id = v_cat_house
		WHERE category_id IS NULL AND deleted_at IS NULL;

	-- 4. Employees (active + one inactive) -----------------------------------
	INSERT INTO employees (name, position, phone, address, base_salary, is_active)
	SELECT x.name, x.pos, 'demo-seed', x.addr, x.sal, x.active
	FROM (VALUES
		('Dewi Lestari',  'Senior Cashier', 'Jl. Melati No. 1',  4000000, TRUE),
		('Budi Santoso',  'Cashier',        'Jl. Mawar No. 2',   3500000, TRUE),
		('Sari Wahyuni',  'Cashier',        'Jl. Anggrek No. 3', 3500000, TRUE),
		('Eko Prasetyo',  'Store Keeper',   'Jl. Kenanga No. 4', 3800000, TRUE),
		('Rina Marlina',  'Supervisor',     'Jl. Dahlia No. 5',  5500000, TRUE),
		('Joko Susilo',   'Helper',         'Jl. Flamboyan No.6',3000000, FALSE)
	) AS x(name, pos, addr, sal, active)
	WHERE NOT EXISTS (
		SELECT 1 FROM employees e WHERE e.name = x.name AND e.deleted_at IS NULL
	);

	-- 5. Cashier users (reuse the admin password hash so they can also log in) -
	INSERT INTO users (name, email, password_hash, role, role_id, is_active, last_login_at)
	SELECT x.name, x.email, v_pw_hash, 'cashier', 2, TRUE, now()
	FROM (VALUES
		('Dewi Lestari', 'dewi@example.com'),
		('Budi Santoso', 'budi@example.com'),
		('Sari Wahyuni', 'sari@example.com')
	) AS x(name, email)
	WHERE NOT EXISTS (SELECT 1 FROM users u WHERE lower(u.email) = lower(x.email));

	SELECT array_agg(id) INTO v_cashiers
	FROM users
	WHERE email IN ('dewi@example.com', 'budi@example.com', 'sari@example.com');
	v_cashiers := v_cashiers || v_admin_id;

	-- 6. Cash register + one open session (anchors POS) + closed sessions ------
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

	-- A few historical CLOSED sessions (one per day, days 5..3 ago).
	FOR k IN 3..5 LOOP
		INSERT INTO cash_register_sessions
			(cash_register_id, opened_at, opened_by, opening_cash, opening_notes,
			 closed_at, closed_by, closing_cash_counted, expected_cash, cash_difference,
			 closing_notes, status, created_at)
		VALUES (
			v_register_id,
			CURRENT_DATE - k * INTERVAL '1 day' + INTERVAL '8 hours',
			v_cashiers[1 + floor(random() * array_length(v_cashiers, 1))::int],
			500000, 'demo-seed',
			CURRENT_DATE - k * INTERVAL '1 day' + INTERVAL '21 hours',
			v_cashiers[1 + floor(random() * array_length(v_cashiers, 1))::int],
			500000 + floor(random() * 2000000)::int,
			500000 + floor(random() * 2000000)::int,
			(floor(random() * 100000) - 50000)::int,
			'demo-seed closing', 'closed',
			CURRENT_DATE - k * INTERVAL '1 day'
		);
	END LOOP;

	-- 7. ~90 days of COMPLETED sales -----------------------------------------
	FOR v_day IN
		SELECT generate_series(CURRENT_DATE - INTERVAL '89 days', CURRENT_DATE, INTERVAL '1 day')::date
	LOOP
		-- Busier on weekends (Sun=0, Sat=6).
		v_num_sales := 3 + floor(random() *
			(CASE WHEN extract(dow FROM v_day) IN (0, 6) THEN 12 ELSE 8 END))::int;

		FOR i IN 1..v_num_sales LOOP
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

			FOR v_sku IN
				SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name,
					ps.sku, u.code AS unit, ps.selling_price
				FROM product_skus ps
				JOIN products p ON p.id = ps.product_id
				JOIN units u ON u.id = ps.unit_id
				WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
					AND p.deleted_at IS NULL AND p.is_active = TRUE
				ORDER BY random()
				LIMIT v_num_items
			LOOP
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

	-- 8. A handful of CANCELLED sales (recent) -------------------------------
	FOR i IN 1..6 LOOP
		v_sold_at := CURRENT_DATE - floor(random() * 20)::int * INTERVAL '1 day'
			+ (9 + floor(random() * 11)) * INTERVAL '1 hour';
		v_cashier := v_cashiers[1 + floor(random() * array_length(v_cashiers, 1))::int];

		INSERT INTO sales
			(sold_at, cashier_id, cash_register_session_id, status, notes,
			 subtotal_amount, total_amount, paid_amount, change_amount,
			 cancelled_at, cancelled_by, cancel_reason, created_at, updated_at)
		VALUES
			(v_sold_at, v_cashier, v_session_id, 'cancelled', 'demo-seed',
			 0, 0, 0, 0,
			 v_sold_at + INTERVAL '10 minutes', v_admin_id, 'Customer changed mind', v_sold_at, v_sold_at)
		RETURNING id INTO v_sale_id;

		v_subtotal := 0;
		FOR v_sku IN
			SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name,
				ps.sku, u.code AS unit, ps.selling_price
			FROM product_skus ps
			JOIN products p ON p.id = ps.product_id
			JOIN units u ON u.id = ps.unit_id
			WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
			ORDER BY random() LIMIT 2
		LOOP
			v_qty := (1 + floor(random() * 3))::numeric;
			INSERT INTO sale_items
				(sale_id, product_id, product_sku_id, product_name, product_sku,
				 unit, quantity, unit_price, created_at)
			VALUES
				(v_sale_id, v_sku.product_id, v_sku.sku_id, v_sku.product_name, v_sku.sku,
				 v_sku.unit, v_qty, v_sku.selling_price, v_sold_at);
			v_subtotal := v_subtotal + (v_qty * v_sku.selling_price);
		END LOOP;

		v_paid := ceil(v_subtotal / 5000.0) * 5000;
		UPDATE sales SET
			subtotal_amount = v_subtotal, total_amount = v_subtotal,
			paid_amount = v_paid, change_amount = v_paid - v_subtotal
		WHERE id = v_sale_id;
	END LOOP;

	-- 9. Sale returns (completed + cancelled) against recent completed sales --
	i := 0;
	FOR v_rec IN
		SELECT s.id AS sale_id, s.cashier_id, s.sold_at
		FROM sales s
		WHERE s.notes = 'demo-seed' AND s.status = 'completed'
			AND s.sold_at >= CURRENT_DATE - INTERVAL '25 days'
		ORDER BY s.sold_at DESC
		LIMIT 8
	LOOP
		i := i + 1;

		INSERT INTO sale_returns
			(sale_id, returned_at, cashier_id, cash_register_session_id,
			 subtotal_amount, refund_amount, status, reason, notes,
			 cancelled_at, cancelled_by, cancel_reason, created_at)
		VALUES (
			v_rec.sale_id, v_rec.sold_at + INTERVAL '1 day', v_rec.cashier_id, v_session_id,
			0, 0,
			(CASE WHEN i > 6 THEN 'cancelled' ELSE 'completed' END)::sale_return_status,
			'Damaged item', 'demo-seed',
			CASE WHEN i > 6 THEN v_rec.sold_at + INTERVAL '1 day 1 hour' END,
			CASE WHEN i > 6 THEN v_admin_id END,
			CASE WHEN i > 6 THEN 'Return voided' END,
			v_rec.sold_at + INTERVAL '1 day'
		)
		RETURNING id INTO v_ret_id;

		v_subtotal := 0;
		-- Return the first line item of the sale (full quantity).
		FOR v_sku IN
			SELECT si.id AS sale_item_id, si.product_id, si.product_sku_id,
				si.product_name, si.product_sku, si.unit, si.quantity, si.unit_price
			FROM sale_items si
			WHERE si.sale_id = v_rec.sale_id
			ORDER BY si.id ASC
			LIMIT 1
		LOOP
			INSERT INTO sale_return_items
				(sale_return_id, sale_item_id, product_id, product_sku_id, product_name,
				 product_sku, unit, quantity, unit_price, restock, reason, created_at)
			VALUES
				(v_ret_id, v_sku.sale_item_id, v_sku.product_id, v_sku.product_sku_id, v_sku.product_name,
				 v_sku.product_sku, v_sku.unit, v_sku.quantity, v_sku.unit_price, TRUE, 'Damaged item',
				 v_rec.sold_at + INTERVAL '1 day');
			v_subtotal := v_subtotal + (v_sku.quantity * v_sku.unit_price);
		END LOOP;

		UPDATE sale_returns SET subtotal_amount = v_subtotal, refund_amount = v_subtotal
		WHERE id = v_ret_id;
	END LOOP;

	-- 10. Cash drawer movements (all 4 types + a cancelled one) ---------------
	INSERT INTO cash_drawer_movements
		(cash_register_session_id, occurred_at, created_by, movement_type, direction, amount, reason, notes, status)
	VALUES
		(v_session_id, now() - INTERVAL '2 days', v_admin_id,     'cash_in',   'in',  200000, 'Add float',        'demo-seed', 'completed'),
		(v_session_id, now() - INTERVAL '2 days', v_cashiers[1],  'cash_in',   'in',  100000, 'Change top-up',    'demo-seed', 'completed'),
		(v_session_id, now() - INTERVAL '1 day',  v_admin_id,     'cash_out',  'out', 150000, 'Bank deposit',     'demo-seed', 'completed'),
		(v_session_id, now() - INTERVAL '1 day',  v_cashiers[1],  'cash_drop', 'out', 300000, 'Safe drop',        'demo-seed', 'completed'),
		(v_session_id, now() - INTERVAL '6 hours', v_admin_id,    'petty_cash','out', 75000,  'Office supplies',  'demo-seed', 'completed');

	INSERT INTO cash_drawer_movements
		(cash_register_session_id, occurred_at, created_by, movement_type, direction, amount, reason, notes,
		 status, cancelled_at, cancelled_by, cancel_reason)
	VALUES
		(v_session_id, now() - INTERVAL '3 hours', v_admin_id, 'cash_out', 'out', 50000, 'Wrong entry', 'demo-seed',
		 'cancelled', now() - INTERVAL '2 hours', v_admin_id, 'Entered by mistake');

	-- 11. Purchase orders with items, in every status -------------------------
	FOR v_rec IN
		SELECT * FROM (VALUES
			('draft'), ('draft'),
			('ordered'), ('ordered'),
			('partially_received'),
			('received'), ('received'),
			('cancelled')
		) AS t(status)
	LOOP
		v_status := v_rec.status;

		INSERT INTO purchase_orders
			(supplier_id, order_date, expected_date, status, subtotal_amount, notes, created_by,
			 ordered_by, ordered_at, cancelled_at, cancelled_by, cancel_reason, created_at)
		VALUES (
			v_supplier_ids[1 + floor(random() * array_length(v_supplier_ids, 1))::int],
			CURRENT_DATE - floor(random() * 40)::int,
			CURRENT_DATE + floor(random() * 10)::int,
			v_status::purchase_order_status, 0, 'demo-seed', v_admin_id,
			CASE WHEN v_status IN ('ordered', 'partially_received', 'received') THEN v_admin_id END,
			CASE WHEN v_status IN ('ordered', 'partially_received', 'received') THEN now() - INTERVAL '20 days' END,
			CASE WHEN v_status = 'cancelled' THEN now() - INTERVAL '15 days' END,
			CASE WHEN v_status = 'cancelled' THEN v_admin_id END,
			CASE WHEN v_status = 'cancelled' THEN 'Supplier out of stock' END,
			now() - INTERVAL '25 days'
		)
		RETURNING id INTO v_po_id;

		v_num_items := 1 + floor(random() * 3)::int; -- 1..3 distinct SKUs
		FOR v_sku IN
			SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name,
				ps.sku, u.code AS unit, ps.purchase_price
			FROM product_skus ps
			JOIN products p ON p.id = ps.product_id
			JOIN units u ON u.id = ps.unit_id
			WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
			ORDER BY random() LIMIT v_num_items
		LOOP
			v_qty := (10 + floor(random() * 40))::numeric; -- ordered qty
			INSERT INTO purchase_order_items
				(purchase_order_id, product_id, product_sku_id, product_name, product_sku,
				 unit, quantity_ordered, quantity_received, unit_cost)
			VALUES (
				v_po_id, v_sku.product_id, v_sku.sku_id, v_sku.product_name, v_sku.sku,
				v_sku.unit, v_qty,
				CASE
					WHEN v_status = 'received' THEN v_qty
					WHEN v_status = 'partially_received' THEN floor(v_qty / 2)
					ELSE 0
				END,
				GREATEST(v_sku.purchase_price, 1000)
			);
		END LOOP;

		UPDATE purchase_orders po
		SET subtotal_amount = COALESCE((
			SELECT SUM(line_total) FROM purchase_order_items WHERE purchase_order_id = po.id
		), 0)
		WHERE po.id = v_po_id;

		IF v_status = 'received' THEN
			v_received_pos := v_received_pos || v_po_id;
		ELSIF v_status = 'partially_received' THEN
			v_partial_pos := v_partial_pos || v_po_id;
		ELSIF v_status = 'ordered' THEN
			v_ordered_pos := v_ordered_pos || v_po_id;
		END IF;
	END LOOP;

	-- 12. Goods receipts with items, in every status --------------------------
	-- Posted GRs against received / partially_received POs (these add stock,
	-- so we also write matching purchase_receipt stock movements).
	FOREACH v_po_id IN ARRAY (v_received_pos || v_partial_pos) LOOP
		INSERT INTO goods_receipts
			(purchase_order_id, received_at, status, received_by, posted_at, posted_by, notes, created_at)
		VALUES
			(v_po_id, now() - INTERVAL '10 days', 'posted', v_admin_id, now() - INTERVAL '10 days', v_admin_id,
			 'demo-seed', now() - INTERVAL '10 days')
		RETURNING id INTO v_gr_id;

		FOR v_rec IN
			SELECT poi.id AS poi_id, poi.product_id, poi.product_sku_id, poi.product_name,
				poi.product_sku, poi.unit, poi.unit_cost,
				GREATEST(poi.quantity_received, 1) AS qty
			FROM purchase_order_items poi
			WHERE poi.purchase_order_id = v_po_id
		LOOP
			INSERT INTO goods_receipt_items
				(goods_receipt_id, purchase_order_item_id, product_id, product_sku_id, product_name,
				 product_sku, unit, quantity_received, unit_cost)
			VALUES
				(v_gr_id, v_rec.poi_id, v_rec.product_id, v_rec.product_sku_id, v_rec.product_name,
				 v_rec.product_sku, v_rec.unit, v_rec.qty, v_rec.unit_cost);

			INSERT INTO stock_movements
				(product_id, product_sku_id, goods_receipt_id, movement_type, quantity,
				 stock_before, stock_after, notes, created_by, created_at)
			VALUES
				(v_rec.product_id, v_rec.product_sku_id, v_gr_id, 'purchase_receipt', v_rec.qty,
				 100, 100 + v_rec.qty, 'demo-seed', v_admin_id, now() - INTERVAL '10 days');
		END LOOP;
	END LOOP;

	-- Draft + cancelled GRs against ordered POs (no stock impact).
	IF array_length(v_ordered_pos, 1) >= 1 THEN
		FOR k IN 1..LEAST(2, array_length(v_ordered_pos, 1)) LOOP
			v_po_id := v_ordered_pos[k];
			INSERT INTO goods_receipts
				(purchase_order_id, received_at, status, received_by, notes, created_at,
				 cancelled_at, cancelled_by, cancel_reason)
			VALUES
				(v_po_id, now() - INTERVAL '5 days',
				 CASE WHEN k = 1 THEN 'draft' ELSE 'cancelled' END::goods_receipt_status,
				 v_admin_id, 'demo-seed', now() - INTERVAL '5 days',
				 CASE WHEN k = 2 THEN now() - INTERVAL '4 days' END,
				 CASE WHEN k = 2 THEN v_admin_id END,
				 CASE WHEN k = 2 THEN 'Wrong PO selected' END)
			RETURNING id INTO v_gr_id;

			FOR v_rec IN
				SELECT poi.id AS poi_id, poi.product_id, poi.product_sku_id, poi.product_name,
					poi.product_sku, poi.unit, poi.unit_cost, poi.quantity_ordered
				FROM purchase_order_items poi
				WHERE poi.purchase_order_id = v_po_id
			LOOP
				INSERT INTO goods_receipt_items
					(goods_receipt_id, purchase_order_item_id, product_id, product_sku_id, product_name,
					 product_sku, unit, quantity_received, unit_cost)
				VALUES
					(v_gr_id, v_rec.poi_id, v_rec.product_id, v_rec.product_sku_id, v_rec.product_name,
					 v_rec.product_sku, v_rec.unit, v_rec.quantity_ordered, v_rec.unit_cost);
			END LOOP;
		END LOOP;
	END IF;

	-- 13. Stock adjustments with items, in every status -----------------------
	FOR v_rec IN
		SELECT * FROM (VALUES
			('draft',     'Stock opname'),
			('posted',    'Stock opname'),
			('posted',    'Damaged goods'),
			('cancelled', 'Data entry error')
		) AS t(status, reason)
	LOOP
		v_status := v_rec.status;

		INSERT INTO stock_adjustments
			(adjusted_at, status, reason, notes, created_by, posted_at, posted_by,
			 cancelled_at, cancelled_by, cancel_reason, created_at)
		VALUES (
			now() - floor(random() * 50)::int * INTERVAL '1 day',
			v_status::stock_adjustment_status, v_rec.reason, 'demo-seed', v_admin_id,
			CASE WHEN v_status = 'posted'    THEN now() - INTERVAL '8 days' END,
			CASE WHEN v_status = 'posted'    THEN v_admin_id END,
			CASE WHEN v_status = 'cancelled' THEN now() - INTERVAL '7 days' END,
			CASE WHEN v_status = 'cancelled' THEN v_admin_id END,
			CASE WHEN v_status = 'cancelled' THEN 'Re-counted, no change needed' END,
			now() - INTERVAL '12 days'
		)
		RETURNING id INTO v_sa_id;

		SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name, ps.sku, u.code AS unit, ps.stock
		INTO v_sku
		FROM product_skus ps
		JOIN products p ON p.id = ps.product_id
		JOIN units u ON u.id = ps.unit_id
		WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
		ORDER BY random() LIMIT 1;

		v_qty := GREATEST(v_sku.stock, 0);
		INSERT INTO stock_adjustment_items
			(stock_adjustment_id, product_id, product_sku_id, product_name, product_sku,
			 unit, stock_before, stock_after, notes)
		VALUES
			(v_sa_id, v_sku.product_id, v_sku.sku_id, v_sku.product_name, v_sku.sku,
			 v_sku.unit, v_qty, v_qty + 5, 'demo-seed'); -- always changes

		IF v_status = 'posted' THEN
			INSERT INTO stock_movements
				(product_id, product_sku_id, stock_adjustment_id, movement_type, quantity,
				 stock_before, stock_after, notes, created_by, created_at)
			VALUES
				(v_sku.product_id, v_sku.sku_id, v_sa_id, 'adjustment', 5,
				 v_qty, v_qty + 5, 'demo-seed', v_admin_id, now() - INTERVAL '8 days');
		END IF;
	END LOOP;

	-- 14. Stock conversions with items, in every status -----------------------
	SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name, ps.sku, u.code AS unit, ps.stock
	INTO v_from_sku
	FROM product_skus ps JOIN products p ON p.id = ps.product_id JOIN units u ON u.id = ps.unit_id
	WHERE ps.product_id = v_conv_product AND ps.deleted_at IS NULL AND ps.is_active = TRUE
	ORDER BY ps.id ASC LIMIT 1;

	SELECT ps.id AS sku_id, ps.product_id, p.name AS product_name, ps.sku, u.code AS unit, ps.stock
	INTO v_to_sku
	FROM product_skus ps JOIN products p ON p.id = ps.product_id JOIN units u ON u.id = ps.unit_id
	WHERE ps.product_id = v_conv_product AND ps.deleted_at IS NULL AND ps.is_active = TRUE
		AND ps.id <> v_from_sku.sku_id
	ORDER BY ps.id ASC LIMIT 1;

	FOR v_rec IN SELECT * FROM (VALUES ('draft'), ('posted'), ('cancelled')) AS t(status) LOOP
		v_status := v_rec.status;

		INSERT INTO stock_conversions
			(converted_at, status, notes, created_by, posted_at, posted_by,
			 cancelled_at, cancelled_by, cancel_reason, created_at)
		VALUES (
			now() - floor(random() * 40)::int * INTERVAL '1 day',
			v_status::stock_conversion_status, 'demo-seed', v_admin_id,
			CASE WHEN v_status = 'posted'    THEN now() - INTERVAL '6 days' END,
			CASE WHEN v_status = 'posted'    THEN v_admin_id END,
			CASE WHEN v_status = 'cancelled' THEN now() - INTERVAL '5 days' END,
			CASE WHEN v_status = 'cancelled' THEN v_admin_id END,
			CASE WHEN v_status = 'cancelled' THEN 'Conversion not needed' END,
			now() - INTERVAL '9 days'
		)
		RETURNING id INTO v_sc_id;

		INSERT INTO stock_conversion_items
			(stock_conversion_id,
			 from_product_id, from_product_sku_id, from_product_name, from_product_sku, from_unit,
			 from_stock_before, from_quantity, from_stock_after,
			 to_product_id, to_product_sku_id, to_product_name, to_product_sku, to_unit,
			 to_stock_before, to_quantity, to_stock_after, notes)
		VALUES (
			v_sc_id,
			v_from_sku.product_id, v_from_sku.sku_id, v_from_sku.product_name, v_from_sku.sku, v_from_sku.unit,
			10, 2, 8,
			v_to_sku.product_id, v_to_sku.sku_id, v_to_sku.product_name, v_to_sku.sku, v_to_sku.unit,
			20, 48, 68, 'demo-seed'); -- 2 boxes (24/box) -> 48 pcs

		IF v_status = 'posted' THEN
			INSERT INTO stock_movements
				(product_id, product_sku_id, stock_conversion_id, movement_type, quantity,
				 stock_before, stock_after, notes, created_by, created_at)
			VALUES
				(v_from_sku.product_id, v_from_sku.sku_id, v_sc_id, 'conversion_out', 2,
				 10, 8, 'demo-seed', v_admin_id, now() - INTERVAL '6 days'),
				(v_to_sku.product_id, v_to_sku.sku_id, v_sc_id, 'conversion_in', 48,
				 20, 68, 'demo-seed', v_admin_id, now() - INTERVAL '6 days');
		END IF;
	END LOOP;

	-- 15. Stock movements for the remaining movement types --------------------
	-- 'sale' movements for a sample of recent completed sale lines.
	FOR v_rec IN
		SELECT si.id AS sale_item_id, si.sale_id, si.product_id, si.product_sku_id,
			si.quantity, s.sold_at, s.cashier_id
		FROM sale_items si
		JOIN sales s ON s.id = si.sale_id
		WHERE s.notes = 'demo-seed' AND s.status = 'completed'
			AND s.sold_at >= CURRENT_DATE - INTERVAL '3 days'
		ORDER BY s.sold_at DESC
		LIMIT 50
	LOOP
		INSERT INTO stock_movements
			(product_id, product_sku_id, sale_id, sale_item_id, movement_type, quantity,
			 stock_before, stock_after, notes, created_by, created_at)
		VALUES
			(v_rec.product_id, v_rec.product_sku_id, v_rec.sale_id, v_rec.sale_item_id, 'sale',
			 v_rec.quantity, v_rec.quantity + 50, 50, 'demo-seed', v_rec.cashier_id, v_rec.sold_at);
	END LOOP;

	-- 'sale_return' movements for completed returns (restocked).
	FOR v_rec IN
		SELECT sri.id AS sri_id, sri.product_id, sri.product_sku_id, sri.quantity,
			sr.id AS ret_id, sr.returned_at, sr.cashier_id
		FROM sale_return_items sri
		JOIN sale_returns sr ON sr.id = sri.sale_return_id
		WHERE sr.notes = 'demo-seed' AND sr.status = 'completed' AND sri.restock = TRUE
	LOOP
		INSERT INTO stock_movements
			(product_id, product_sku_id, sale_return_id, sale_return_item_id, movement_type, quantity,
			 stock_before, stock_after, notes, created_by, created_at)
		VALUES
			(v_rec.product_id, v_rec.product_sku_id, v_rec.ret_id, v_rec.sri_id, 'sale_return',
			 v_rec.quantity, 50, 50 + v_rec.quantity, 'demo-seed', v_rec.cashier_id, v_rec.returned_at);
	END LOOP;

	-- Manual stock_in / stock_out / correction examples on random SKUs.
	FOR v_sku IN
		SELECT ps.id AS sku_id, ps.product_id
		FROM product_skus ps
		WHERE ps.deleted_at IS NULL AND ps.is_active = TRUE
		ORDER BY random() LIMIT 3
	LOOP
		INSERT INTO stock_movements
			(product_id, product_sku_id, movement_type, quantity, stock_before, stock_after, notes, created_by, created_at)
		VALUES
			(v_sku.product_id, v_sku.sku_id, 'stock_in',   10, 40, 50, 'demo-seed opening stock', v_admin_id, now() - INTERVAL '20 days'),
			(v_sku.product_id, v_sku.sku_id, 'stock_out',   3, 50, 47, 'demo-seed wastage',        v_admin_id, now() - INTERVAL '14 days'),
			(v_sku.product_id, v_sku.sku_id, 'correction',  2, 47, 49, 'demo-seed recount',        v_admin_id, now() - INTERVAL '7 days');
	END LOOP;

	-- 16. Payroll slips for the last 3 months per active demo employee --------
	FOR v_rec IN
		SELECT id, name, position, base_salary
		FROM employees
		WHERE phone = 'demo-seed' AND is_active = TRUE AND deleted_at IS NULL
	LOOP
		FOR k IN 1..3 LOOP
			v_period_start := date_trunc('month', CURRENT_DATE - (k || ' months')::interval)::date;
			v_period_end := (date_trunc('month', CURRENT_DATE - (k || ' months')::interval)
				+ INTERVAL '1 month' - INTERVAL '1 day')::date;

			INSERT INTO payroll_slips
				(employee_id, employee_name, employee_position, period_start, period_end,
				 base_salary, bonus, deduction, created_by)
			VALUES (
				v_rec.id, v_rec.name, v_rec.position, v_period_start, v_period_end,
				v_rec.base_salary,
				floor(random() * 500000)::int,
				floor(random() * 200000)::int,
				v_admin_id
			)
			ON CONFLICT (employee_id, period_start, period_end) DO NOTHING;
		END LOOP;
	END LOOP;

	-- 17. Make a handful of SKUs low on stock (low-stock table + KPI) ---------
	WITH picks AS (
		SELECT id, row_number() OVER (ORDER BY id) AS rn
		FROM product_skus
		WHERE deleted_at IS NULL AND is_active = TRUE
		ORDER BY id
		LIMIT 5
	)
	UPDATE product_skus ps SET
		minimum_stock = v.min_stock,
		stock = v.cur_stock
	FROM picks
	JOIN (VALUES (1, 24, 10), (2, 30, 12), (3, 5, 2), (4, 1, 0), (5, 2, 1)) AS v(rn, min_stock, cur_stock)
		ON v.rn = picks.rn
	WHERE ps.id = picks.id;

	RAISE NOTICE 'Demo seed complete.';
END $$;

COMMIT;
