-- Your SQL goes here
CREATE TABLE invoice_items(
    id SERIAL PRIMARY KEY,
    invoice_id INTEGER NOT NULL REFERENCES invoices ON DELETE CASCADE,
    name VARCHAR(256) NOT NULL,
    description TEXT NOT NULL,
    amount DECIMAL(30, 2) NOT NULL DEFAULT 0,
    quantity DECIMAL(12, 2) NOT NULL DEFAULT 0
);
