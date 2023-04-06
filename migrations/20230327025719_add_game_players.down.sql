-- Add down migration script here
DELETE FROM game_players
WHERE id BETWEEN 1 AND 116;
