ALTER TABLE registration_codes RENAME TO confirmation_codes;
ALTER TABLE confirmation_codes ADD COLUMN _type TEXT;