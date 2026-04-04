ALTER TABLE user_permissions
DROP CONSTRAINT user_permissions_permission_name_fkey;

ALTER TABLE user_permissions
ADD CONSTRAINT user_permissions_permission_name_fkey
FOREIGN KEY (user_id)
REFERENCES users(id)
ON DELETE CASCADE;