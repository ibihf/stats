-- Add up migration script here
CREATE OR REPLACE VIEW team_points_view
AS SELECT
  teams.id AS team_id,
  games.id AS game_id,
  games.division AS division_id,
  divisions.league AS league_id,
  reg_win(games.id, teams.id) AS reg_wins,
  reg_loss(games.id, teams.id) AS reg_losses,
  ot_win(games.id, teams.id) AS ot_wins,
  ot_loss(games.id, teams.id) AS ot_losses,
  tie(games.id, teams.id) AS ties,
  iihf_points(games.id, teams.id) AS points
FROM games
JOIN divisions
  ON divisions.id=games.division
JOIN periods
  ON periods.game=games.id
JOIN shots
  ON shots.period=periods.id
JOIN game_players
  ON game_players.id=shots.shooter
JOIN teams scoring_team
  ON scoring_team.id=game_players.team
JOIN teams
  ON teams.id=games.team_home
  OR teams.id=games.team_away
GROUP BY team_id,game_id,division_id,league_id;
