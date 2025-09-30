DO
$$
BEGIN
  IF
NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'booking_status') THEN
CREATE TYPE booking_status AS ENUM ('pending', 'confirmed', 'cancelled', 'completed', 'no_show', 'delayed');
END IF;
END
$$;

CREATE TABLE bookings
(
    id           UUID PRIMARY KEY        DEFAULT gen_random_uuid(),
    user_id      UUID           NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    title        VARCHAR(255)   NOT NULL,
    description  TEXT,
    booking_date TIMESTAMPTZ    NOT NULL,
    status       booking_status NOT NULL DEFAULT 'pending',
    created_at   TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ    NOT NULL DEFAULT now(),
    -- Soft Delete
    deleted_at   TIMESTAMPTZ             DEFAULT NULL
);
