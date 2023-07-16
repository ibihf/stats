-- Add down migration script here
DELETE FROM roles
	WHERE id BETWEEN 1 AND 3;
