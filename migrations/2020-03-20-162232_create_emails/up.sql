-- Your SQL goes here
-- Your SQL goes here
CREATE table emails (
  id         SERIAL PRIMARY KEY,
  user_id    INTEGER NOT NULL REFERENCES users,
  email      VARCHAR NOT NULL UNIQUE,
  verified   BOOLEAN DEFAULT false NOT NULL
);

CREATE INDEX emails_user_id_fk ON emails(user_id);

CREATE table email_tokens (
  id         SERIAL PRIMARY KEY,
  email_id   INTEGER NOT NULL UNIQUE REFERENCES emails,
  token      VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);