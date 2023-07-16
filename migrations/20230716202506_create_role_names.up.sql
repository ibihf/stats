-- Add up migration script here
CREATE TABLE IF NOT EXISTS role_names (
  id SERIAL PRIMARY KEY NOT NULL,
  language INTEGER NOT NULL,
  name VARCHAR(32) NOT NULL,
  role INTEGER NOT NULL,
  CONSTRAINT language_fk
    FOREIGN KEY(language)
      REFERENCES supported_languages(id)
      ON DELETE RESTRICT,
  CONSTRAINT role_fk
    FOREIGN KEY(role)
      REFERENCES roles(id)
      ON DELETE RESTRICT,
  CONSTRAINT no_duplicated_role_names
    UNIQUE (role, language)
);
