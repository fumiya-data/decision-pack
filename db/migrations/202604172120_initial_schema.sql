CREATE TABLE IF NOT EXISTS customers (
    customer_id TEXT PRIMARY KEY,
    full_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    address_line TEXT,
    city TEXT,
    region TEXT,
    postal_code TEXT,
    country TEXT,
    birth_date DATE,
    signup_date DATE,
    last_purchase_date DATE,
    status TEXT,
    tier TEXT,
    preferred_language TEXT,
    marketing_opt_in BOOLEAN,
    total_spend NUMERIC(14, 2),
    order_count INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS customer_load_issues (
    issue_id BIGSERIAL PRIMARY KEY,
    customer_id TEXT,
    column_name TEXT NOT NULL,
    issue_code TEXT NOT NULL,
    raw_value TEXT,
    message TEXT NOT NULL,
    source_row_number BIGINT,
    run_id TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS items (
    item_id TEXT PRIMARY KEY,
    item_name TEXT NOT NULL,
    category TEXT NOT NULL,
    uom TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    lead_time_days INTEGER NOT NULL DEFAULT 0,
    moq INTEGER,
    lot_size INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory_balance (
    item_id TEXT PRIMARY KEY REFERENCES items (item_id) ON DELETE CASCADE,
    on_hand INTEGER NOT NULL DEFAULT 0 CHECK (on_hand >= 0),
    on_order INTEGER NOT NULL DEFAULT 0 CHECK (on_order >= 0),
    reserved_qty INTEGER NOT NULL DEFAULT 0 CHECK (reserved_qty >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory_movements (
    movement_id BIGSERIAL PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES items (item_id) ON DELETE CASCADE,
    movement_type TEXT NOT NULL,
    qty INTEGER NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reference_type TEXT,
    reference_id TEXT
);

CREATE TABLE IF NOT EXISTS orders (
    order_id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers (customer_id) ON DELETE RESTRICT,
    ordered_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'JPY',
    total_amount NUMERIC(14, 2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS order_items (
    order_id TEXT NOT NULL REFERENCES orders (order_id) ON DELETE CASCADE,
    line_no INTEGER NOT NULL,
    item_id TEXT NOT NULL REFERENCES items (item_id) ON DELETE RESTRICT,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(14, 2),
    line_amount NUMERIC(14, 2),
    PRIMARY KEY (order_id, line_no)
);

CREATE TABLE IF NOT EXISTS customer_item_next_buy_score (
    customer_id TEXT NOT NULL REFERENCES customers (customer_id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES items (item_id) ON DELETE CASCADE,
    score DOUBLE PRECISION NOT NULL,
    rank INTEGER NOT NULL CHECK (rank > 0),
    as_of TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (customer_id, item_id, as_of)
);

CREATE TABLE IF NOT EXISTS item_demand_forecast (
    forecast_date DATE NOT NULL,
    item_id TEXT NOT NULL REFERENCES items (item_id) ON DELETE CASCADE,
    expected_qty INTEGER NOT NULL CHECK (expected_qty >= 0),
    low_qty INTEGER NOT NULL CHECK (low_qty >= 0),
    high_qty INTEGER NOT NULL CHECK (high_qty >= 0),
    as_of TIMESTAMPTZ NOT NULL,
    source_run_id TEXT,
    PRIMARY KEY (forecast_date, item_id, as_of)
);

CREATE TABLE IF NOT EXISTS simulation_runs (
    run_id TEXT PRIMARY KEY,
    scenario_id TEXT NOT NULL,
    scenario_name TEXT NOT NULL,
    status TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    report_schema_version TEXT,
    report_json JSONB,
    report_uri TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS simulation_item_results (
    run_id TEXT NOT NULL REFERENCES simulation_runs (run_id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES items (item_id) ON DELETE CASCADE,
    risk_level TEXT,
    recommended_reorder_qty INTEGER,
    expected_stockout_qty INTEGER,
    expected_days_on_hand DOUBLE PRECISION,
    PRIMARY KEY (run_id, item_id)
);

CREATE TABLE IF NOT EXISTS etl_job_runs (
    job_id TEXT PRIMARY KEY,
    job_kind TEXT NOT NULL,
    status TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    source_uri TEXT,
    artifact_uri TEXT,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_customers_country_status ON customers (country, status);
CREATE INDEX IF NOT EXISTS idx_items_category ON items (category);
CREATE INDEX IF NOT EXISTS idx_orders_customer_ordered_at ON orders (customer_id, ordered_at DESC);
CREATE INDEX IF NOT EXISTS idx_order_items_item_id ON order_items (item_id);
CREATE INDEX IF NOT EXISTS idx_next_buy_customer_rank ON customer_item_next_buy_score (customer_id, rank);
CREATE INDEX IF NOT EXISTS idx_forecast_item_date ON item_demand_forecast (item_id, forecast_date);
CREATE INDEX IF NOT EXISTS idx_simulation_runs_status_requested_at ON simulation_runs (status, requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_job_runs_kind_requested_at ON etl_job_runs (job_kind, requested_at DESC);
