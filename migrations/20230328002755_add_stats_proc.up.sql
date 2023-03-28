-- Add up migration script here
CREATE OR REPLACE FUNCTION player_stats_overview_all() RETURNS VOID
LANGUAGE SQL
AS $$
  SELECT
    (
      SELECT COUNT(id)
      FROM shots
      WHERE shooter=players.id
        AND goal=true
    ) AS goals,
    (
      SELECT COUNT(id)
      FROM shots
      WHERE assistant=players.id
        AND goal=true
    ) AS assists,
    (
      SELECT COUNT(id)
      FROM shots
      WHERE assistant=players.id
         OR shooter=players.id
    ) AS points,
    players.name AS player_name
  FROM players
  ORDER BY
    points DESC,
    players.name;
$$;
