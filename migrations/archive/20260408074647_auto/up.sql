CREATE SCHEMA IF NOT EXISTS activitylog;

CREATE SCHEMA IF NOT EXISTS blogs;

CREATE SCHEMA IF NOT EXISTS products;

CREATE SCHEMA IF NOT EXISTS user;

ALTER TABLE products.products ADD UNIQUE (name);

CREATE INDEX IF NOT EXISTS idx_products_is_active ON products.products (is_active);