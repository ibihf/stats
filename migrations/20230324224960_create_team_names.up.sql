CREATE TABLE IF NOT EXISTS team_names (
  id SERIAL PRIMARY KEY NOT NULL,
  language INTEGER NOT NULL,
  name VARCHAR(255) NOT NULL,
  team INTEGER NOT NULL,
  CONSTRAINT language_fk
    FOREIGN KEY(language)
      REFERENCES supported_languages(id)
      ON DELETE RESTRICT,
  CONSTRAINT team_fk
    FOREIGN KEY(team)
      REFERENCES teams(id)
      ON DELETE RESTRICT
);

