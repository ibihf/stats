CREATE FUNCTION reg_win(game_id INT, team_id INT)
RETURNS INTEGER 
AS $$
DECLARE
	opponent_team_id INTEGER;
BEGIN
	SELECT
    teams.id
  INTO
    opponent_team_id
  FROM games
  JOIN teams
    ON (teams.id=games.team_home
    OR teams.id=games.team_away)
  WHERE games.id=game_id
    AND teams.id!=team_id;
	RETURN (SELECT (CASE WHEN goals(game_id, team_id) > goals(game_id, opponent_team_id) AND periods(game_id) <= 3 THEN 1 ELSE 0 END));
END;
$$ LANGUAGE plpgsql;
