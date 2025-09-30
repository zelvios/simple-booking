CREATE TABLE roles_permissions
(
    role_id       INT REFERENCES roles (id) ON DELETE CASCADE,
    permission_id INT REFERENCES permissions (id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

INSERT INTO roles_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r,
     permissions p
WHERE r.name = 'owner' ON CONFLICT DO NOTHING;

INSERT INTO roles_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r,
     permissions p
WHERE r.name = 'mod' ON CONFLICT DO NOTHING;

INSERT INTO roles_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
         JOIN permissions p ON p.name IN ('bookings:create', 'bookings:edit, bookings:delete, users:lock')
WHERE r.name = 'staff' ON CONFLICT DO NOTHING;