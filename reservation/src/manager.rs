use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, Row};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp};

impl Rsvp for ReservationManager {
    async fn reserve(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        if rsvp.start.is_none() || rsvp.end.is_none() {
            return Err(ReservationError::InvalidTime);
        }

        let start: DateTime<Utc> =
            DateTime::from_timestamp(rsvp.start.as_ref().unwrap().seconds.clone(), 0).unwrap();
        let end: DateTime<Utc> = DateTime::from_timestamp(rsvp.end.as_ref().unwrap().seconds.clone(), 0).unwrap();
        if start >= end {
            return Err(ReservationError::InvalidTime);
        }

        let timespan: PgRange<DateTime<Utc>> = (start..end).into();
        // generate a insert sql for the reservation

        // excute the sql
        let id = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) 
        VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(rsvp.status)
        .fetch_one(&self.pool)
        .await?
        .get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    async fn change_status(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), ReservationError> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        todo!()
    }
}
