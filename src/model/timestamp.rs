//! Utilities for parsing and formatting RFC 3339 timestamps.
//!
//! The [`Timestamp`] newtype wraps `chrono::DateTime<Utc>` or `time::OffsetDateTime` if the `time`
//! feature is enabled.
//!
//! # Formatting
//! ```
//! # use serenity::model::id::GuildId;
//! # use serenity::model::Timestamp;
//! #
//! let timestamp: Timestamp = GuildId::new(175928847299117063).created_at();
//! assert_eq!(timestamp.unix_timestamp(), 1462015105);
//! assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.796Z");
//! ```
//!
//! # Parsing RFC 3339 string
//! ```
//! # use serenity::model::Timestamp;
//! #
//! let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
//! let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
//! let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
//!
//! let timestamp: Timestamp = "2016-04-30T11:18:25Z".parse().unwrap();
//! let timestamp: Timestamp = "2016-04-30T11:18:25+00:00".parse().unwrap();
//! let timestamp: Timestamp = "2016-04-30T11:18:25.796Z".parse().unwrap();
//!
//! assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
//! assert!(Timestamp::parse("2016-04-30T11:18").is_err());
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Discord's epoch starts at "2015-01-01T00:00:00+00:00"
const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[cfg(feature = "chrono")]
mod imp {
    pub(super) use chrono::ParseError as InnerError;
    use chrono::{DateTime, NaiveDateTime, SecondsFormat, TimeZone, Utc};

    use super::*;

    /// Representation of a Unix timestamp.
    ///
    /// The struct implements the `std::fmt::Display` trait to format the underlying type as
    /// an RFC 3339 date and string such as `2016-04-30T11:18:25.796Z`.
    ///
    /// ```
    /// # use serenity::model::id::GuildId;
    /// # use serenity::model::Timestamp;
    /// #
    /// let timestamp: Timestamp = GuildId::new(175928847299117063).created_at();
    /// assert_eq!(timestamp.unix_timestamp(), 1462015105);
    /// assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.796Z");
    /// ```
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct Timestamp(DateTime<Utc>);

    impl Timestamp {
        pub(crate) fn from_discord_id(id: u64) -> Timestamp {
            Self(Utc.timestamp_millis_opt(((id >> 22) + DISCORD_EPOCH) as i64).unwrap())
        }

        /// Create a new `Timestamp` with the current date and time in UTC.
        #[must_use]
        pub fn now() -> Self {
            Self(Utc::now())
        }

        /// Create a new `Timestamp` from a UNIX timestamp.
        ///
        /// # Errors
        ///
        /// Returns `Err` if the value is invalid.
        pub fn from_unix_timestamp(secs: i64) -> Result<Self, InvalidTimestamp> {
            let dt = NaiveDateTime::from_timestamp_opt(secs, 0).ok_or(InvalidTimestamp)?;
            Ok(Self(DateTime::from_utc(dt, Utc)))
        }

        /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
        #[must_use]
        pub fn unix_timestamp(&self) -> i64 {
            self.0.timestamp()
        }

        /// Parse a timestamp from an RFC 3339 date and time string.
        ///
        /// # Examples
        /// ```
        /// # use serenity::model::Timestamp;
        /// #
        /// let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
        /// let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
        /// let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
        ///
        /// assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
        /// assert!(Timestamp::parse("2016-04-30T11:18").is_err());
        /// ```
        ///
        /// # Errors
        ///
        /// Returns `Err` if the string is not a valid RFC 3339 date and time string.
        pub fn parse(input: &str) -> Result<Timestamp, ParseError> {
            DateTime::parse_from_rfc3339(input)
                .map(|d| Self(d.with_timezone(&Utc)))
                .map_err(ParseError)
        }
    }

    impl std::fmt::Display for Timestamp {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = self.0.to_rfc3339_opts(SecondsFormat::Millis, true);
            f.write_str(&s)
        }
    }

    impl std::ops::Deref for Timestamp {
        type Target = DateTime<Utc>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<Tz: TimeZone> From<DateTime<Tz>> for Timestamp {
        fn from(dt: DateTime<Tz>) -> Self {
            Self(dt.with_timezone(&Utc))
        }
    }
}
#[cfg(not(feature = "chrono"))]
mod imp {
    pub(super) use dep_time::error::Parse as InnerError;
    use dep_time::format_description::well_known::Rfc3339;
    use dep_time::serde::rfc3339;
    use dep_time::{Duration, OffsetDateTime};

    use super::*;

