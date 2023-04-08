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

	RETURN QUERY
	SELECT
		reg_win(game_id, team_id) AS reg_win,
		reg_loss(game_id, team_id) AS reg_loss,
		ot_win(game_id, team_id) AS ot_win,
		ot_loss(game_id, team_id) AS ot_loss,
		tie(game_id, team_id) AS tie,
		game_id AS game,
		team_id AS team;
END;
$$ LANGUAGE plpgsql;
