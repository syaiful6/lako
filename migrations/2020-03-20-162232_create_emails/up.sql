-- Your SQL goes here
CREATE OR REPLACE FUNCTION lako_random_string(int4) RETURNS text AS $$
  SELECT (array_to_string(array(
    SELECT substr(
      'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789',
      floor(random() * 62)::int4 + 1,
      1
    ) FROM generate_series(1, $1)
  ), ''))
$$ LANGUAGE SQL;

CREATE table emails (
  id                 SERIAL PRIMARY KEY,
  user_id            INTEGER NOT NULL REFERENCES users,
  is_primary         BOOLEAN NOT NULL DEFAULT false,
  email              VARCHAR NOT NULL UNIQUE,
  token              TEXT NOT NULL DEFAULT lako_random_string(26),
  verified           BOOLEAN NOT NULL DEFAULT false,
  token_generated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX emails_user_id_fk ON emails(user_id);

CREATE OR REPLACE FUNCTION lako_reconfirm_email_on_email_change() RETURNS trigger AS $$
  BEGIN
    IF NEW.email IS DISTINCT FROM OLD.email THEN
      NEW.token := lako_random_string(26);
      NEW.verified := false;
    END IF;
    RETURN NEW;
  END
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_lako_reconfirm BEFORE UPDATE
ON emails
FOR EACH ROW EXECUTE PROCEDURE lako_reconfirm_email_on_email_change();
