-- Add down migration script here
DELETE FROM team_names WHERE id BETWEEN 1 AND 4;
DELETE FROM teams WHERE id=1 OR id=2;
