-- Add up migration script here
CREATE TABLE IF NOT EXISTS players (
  id SERIAL PRIMARY KEY NOT NULL,
  first_names VARCHAR(255) NOT NULL,
  last_name VARCHAR(32) NOT NULL,
  height_cm INTEGER,
  weight_kg INTEGER
);
