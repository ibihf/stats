-- Add up migration script here
CREATE TABLE IF NOT EXISTS games (
  id SERIAL PRIMARY KEY NOT NULL,
  -- a possibly null name for the game; this allows there to be special names like "Gold Medal Game"
  name VARCHAR(255),
  team_home INTEGER NOT NULL,
  team_away INTEGER NOT NULL,
  -- home and away teams need to actually be teams
  CONSTRAINT team_home_fk
    FOREIGN KEY(team_home)
      REFERENCES teams(id)
      ON DELETE RESTRICT,
  CONSTRAINT team_away_fk
    FOREIGN KEY(team_away)
      REFERENCES teams(id)
      ON DELETE RESTRICT
);
