-- Add up migration script here
CREATE FUNCTION role_name(role_id INT, lang_id INT)
RETURNS TEXT
AS $$
SELECT
  COALESCE(
    MAX(a.name),
    MAX(b.name),
    MAX(c.name
  )) AS name
FROM roles
LEFT JOIN role_names a ON a.role = roles.id AND a.language = lang_id
LEFT JOIN role_names b ON b.role = roles.id AND b.language = 1
LEFT JOIN role_names c ON c.role = roles.id
WHERE roles.id = role_id
GROUP BY roles.id;
$$ LANGUAGE SQL;
