-- Add up migration script here
CREATE TABLE users (
  id SERIAL PRIMARY KEY NOT NULL,
	user_name VARCHAR(32) NOT NULL,
	pass_hash VARCHAR(512) NOT NULL
);
