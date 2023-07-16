-- Add up migration script here
-- admin/admin login
INSERT INTO users
	(id, user_name, pass_hash)
VALUES
	(1, 'admin', '$2y$10$eoghV7BDZSDDKAjjYuESUuMwl3IqFBzawFybgkBKWl.CKlI6jXAQ.');
