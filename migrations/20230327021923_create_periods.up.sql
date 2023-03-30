CREATE TABLE IF NOT EXISTS periods (
  id SERIAL PRIMARY KEY NOT NULL,
  -- which kind of period is it: 1st, 2nd, third, SO, OT, 5OT, etc.
  period_type INTEGER NOT NULL,
  -- length of period in seconds
  period_length INTEGER NOT NULL,
  -- which game does this period refer to
  game INTEGER NOT NULL,
  -- period type must exists
  CONSTRAINT period_type_fk
    FOREIGN KEY(period_type)
      REFERENCES period_types(id)
      ON DELETE RESTRICT,
  -- game must exist
  CONSTRAINT game_fk
    FOREIGN KEY(game)
      REFERENCES games(id)
      ON DELETE RESTRICT
);
