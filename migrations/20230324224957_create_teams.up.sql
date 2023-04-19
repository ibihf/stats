-- Add up migration script here
CREATE TABLE IF NOT EXISTS teams (
  id SERIAL PRIMARY KEY NOT NULL,
  division INTEGER NOT NULL,
  -- possibly add an image
  image VARCHAR(255),
  CONSTRAINT division_fk
    FOREIGN KEY(division)
      REFERENCES divisions(id)
      ON DELETE RESTRICT
);
