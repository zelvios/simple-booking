CREATE TABLE permissions
(
    id   SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL
);

INSERT INTO permissions (name)
SELECT p
FROM (VALUES ('bookings:create'),
             ('bookings:edit'),
             ('bookings:delete'),
             ('bookings:hard_delete'),
             ('users:delete'),
             ('users:hard_delete'),
             ('users:edit'),
             ('users:lock'),
             ('users:assign_role')) AS v(p) ON CONFLICT (name) DO NOTHING;