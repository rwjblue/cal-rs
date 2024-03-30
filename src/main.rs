use clap::{Parser, ValueEnum};
use std::fmt;

use chrono::prelude::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Sets the first day of the week. If not set, defaults to the system preference.
    #[arg(short, long, value_enum)]
    first_day_of_week: Option<FirstDayOfWeek>,

    #[arg(short, long)]
    year: Option<i32>,

    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..=12))]
    month: Option<u32>,
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
fn determine_start_date(year: Option<i32>, month: Option<u32>) -> chrono::NaiveDate {
    match (year, month) {
        (Some(year), Some(month)) => NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap_or_else(|| panic!("Invalid year and month combination: {}-{:02}", year, month)),
        _ => Local::now().date_naive().with_day(1).unwrap(),
    }
}

#[derive(Debug)]
struct Month {
    start_date: NaiveDate,
    weeks: Vec<Week>,
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let start_date = self.start_date;

        writeln!(
            f,
            "{:^20}",
            format!("{} {}", start_date.format("%B"), start_date.year())
        )?;
        writeln!(f, "Mo Tu We Th Fr Sa Su")?;

        for week in &self.weeks {
            write!(
                f,
                "{} {} {} {} {} {} {}",
                format_date(week.monday),
                format_date(week.tuesday),
                format_date(week.wednesday),
                format_date(week.thursday),
                format_date(week.friday),
                format_date(week.saturday),
                format_date(week.sunday)
            )?;
            writeln!(f)?;
        }

        Ok(())
    }
}

fn format_date(date: Option<NaiveDate>) -> String {
    match date {
        Some(d) => format!("{:2}", d.day()),
        None => "  ".to_string(),
    }
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

    fn is_empty(&self) -> bool {
        self.monday.is_none()
            && self.tuesday.is_none()
            && self.wednesday.is_none()
            && self.thursday.is_none()
            && self.friday.is_none()
            && self.saturday.is_none()
            && self.sunday.is_none()
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

    if !current_week.is_empty() {
        weeks.push(current_week);
    }

    Month { start_date, weeks }
}

fn main() {
    let args = Arguments::parse();

    let first_day_of_week = determine_default_first_day_of_week(args.first_day_of_week);
    let start_date = determine_start_date(args.year, args.month);

    let month = build_month(start_date, first_day_of_week);

    println!("{}", month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_month_simple() {
        let start_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let first_day_of_week = Weekday::Mon;
        let month = build_month(start_date, first_day_of_week);

        insta::assert_snapshot!(month, @r###"
             March 2024     
        Mo Tu We Th Fr Sa Su
                     1  2  3
         4  5  6  7  8  9 10
        11 12 13 14 15 16 17
        18 19 20 21 22 23 24
        25 26 27 28 29 30 31
        "###);
    }

    #[test]
    fn test_build_month_leap_february() {
        let start_date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let first_day_of_week = Weekday::Mon;
        let month = build_month(start_date, first_day_of_week);

        insta::assert_snapshot!(month, @r###"
           February 2024    
        Mo Tu We Th Fr Sa Su
                  1  2  3  4
         5  6  7  8  9 10 11
        12 13 14 15 16 17 18
        19 20 21 22 23 24 25
        26 27 28 29         
        "###);
    }
}
