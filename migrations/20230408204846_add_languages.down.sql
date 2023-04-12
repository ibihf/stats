-- Add down migration script here
DELETE FROM supported_languages
WHERE id=1 OR id=2;
