use clap::{Parser, ValueEnum};

use chrono::Weekday;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Sets the first day of the week. If not set, defaults to the system preference.
    #[arg(short, long, value_enum)]
    first_day_of_week: Option<FirstDayOfWeek>,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum FirstDayOfWeek {
    Sunday,
    Monday,
}

impl From<FirstDayOfWeek> for chrono::Weekday {
    fn from(day: FirstDayOfWeek) -> Self {
        match day {
            FirstDayOfWeek::Sunday => chrono::Weekday::Sun,
            FirstDayOfWeek::Monday => chrono::Weekday::Mon,
        }
    }
}

fn determine_default_first_day_of_week(
    specified_first_day_of_week: Option<FirstDayOfWeek>,
) -> chrono::Weekday {
    if let Some(first_day_of_week) = specified_first_day_of_week {
        first_day_of_week.into()
    } else {
        // TODO: figure out how to default to system preference
        // https://www.perplexity.ai/search/How-can-I-zngZ7lVUQMWV13U92fmJXQ
        Weekday::Mon
    }
}

fn main() {
    let args = Arguments::parse();

    let first_day_of_week = determine_default_first_day_of_week(args.first_day_of_week);
}

#[cfg(test)]
mod tests {
    use super::*;
}
