-- Your SQL goes here
CREATE TABLE users (
  id                SERIAL PRIMARY KEY,
  role              smallint NOT NULL DEFAULT 2,
  username          VARCHAR NOT NULL UNIQUE,
  hashed_password   VARCHAR NOT NULL,
  profile_name      VARCHAR(255) NOT NULL,
  profile_image     VARCHAR(255) NOT NULL,
  joined_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('users');