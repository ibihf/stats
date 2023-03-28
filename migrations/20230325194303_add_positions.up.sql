-- Add up migration script here
INSERT INTO positions
  (id, name, short_name)
VALUES
  (1, 'Center', 'C'),
  (2, 'Right-Wing', 'R'),
  (3, 'Left-Wing', 'L'),
  (4, 'Defence', 'D'),
  (5, 'Goalie', 'G'),
  (6, 'Head Coach', 'HC'),
  (7, 'Assistant Coach', 'AC');
