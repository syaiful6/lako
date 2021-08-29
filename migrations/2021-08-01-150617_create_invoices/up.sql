-- Your SQL goes here
CREATE TABLE invoices (
    id SERIAL PRIMARY KEY,
    invoice_id UUID NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users ON DELETE CASCADE,
    client_id INTEGER NOT NULL REFERENCES clients ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES companies ON DELETE CASCADE,
    invoice_number VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status smallint NOT NULL DEFAULT 0,
    billing_reason smallint NOT NULL DEFAULT 0,
    due_date TIMESTAMP NULL,
    invoice_date TIMESTAMP NULL,
    last_send_date TIMESTAMP NULL,
    amount DECIMAL(30, 2) NOT NULL DEFAULT 0,
    balance DECIMAL(30, 2) NOT NULL DEFAULT 0,
    discount DECIMAL(30, 2) NOT NULL DEFAULT 0,
    tax DECIMAL(30, 2) NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_invoice_id ON invoices(invoice_id);
CREATE UNIQUE INDEX idx_invoices_client_invoice_number ON invoices(client_id, invoice_number);
CREATE INDEX idx_users_client_fk ON invoices(user_id, client_id);
CREATE INDEX idx_invoices_created_at ON invoices(created_at);

SELECT diesel_manage_updated_at('invoices');
