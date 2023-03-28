-- Add down migration script here
DELETE FROM teams WHERE id=1 OR id=2;
