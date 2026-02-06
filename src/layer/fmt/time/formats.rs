use std::num::NonZeroU8;

use time::{
    format_description::{
        BorrowedFormatItem,
        well_known::{
            Iso8601,
            iso8601::{Config, TimePrecision},
        },
    },
    macros::format_description,
};

macro mk_fmts($(($format_name:ident, $format:tt),)*) {
    $(pub const $format_name: &'static [BorrowedFormatItem<'static>] = format_description!($format);)*
}

mk_fmts!(
    (
        YYYY_MM_DD_HH_MM_SS,
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ),
    (
        YYYY_MM_DD_HH_MM_SS_OFFSET,
        "[year]-[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
    ),
    (MM_DD_HH_MM_SS, "[month]-[day] [hour]:[minute]:[second]"),
    (
        MM_DD_HH_MM_SS_OFFSET,
        "[month]-[day] [hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
    ),
    (HH_MM_SS, "[hour]:[minute]:[second]"),
    (
        HH_MM_SS_OFFSET,
        "[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]"
    ),
);

pub const ISO8601: Iso8601<
    {
        Config::DEFAULT
            .set_time_precision(TimePrecision::Second {
                decimal_digits: Some(NonZeroU8::new(6).unwrap()),
            })
            .encode()
    },
> = Iso8601;

pub const ISO8601_NO_SUBSECONDS: Iso8601<
    {
        Config::DEFAULT
            .set_time_precision(TimePrecision::Second {
                decimal_digits: None,
            })
            .encode()
    },
> = Iso8601;
