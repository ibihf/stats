-- Add down migration script here
DELETE FROM role_names
	WHERE id BETWEEN 1 AND 4;
