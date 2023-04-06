-- Add up migration script here
CREATE OR REPLACE FUNCTION iihf_points(game_id INT, team_id INT)
RETURNS INTEGER AS $$
BEGIN
  RETURN (
    SELECT
      (iihs_stats.reg_win * 3) +
      (iihs_stats.reg_loss * 0) +
      (iihs_stats.ot_win * 2) +
      (iihs_stats.ot_loss * 1) +
      (iihs_stats.tie * 2) AS points
    FROM calculate_iihs_stats_stats(game_id, team_id) iihs_stats);
END;
$$ LANGUAGE plpgsql;
