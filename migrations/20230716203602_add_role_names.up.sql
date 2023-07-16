-- Add up migration script here
INSERT INTO role_names
	(id, role, name, language)
VALUES
	(1, 1, 'admin', 1),
	(2, 2, 'reviewer', 1),
	(3, 3, 'user', 1),
	(4, 1, 'administrateur', 2);
