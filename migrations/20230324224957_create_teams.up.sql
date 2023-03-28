-- Add up migration script here
CREATE TABLE IF NOT EXISTS teams (
  id SERIAL PRIMARY KEY NOT NULL,
  name VARCHAR(255) NOT NULL,
  division INTEGER NOT NULL,
  CONSTRAINT division_fk
    FOREIGN KEY(division)
      REFERENCES divisions(id)
      ON DELETE RESTRICT
);
