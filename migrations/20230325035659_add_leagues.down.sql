-- Add down migration script here
DELETE FROM league_names WHERE id=1;
DELETE FROM leagues WHERE id=1;
