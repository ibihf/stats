CREATE TABLE IF NOT EXISTS period_types (
  id SERIAL PRIMARY KEY NOT NULL,
  -- "first", "second", "third", "second overtime", "shootout"
  name VARCHAR(32) NOT NULL,
  -- "1", "2", "3", "OT", "[2-9]OT", "SO"
  -- technically 10+OT would not work, but this should be rare enough to not worry about.
  short_name VARCHAR(3) NOT NULL,
  -- default length
  default_length INTEGER NOT NULL
);
