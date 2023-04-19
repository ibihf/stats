CREATE TABLE IF NOT EXISTS league_names (
  id SERIAL PRIMARY KEY NOT NULL,
  language INTEGER NOT NULL,
  name VARCHAR(255) NOT NULL,
  league INTEGER NOT NULL,
  CONSTRAINT language_fk
    FOREIGN KEY(language)
      REFERENCES supported_languages(id)
      ON DELETE RESTRICT,
  CONSTRAINT league_fk
    FOREIGN KEY(league)
      REFERENCES leagues(id)
      ON DELETE RESTRICT
);
