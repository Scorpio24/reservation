-- Add down migration script here
DROP TABLE rsvp.reservations;
DROP TYPE rsvp.reservation_status;
DROP TYPE rsvp.reservation_update_type;