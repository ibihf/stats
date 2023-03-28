-- Add up migration script here
CREATE TABLE IF NOT EXISTS positions (
  id SERIAL PRIMARY KEY NOT NULL,
  name VARCHAR(32) NOT NULL,
  -- the short version, which should usually one character can be 2 charaters in some rare cases.
  -- for example, in Goalball you'd have L, R, and C.
  -- In hockey, you'd have C, D, LW and RW.
  short_name VARCHAR(2) NOT NULL
);
