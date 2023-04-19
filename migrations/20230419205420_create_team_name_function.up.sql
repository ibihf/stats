-- Add up migration script here
CREATE FUNCTION team_name(team_id INT, lang_id INT)
RETURNS TEXT
AS $$
SELECT
  COALESCE(
    MAX(a.name),
    MAX(b.name),
    MAX(c.name
  )) AS name
FROM teams
LEFT JOIN team_names a ON a.team = teams.id AND a.language = lang_id
LEFT JOIN team_names b ON b.team = teams.id AND b.language = 1
LEFT JOIN team_names c ON c.team = teams.id
WHERE teams.id = team_id
GROUP BY teams.id;
$$ LANGUAGE SQL;
