-- Add up migration script here
CREATE OR REPLACE VIEW player_points_view
AS SELECT
  COUNT(shots.id) AS points,
  COUNT(CASE WHEN shots.shooter = game_players.id THEN shots.id END) AS goals,
  COUNT(CASE WHEN shots.assistant = game_players.id OR shots.assistant_second = game_players.id THEN shots.id END) AS assists,
  periods.id AS period_id,
  games.id AS game_id,
  games.division AS division_id,
  players.first_names,
  players.last_name,
  players.id
FROM players JOIN game_players 
  ON game_players.player = players.id
LEFT JOIN shots
  ON shots.goal=true
 AND (shots.shooter=game_players.id
  OR shots.assistant=game_players.id
  OR shots.assistant_second=game_players.id)
LEFT JOIN periods
  ON periods.id=shots.period
LEFT JOIN games
  ON games.id=periods.game
GROUP BY
  players.id,division_id,game_id,period_id;
