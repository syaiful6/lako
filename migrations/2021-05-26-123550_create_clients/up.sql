-- Your SQL goes here
CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR NOT NULL,
    company_name VARCHAR(255) NOT NULL,
    address_1 VARCHAR(512) NOT NULL,
    address_2 VARCHAR(512) NOT NULL,
    city VARCHAR(255) NOT NULL,
    state VARCHAR(255) NOT NULL,
    zip_code VARCHAR(255) NOT NULL,
    country VARCHAR(255) NOT NULL,
    website VARCHAR(255) NOT NULL,
    notes TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_clients_user_id_fk ON clients(user_id);
CREATE INDEX idx_clients_email ON clients(email);

SELECT diesel_manage_updated_at('clients');
