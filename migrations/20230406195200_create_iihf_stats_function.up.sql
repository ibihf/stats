-- Add up migration script here
CREATE OR REPLACE FUNCTION iihf_stats(game_id INT, team_id INT)
RETURNS TABLE (
  reg_win INT,
  reg_loss INT,
  ot_win INT,
  ot_loss INT,
  tie INT,
	game INT,
	team INT
) AS $$
DECLARE
	opponent_team_id INTEGER;
BEGIN
	IF NOT EXISTS (SELECT * FROM games WHERE games.id=game_id) THEN
		RAISE EXCEPTION 'The game does not exist.';
	END IF;
	IF NOT EXISTS (SELECT * FROM teams WHERE teams.id=team_id) THEN
		RAISE EXCEPTION 'The team does not exist.';
	END IF;
	IF NOT EXISTS (SELECT * FROM games JOIN teams ON teams.id=games.team_home OR teams.id=team_away WHERE games.id=game_id) THEN
		RAISE EXCEPTION 'The team specified did not play this game.';
	END IF;

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

	RETURN QUERY
	SELECT
		(CASE WHEN goals(game_id, team_id) > goals(game_id, opponent_team_id) AND periods(game_id) <= 3 THEN 1 ELSE 0 END) AS reg_win,
		(CASE WHEN goals(game_id, team_id) < goals(game_id, opponent_team_id) AND periods(game_id) <= 3 THEN 1 ELSE 0 END) AS reg_loss,
		(CASE WHEN goals(game_id, team_id) > goals(game_id, opponent_team_id) AND periods(game_id) > 3 THEN 1 ELSE 0 END) AS ot_win,
		(CASE WHEN goals(game_id, team_id) < goals(game_id, opponent_team_id) AND periods(game_id) > 3 THEN 1 ELSE 0 END) AS ot_loss,
		(CASE WHEN goals(game_id, team_id) = goals(game_id, opponent_team_id) THEN 1 ELSE 0 END) AS tie,
		game_id AS game,
		team_id AS team;
END;
$$ LANGUAGE plpgsql;
