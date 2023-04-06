-- Add up migration script here
CREATE TABLE IF NOT EXISTS game_players (
  id SERIAL PRIMARY KEY NOT NULL,
  team INTEGER NOT NULL,
  game INTEGER NOT NULL,
  player INTEGER NOT NULL,
  position INTEGER NOT NULL,
  -- the number of the player, usually a 1 or 2 digit number
  player_number INTEGER NOT NULL,

  CONSTRAINT team_fk
    FOREIGN KEY(team)
      REFERENCES teams(id)
      ON DELETE RESTRICT,
  CONSTRAINT game_fk
    FOREIGN KEY(game)
      REFERENCES games(id)
      ON DELETE RESTRICT,
  CONSTRAINT player_fk
    FOREIGN KEY(player)
      REFERENCES players(id)
      ON DELETE RESTRICT,
  CONSTRAINT position_fk
    FOREIGN KEY(position)
      REFERENCES positions(id)
      ON DELETE RESTRICT
);
