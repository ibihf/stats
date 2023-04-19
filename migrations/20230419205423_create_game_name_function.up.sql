-- Add up migration script here
CREATE FUNCTION game_name(game_id INT, lang_id INT)
RETURNS TEXT
AS $$
SELECT
  COALESCE(
    MAX(a.name),
    MAX(b.name),
    MAX(c.name
  )) AS name
FROM games
LEFT JOIN game_names a ON a.game = games.id AND a.language = lang_id
LEFT JOIN game_names b ON b.game = games.id AND b.language = 1
LEFT JOIN game_names c ON c.game = games.id
WHERE games.id = game_id
GROUP BY games.id;
$$ LANGUAGE SQL;
