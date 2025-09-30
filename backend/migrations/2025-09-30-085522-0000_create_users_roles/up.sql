CREATE TABLE users_roles
(
    user_id UUID REFERENCES users (id) ON DELETE CASCADE,
    role_id INT REFERENCES roles (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- default owner
INSERT INTO users_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u
         JOIN roles r ON r.name = 'owner'
WHERE u.username = 'owner' ON CONFLICT DO NOTHING;