    /// Representation of a Unix timestamp.
    ///
    /// The struct implements the `std::fmt::Display` trait to format the underlying type as
    /// an RFC 3339 date and string such as `2016-04-30T11:18:25.796Z`.
    ///
    /// ```
    /// # use serenity::model::id::GuildId;
    /// # use serenity::model::Timestamp;
    /// #
    /// let timestamp: Timestamp = GuildId::new(175928847299117063).created_at();
    /// assert_eq!(timestamp.unix_timestamp(), 1462015105);
    /// assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.796Z");
    /// ```
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct Timestamp(#[serde(with = "rfc3339")] OffsetDateTime);

    impl Timestamp {
        pub(crate) fn from_discord_id(id: u64) -> Timestamp {
            let ns =
                Duration::milliseconds(((id >> 22) + DISCORD_EPOCH) as i64).whole_nanoseconds();
            // This can't fail because of the bit shifting
            // `(u64::MAX >> 22) + DISCORD_EPOCH` = 5818116911103 = "Wed May 15 2154 07:35:11 GMT+0000"
            Self(OffsetDateTime::from_unix_timestamp_nanos(ns).expect("can't fail"))
        }

        /// Create a new `Timestamp` with the current date and time in UTC.
        #[must_use]
        pub fn now() -> Self {
            Self(OffsetDateTime::now_utc())
        }

        /// Create a new `Timestamp` from a UNIX timestamp.
        ///
        /// # Errors
        ///
        /// Returns `Err` if the value is invalid. The valid range of the value may vary depending on
        /// the feature flags enabled (`time` with `large-dates`).
        pub fn from_unix_timestamp(secs: i64) -> Result<Self, InvalidTimestamp> {
            let dt = OffsetDateTime::from_unix_timestamp(secs).map_err(|_| InvalidTimestamp)?;
            Ok(Self(dt))
        }

        /// Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC
        #[must_use]
        pub fn unix_timestamp(&self) -> i64 {
            self.0.unix_timestamp()
        }

        /// Parse a timestamp from an RFC 3339 date and time string.
        ///
        /// # Examples
        /// ```
        /// # use serenity::model::Timestamp;
        /// #
        /// let timestamp = Timestamp::parse("2016-04-30T11:18:25Z").unwrap();
        /// let timestamp = Timestamp::parse("2016-04-30T11:18:25+00:00").unwrap();
        /// let timestamp = Timestamp::parse("2016-04-30T11:18:25.796Z").unwrap();
        ///
        /// assert!(Timestamp::parse("2016-04-30T11:18:25").is_err());
        /// assert!(Timestamp::parse("2016-04-30T11:18").is_err());
        /// ```
        ///
        /// # Errors
        ///
        /// Returns `Err` if the string is not a valid RFC 3339 date and time string.
        pub fn parse(input: &str) -> Result<Timestamp, ParseError> {
            OffsetDateTime::parse(input, &Rfc3339).map(Self).map_err(ParseError)
        }
    }

    impl std::fmt::Display for Timestamp {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = self.0.format(&Rfc3339).map_err(|_| std::fmt::Error)?;
            f.write_str(&s)
        }
    }

    impl std::ops::Deref for Timestamp {
        type Target = OffsetDateTime;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<OffsetDateTime> for Timestamp {
        fn from(dt: OffsetDateTime) -> Self {
            Self(dt)
        }
    }
}
pub use imp::*;

#[derive(Debug)]
pub struct InvalidTimestamp;

impl std::error::Error for InvalidTimestamp {}

impl fmt::Display for InvalidTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid UNIX timestamp value")
    }
}

/// Signifies the failure to parse the `Timestamp` from an RFC 3339 string.
#[derive(Debug)]
pub struct ParseError(InnerError);

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for Timestamp {
    type Err = ParseError;

    /// Parses an RFC 3339 date and time string such as `2016-04-30T11:18:25.796Z`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Timestamp::parse(s)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for Timestamp {
    type Error = ParseError;

    /// Parses an RFC 3339 date and time string such as `2016-04-30T11:18:25.796Z`.
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Timestamp::parse(s)
    }
}

impl From<&Timestamp> for Timestamp {
    fn from(ts: &Timestamp) -> Self {
        *ts
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn from_unix_timestamp() {
        let timestamp = Timestamp::from_unix_timestamp(1462015105).unwrap();
        assert_eq!(timestamp.unix_timestamp(), 1462015105);
        if cfg!(feature = "chrono") {
            assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25.000Z");
        } else {
            assert_eq!(timestamp.to_string(), "2016-04-30T11:18:25Z");
        }
    }
}
