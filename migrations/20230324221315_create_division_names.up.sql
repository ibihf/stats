CREATE TABLE IF NOT EXISTS division_names (
  id SERIAL PRIMARY KEY NOT NULL,
  language INTEGER NOT NULL,
  name VARCHAR(255) NOT NULL,
  division INTEGER NOT NULL,
  CONSTRAINT language_fk
    FOREIGN KEY(language)
      REFERENCES supported_languages(id)
      ON DELETE RESTRICT,
  CONSTRAINT division_fk
    FOREIGN KEY(division)
      REFERENCES divisions(id)
      ON DELETE RESTRICT
);
