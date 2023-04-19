-- Add up migration script here
CREATE FUNCTION division_name(division_id INT, lang_id INT)
RETURNS TEXT
AS $$
SELECT
  COALESCE(
    MAX(a.name),
    MAX(b.name),
    MAX(c.name
  )) AS name
FROM divisions
LEFT JOIN division_names a ON a.division = divisions.id AND a.language = lang_id
LEFT JOIN division_names b ON b.division = divisions.id AND b.language = 1
LEFT JOIN division_names c ON c.division = divisions.id
WHERE divisions.id = division_id
GROUP BY divisions.id;
$$ LANGUAGE SQL;
