-- Add up migration script here
CREATE TABLE IF NOT EXISTS shots (
  id SERIAL PRIMARY KEY NOT NULL,
  
  -- video timestampt if known; seconds offset from beginning of video
  video_timestamp INTEGER,
  -- player that blocked the shot, if applicable
  blocker INTEGER,
  -- on net; did it go towards the goalie (this does not say whether it went in or not)
  on_net BOOLEAN NOT NULL,
  -- did the puck go in?
  goal BOOLEAN NOT NULL,
  -- what team was the shooter on
  shooter_team INTEGER NOT NULL,
  -- which player is the shooter
  shooter INTEGER NOT NULL,
  -- which player was the goalie
  goalie INTEGER NOT NULL,
  -- which game was this a part of
  game INTEGER NOT NULL,
  -- which period did the shot happen in
  period INTEGER NOT NULL,
  -- when did the shot happen relative to the beginning of the period
  period_time INTEGER NOT NULL,
  -- if applicable, set assistant(s)
  assistant INTEGER,
  assistant_second INTEGER,
  -- was the shooter a real player
  CONSTRAINT shooter_fk
    FOREIGN KEY(shooter)
      REFERENCES players(id)
      ON DELETE RESTRICT,
  -- was the assistant is a real player
  CONSTRAINT assistant_fk
    FOREIGN KEY(assistant)
      REFERENCES players(id)
      ON DELETE RESTRICT,
  -- was the second assistant a real player
  CONSTRAINT assistant_second_fk
    FOREIGN KEY(assistant_second)
      REFERENCES players(id)
      ON DELETE RESTRICT,
  -- was the goalie a real player
  CONSTRAINT goalie_fk
    FOREIGN KEY(goalie)
      REFERENCES players(id)
      ON DELETE RESTRICT,
  -- was the (optional) blocker a real player
  CONSTRAINT blocker_fk
    FOREIGN KEY(blocker)
      REFERENCES players(id)
      ON DELETE RESTRICT,
  -- was the shooter's team a real team
  CONSTRAINT shooter_team_fk
    FOREIGN KEY(shooter_team)
      REFERENCES teams(id)
      ON DELETE RESTRICT,
  -- is the game a real game
  CONSTRAINT game_fk
    FOREIGN KEY(game)
      REFERENCES games(id)
      ON DELETE RESTRICT,
  -- is the period refgerences a real period type
  CONSTRAINT period_fk
    FOREIGN KEY(period)
      REFERENCES periods(id)
      ON DELETE RESTRICT
);
