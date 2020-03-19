-- Your SQL goes here
CREATE table emails (
  id         SERIAL PRIMARY KEY,
  user_id    INTEGER NOT NULL,
  email      VARCHAR NOT NULL UNIQUE,
  verified   BOOLEAN DEFAULT false NOT NULL
);

CREATE table email_tokens (
  id         SERIAL PRIMARY KEY,
  email_id   INTEGER NOT NULL UNIQUE REFERENCES emails,
  token      VARCHAR NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now()
);
