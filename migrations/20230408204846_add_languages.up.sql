-- Add up migration script here
INSERT INTO supported_languages
  (id, short_name, native_name)
VALUES
  (1, 'en-ca', 'English (Canada)'),
  (2, 'fr-ca', 'Français (Canada)');
