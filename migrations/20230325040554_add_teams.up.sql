-- Add up migration script here
INSERT INTO teams
  (id, division)
VALUES
  (1, 1),
  (2, 1);

INSERT INTO team_names
  (id, team, name, language)
VALUES
  (1, 1, 'Bullseye', 1),
  (2, 2, 'See Cats', 1),
  (3, 1, 'bulle', 2),
  (4, 2, 'Chats Voient', 2);
