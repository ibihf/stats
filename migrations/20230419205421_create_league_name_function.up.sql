-- Add up migration script here
CREATE FUNCTION league_name(league_id INT, lang_id INT)
RETURNS TEXT
AS $$
SELECT
  COALESCE(
    MAX(a.name),
    MAX(b.name),
    MAX(c.name
  )) AS name
FROM leagues
LEFT JOIN league_names a ON a.league = leagues.id AND a.language = lang_id
LEFT JOIN league_names b ON b.league = leagues.id AND b.language = 1
LEFT JOIN league_names c ON c.league = leagues.id
WHERE leagues.id = league_id
GROUP BY leagues.id;
$$ LANGUAGE SQL;
