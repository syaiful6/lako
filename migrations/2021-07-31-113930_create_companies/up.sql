-- Your SQL goes here
CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    address_1 VARCHAR(512) NOT NULL,
    address_2 VARCHAR(512) NOT NULL,
    city VARCHAR(255) NOT NULL,
    state VARCHAR(255) NOT NULL,
    zip_code VARCHAR(255) NOT NULL,
    country VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_company_user_id_fk ON companies(user_id);

SELECT diesel_manage_updated_at('companies');
