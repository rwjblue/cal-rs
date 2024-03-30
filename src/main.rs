use clap::{Parser, ValueEnum};

use chrono::prelude::*;

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
    first_day_of_week: Option<FirstDayOfWeek>,
) -> chrono::Weekday {
    if let Some(first_day_of_week) = first_day_of_week {
        first_day_of_week.into()
    } else {
        // TODO: figure out how to default to system preference
        // https://www.perplexity.ai/search/How-can-I-zngZ7lVUQMWV13U92fmJXQ
        Weekday::Mon
    }
}

// TODO: Update Arguments to allow forms of specifying which month/year to print
fn determine_start_date() -> chrono::NaiveDate {
    let today = chrono::Local::now().date_naive();

    NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .expect("Couldn't determine the first day of the month!")
}

#[derive(Debug)]
struct Month {
    weeks: Vec<Week>,
}

#[derive(Debug)]
struct Week {
    monday: Option<NaiveDate>,
    tuesday: Option<NaiveDate>,
    wednesday: Option<NaiveDate>,
    thursday: Option<NaiveDate>,
    friday: Option<NaiveDate>,
    saturday: Option<NaiveDate>,
    sunday: Option<NaiveDate>,
}

impl Week {
    fn new() -> Week {
        Week {
            monday: None,
            tuesday: None,
            wednesday: None,
            thursday: None,
            friday: None,
            saturday: None,
            sunday: None,
        }
    }
}

fn get_days_in_month(start_date: NaiveDate) -> Vec<NaiveDate> {
    let end_date = if start_date.month() == 12 {
        NaiveDate::from_ymd_opt(start_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(start_date.year(), start_date.month() + 1, 1)
    };

    let end_date = end_date.expect("Couldn't determine a valid end date");

    let num_days = end_date.signed_duration_since(start_date).num_days();

    (0..num_days)
        .map(|days| start_date + chrono::Duration::days(days))
        .collect()
}

fn build_month(start_date: NaiveDate, first_day_of_week: Weekday) -> Month {
    let days = get_days_in_month(start_date);
    let mut weeks: Vec<Week> = vec![];

    let mut current_week = Week::new();

    for day in days {
        let weekday = day.weekday();
        match weekday {
            Weekday::Mon => current_week.monday = Some(day),
            Weekday::Tue => current_week.tuesday = Some(day),
            Weekday::Wed => current_week.wednesday = Some(day),
            Weekday::Thu => current_week.thursday = Some(day),
            Weekday::Fri => current_week.friday = Some(day),
            Weekday::Sat => current_week.saturday = Some(day),
            Weekday::Sun => current_week.sunday = Some(day),
        }

        let last_day_of_week = matches!(
            (first_day_of_week, weekday),
            (Weekday::Sun, Weekday::Sat) | (Weekday::Mon, Weekday::Sun)
        );

        if last_day_of_week {
            weeks.push(current_week);
            current_week = Week::new();
        }
    }

    Month { weeks }
}
fn main() {
    let args = Arguments::parse();

    let first_day_of_week = determine_default_first_day_of_week(args.first_day_of_week);
    let start_date = determine_start_date();

    let month = build_month(start_date, first_day_of_week);

    println!("{:?}", month)
}

#[cfg(test)]
mod tests {
    use super::*;
}
