-- Add up migration script here
CREATE TABLE IF NOT EXISTS supported_languages (
  id SERIAL PRIMARY KEY NOT NULL,
  -- this will be used in the url, like "/en-ca/...", or "/fr-ca/..."
  short_name VARCHAR(5) NOT NULL,
  -- this will be the native name of the langauge on the page where you can select your language
  native_name VARCHAR(32) NOT NULL
);
