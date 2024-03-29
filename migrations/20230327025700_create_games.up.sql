-- Add up migration script here
CREATE TABLE IF NOT EXISTS games (
  id SERIAL PRIMARY KEY NOT NULL,
  -- what divison is the game a part of (usefl for stats)
  division INTEGER NOT NULL,
  team_home INTEGER NOT NULL,
  team_away INTEGER NOT NULL,
  start_at TIMESTAMPTZ NOT NULL,
  end_at TIMESTAMPTZ NOT NULL,
  -- home and away teams need to actually be teams
  CONSTRAINT team_home_fk
    FOREIGN KEY(team_home)
      REFERENCES teams(id)
      ON DELETE RESTRICT,
  CONSTRAINT team_away_fk
    FOREIGN KEY(team_away)
      REFERENCES teams(id)
      ON DELETE RESTRICT,
  -- is divison real
  CONSTRAINT division_fk
    FOREIGN KEY(division)
      REFERENCES divisions(id)
      ON DELETE RESTRICT
);
