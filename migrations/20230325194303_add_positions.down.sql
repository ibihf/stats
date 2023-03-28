-- Add down migration script here
DELETE FROM positions
  WHERE id BETWEEN 1 AND 7;
