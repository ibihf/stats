-- Add up migration script here
CREATE OR REPLACE FUNCTION iihf_points(game_id INT, team_id INT)
RETURNS INTEGER AS $$
BEGIN
  RETURN (
    SELECT
      (iihf_stats.reg_win * 3) +
      (iihf_stats.reg_loss * 0) +
      (iihf_stats.ot_win * 2) +
      (iihf_stats.ot_loss * 1) +
      (iihf_stats.tie * 2) AS points
    FROM iihf_stats(game_id, team_id) iihf_stats);
END;
$$ LANGUAGE plpgsql;
