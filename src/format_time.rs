use icu::datetime::input::DateTime;
use icu::datetime::options::TimePrecision;
use icu::datetime::{DateTimeFormatter, DateTimeFormatterPreferences, fieldsets};
use icu::locale::preferences::extensions::unicode::keywords::HourCycle;
use jiff_icu::ConvertFrom;
use std::fmt::{self, Display};
use std::time::{Duration, SystemTime};

use crate::fl;
use crate::localize::LOCALE;

pub fn date_time_formatter(military_time: bool) -> DateTimeFormatter<fieldsets::YMDT> {
    let mut prefs = DateTimeFormatterPreferences::from(LOCALE.clone());
    prefs.hour_cycle = Some(if military_time {
        HourCycle::H23
    } else {
        HourCycle::H12
    });

    let mut fs = fieldsets::YMDT::medium();
    fs = fs.with_time_precision(TimePrecision::Minute);

    DateTimeFormatter::try_new(prefs, fs).expect("failed to create DateTimeFormatter")
}

pub fn time_formatter(military_time: bool) -> DateTimeFormatter<fieldsets::T> {
    let mut prefs = DateTimeFormatterPreferences::from(LOCALE.clone());
    prefs.hour_cycle = Some(if military_time {
        HourCycle::H23
    } else {
        HourCycle::H12
    });

    let mut fs = fieldsets::T::medium();
    fs = fs.with_time_precision(TimePrecision::Minute);

    DateTimeFormatter::try_new(prefs, fs).expect("failed to create DateTimeFormatter")
}

pub struct FormatTime<'a> {
    pub time: SystemTime,
    pub date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
    pub time_formatter: &'a DateTimeFormatter<fieldsets::T>,
}

impl<'a> FormatTime<'a> {
    pub fn from_secs(
        secs: i64,
        date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
        time_formatter: &'a DateTimeFormatter<fieldsets::T>,
    ) -> Option<Self> {
        // This looks convoluted because we need to ensure the units match up
        let secs: u64 = secs.try_into().ok()?;
        let now = SystemTime::now();
        let filetime_diff = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|from_epoch| from_epoch.as_secs())
            .ok()
            .and_then(|now_secs| now_secs.checked_sub(secs))
            .map(Duration::from_secs)?;
        now.checked_sub(filetime_diff).map(|time| Self {
            time,
            date_time_formatter,
            time_formatter,
        })
    }
}

impl Display for FormatTime<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Ok(zoned) = jiff::Zoned::try_from(self.time) else {
            return Ok(());
        };
        let now = jiff::Zoned::now();
        let icu_datetime = DateTime::convert_from(zoned.datetime());
        if zoned.date() == now.date() {
            f.write_str(fl!("today").as_str())?;
            f.write_str(", ")?;
            self.time_formatter.format(&icu_datetime).fmt(f)
        } else {
            self.date_time_formatter.format(&icu_datetime).fmt(f)
        }
    }
}

pub const fn format_time<'a>(
    time: SystemTime,
    date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
    time_formatter: &'a DateTimeFormatter<fieldsets::T>,
) -> FormatTime<'a> {
    FormatTime {
        time,
        date_time_formatter,
        time_formatter,
    }
}
