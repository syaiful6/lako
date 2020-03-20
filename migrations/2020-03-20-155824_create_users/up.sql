-- Your SQL goes here
CREATE TABLE users (
  id                SERIAL PRIMARY KEY,
  username          VARCHAR NOT NULL UNIQUE,
  hashed_password   VARCHAR NOT NULL,
  role              SMALLINT NOT NULL DEFAULT 0,
  profile_name      VARCHAR(255) NOT NULL,
  profile_image     VARCHAR(255) NOT NULL,
  joined_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('users');