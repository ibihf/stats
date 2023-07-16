-- Add up migration script here
CREATE TABLE users_roles (
	id SERIAL PRIMARY KEY NOT NULL,
	user_id INTEGER NOT NULL,
	role INTEGER NOT NULL,
	-- user must exist
	CONSTRAINT user_id_fk
		FOREIGN KEY(user_id)
			REFERENCES users(id)
			ON DELETE RESTRICT,
	-- role must exist
	CONSTRAINT role_fk
		FOREIGN KEY(role)
			REFERENCES roles(id)
			ON DELETE RESTRICT
);
