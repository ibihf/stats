CREATE TABLE IF NOT EXISTS game_names (
  id SERIAL PRIMARY KEY NOT NULL,
  language INTEGER NOT NULL,
  name VARCHAR(255) NOT NULL,
  game INTEGER NOT NULL,
  CONSTRAINT language_fk
    FOREIGN KEY(language)
      REFERENCES supported_languages(id)
      ON DELETE RESTRICT,
  CONSTRAINT game_fk
    FOREIGN KEY(game)
      REFERENCES games(id)
      ON DELETE RESTRICT
);
