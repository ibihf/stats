-- Add down migration script here
DELETE FROM users
	WHERE id=1;
