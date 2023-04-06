CREATE FUNCTION periods(game_id INTEGER)
RETURNS INTEGER AS $$
BEGIN
  RETURN (SELECT COUNT(id) FROM periods WHERE periods.game=game_id);
END;
$$ LANGUAGE plpgsql;
