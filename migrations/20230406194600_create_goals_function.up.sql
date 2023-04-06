-- Add up migration script here
CREATE FUNCTION goals(game_id INTEGER, team_id INTEGER)
RETURNS INTEGER AS $$
DECLARE
	goals INTEGER;
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
    COUNT(shots.id)
	INTO
		goals
	FROM shots
	JOIN game_players
		ON game_players.id=shots.shooter
 	JOIN periods
	  ON periods.id=shots.period
 WHERE shots.goal=true
 	 AND game_players.team=team_id
	 AND periods.game=game_id;
 -- return 0 if not goals are found given the team and the game
 RETURN COALESCE(goals, 0);
END;
$$ LANGUAGE plpgsql;
