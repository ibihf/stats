-- Add up migration script here
CREATE TABLE IF NOT EXISTS players (
  id SERIAL PRIMARY KEY NOT NULL,
  name VARCHAR(255) NOT NULL,
  height_cm INTEGER,
  weight_kg INTEGER
);
