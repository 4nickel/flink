use chrono::{NaiveDateTime, DateTime, Utc};

pub struct UtcDateTime(pub DateTime<Utc>);

impl From<NaiveDateTime> for UtcDateTime {
    fn from(naive: NaiveDateTime) -> Self
    {
        UtcDateTime(DateTime::<Utc>::from_utc(naive, Utc))
    }
}

impl From<UtcDateTime> for NaiveDateTime {
    fn from(utc: UtcDateTime) -> Self
    {
        NaiveDateTime::new(utc.0.date().naive_utc(), utc.0.time())
    }
}
