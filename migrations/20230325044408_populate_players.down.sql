-- Add down migration script here
DELETE FROM players
  WHERE id BETWEEN 1 AND 31;
