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

    /// Display the number of months after the current month.
    #[arg(short = 'A', value_parser = clap::value_parser!(u32).range(1..=12))]
    months_after: Option<u32>,

    /// Display the number of months before the current month.
    #[arg(short = 'B', value_parser = clap::value_parser!(u32).range(1..=12))]
    months_before: Option<u32>,
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

#[cfg(target_os = "macos")]
fn get_system_default_first_workday() -> Option<Weekday> {
    use plist::Value;

    let plist_path = match home::home_dir() {
        Some(mut path) => {
            path.push("Library/Preferences/.GlobalPreferences.plist");
            path
        }
        None => return None,
    };

    let plist = Value::from_file(plist_path).ok()?;

    if let Some(dict) = plist.as_dictionary() {
        if let Some(Value::Dictionary(calendars)) = dict.get("AppleFirstWeekday") {
            if let Some(Value::Integer(first_weekday)) = calendars.get("gregorian") {
                return match first_weekday.as_signed()? {
                    1 => Some(Weekday::Sun),
                    2 => Some(Weekday::Mon),
                    _ => None,
                };
            }
        }
    } else {
        // could not process the plist file as a dictionary, we shouldn't consider this a
        // "succesful" read (which would default back to Sunday).
        return None;
    }

    // On macOS the default is Sunday if not set via system preferences. When it is in its default
    // value, the plist file will not contain the key `AppleFirstWeekday`.
    Some(Weekday::Sun)
}

#[cfg(not(target_os = "macos"))]
fn get_system_default_first_workday() -> Option<Weekday> {
    None
}

fn determine_default_first_day_of_week(
    first_day_of_week: Option<FirstDayOfWeek>,
) -> chrono::Weekday {
    if let Some(first_day_of_week) = first_day_of_week {
        first_day_of_week.into()
    } else {
        if let Some(weekday) = get_system_default_first_workday() {
            return weekday;
        }

        Weekday::Mon
    }
}

// TODO: Update Arguments to allow forms of specifying which month/year to print
fn determine_start_date(
    year: Option<i32>,
    month: Option<u32>,
    months_before: Option<u32>,
) -> chrono::NaiveDate {
    let start_date = match (year, month) {
        (Some(year), Some(month)) => NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap_or_else(|| panic!("Invalid year and month combination: {}-{:02}", year, month)),
        _ => Local::now().date_naive().with_day(1).unwrap(),
    };

    if let Some(months) = months_before {
        let start_date = if start_date.month() <= months {
            NaiveDate::from_ymd_opt(start_date.year() - 1, 12 - months + start_date.month(), 1)
        } else {
            NaiveDate::from_ymd_opt(start_date.year(), start_date.month() - months, 1)
        }
        .expect("Couldn't determine a valid start date");
        return start_date;
    }

    start_date
}

fn determine_number_of_months(months_after: Option<u32>, months_before: Option<u32>) -> u32 {
    let months_after = months_after.unwrap_or(0);
    let months_before = months_before.unwrap_or(0);

    1 + months_before + months_after
}

#[derive(Debug)]
struct MonthRange {
    months: Vec<Month>,
}

impl MonthRange {
    fn print(&self) -> String {
        let mut output = String::new();

        for chunk in self.months.chunks(3) {
            // print the month headers
            for (index, month) in chunk.iter().enumerate() {
                if index > 0 {
                    output.push_str("  ");
                }

                month.print_header(&mut output);
            }
            output.push('\n');

            // print the weekday headers
            for (index, month) in chunk.iter().enumerate() {
                if index > 0 {
                    output.push_str("  ");
                }

                month.print_weekday_header(&mut output);
            }
            output.push('\n');

            let max_weeks = self
                .months
                .iter()
                .map(|month| month.weeks.len())
                .max()
                .unwrap_or(0);

            for week_index in 0..max_weeks {
                for (index, month) in chunk.iter().enumerate() {
                    if index > 0 {
                        output.push_str("  ");
                    }

                    let week = month.weeks.get(week_index);
                    match week {
                        Some(week) => week.print(month.first_day_of_week, &mut output),
                        None => {
                            output.push_str("                    ");
                        }
                    }
                }
                output.push('\n');
            }
        }

        output
    }
}

#[derive(Debug)]
struct Month {
    start_date: NaiveDate,
    first_day_of_week: Weekday,
    weeks: Vec<Week>,
}

impl Month {
    fn print_header(&self, output: &mut String) {
        output.push_str(&format!(
            "{:^20}",
            format!(
                "{} {}",
                self.start_date.format("%B"),
                self.start_date.year()
            )
        ));
    }

    fn print_weekday_header(&self, output: &mut String) {
        match &self.first_day_of_week {
            Weekday::Mon => {
                output.push_str("Mo Tu We Th Fr Sa Su");
            }
            Weekday::Sun => {
                output.push_str("Su Mo Tu We Th Fr Sa");
            }

            _ => {
                panic!(
                    "Invalid first day of week specified: {}",
                    &self.first_day_of_week
                );
            }
        };
    }

    fn print(&self) -> String {
        let mut output = String::new();

        self.print_header(&mut output);
        output.push('\n');
        self.print_weekday_header(&mut output);
        output.push('\n');

        for week in &self.weeks {
            week.print(self.first_day_of_week, &mut output);
            output.push('\n');
        }

        output
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.print())
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

    fn print(&self, first_day_of_week: Weekday, output: &mut String) {
        match first_day_of_week {
            Weekday::Mon => {
                output.push_str(&format!(
                    "{} {} {} {} {} {} {}",
                    format_date(self.monday),
                    format_date(self.tuesday),
                    format_date(self.wednesday),
                    format_date(self.thursday),
                    format_date(self.friday),
                    format_date(self.saturday),
                    format_date(self.sunday)
                ));
            }
            Weekday::Sun => {
                output.push_str(&format!(
                    "{} {} {} {} {} {} {}",
                    format_date(self.sunday),
                    format_date(self.monday),
                    format_date(self.tuesday),
                    format_date(self.wednesday),
                    format_date(self.thursday),
                    format_date(self.friday),
                    format_date(self.saturday),
                ));
            }

            _ => {
                panic!("Invalid first day of week specified: {}", first_day_of_week);
            }
        };
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

    Month {
        start_date,
        first_day_of_week,
        weeks,
    }
}

fn build_month_range(
    start_date: NaiveDate,
    first_day_of_week: Weekday,
    num_months: u32,
) -> MonthRange {
    let mut months = vec![];
    let mut current_date = start_date;
    for _ in 0..num_months {
        let month = build_month(current_date, first_day_of_week);
        months.push(month);
        current_date = if current_date.month() == 12 {
            NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1)
        }
        .expect("Couldn't determine a valid end date");
    }

    MonthRange { months }
}

fn main() {
    let args = Arguments::parse();

    let first_day_of_week = determine_default_first_day_of_week(args.first_day_of_week);
    let start_date = determine_start_date(args.year, args.month, args.months_before);
    let num_months = determine_number_of_months(args.months_after, args.months_before);

    let months = build_month_range(start_date, first_day_of_week, num_months);

    println!("{}", months.print())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_print_simple() {
        let start_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let first_day_of_week = Weekday::Mon;
        let month = build_month(start_date, first_day_of_week);

        insta::assert_snapshot!(month.print(), @r###"
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
    fn test_month_print_sun_first() {
        let start_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let first_day_of_week = Weekday::Sun;
        let month = build_month(start_date, first_day_of_week);

        insta::assert_snapshot!(month.print(), @r###"
             March 2024     
        Su Mo Tu We Th Fr Sa
                        1  2
         3  4  5  6  7  8  9
        10 11 12 13 14 15 16
        17 18 19 20 21 22 23
        24 25 26 27 28 29 30
        31                  
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

    #[test]
    fn test_month_range_print_simple() {
        let start_date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let first_day_of_week = Weekday::Mon;
        let month = build_month_range(start_date, first_day_of_week, 3);

        insta::assert_snapshot!(month.print(), @r###"
           February 2024           March 2024            April 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
                  1  2  3  4               1  2  3   1  2  3  4  5  6  7
         5  6  7  8  9 10 11   4  5  6  7  8  9 10   8  9 10 11 12 13 14
        12 13 14 15 16 17 18  11 12 13 14 15 16 17  15 16 17 18 19 20 21
        19 20 21 22 23 24 25  18 19 20 21 22 23 24  22 23 24 25 26 27 28
        26 27 28 29           25 26 27 28 29 30 31  29 30               
        "###);
    }

    #[test]
    fn test_month_range_print_sun_first() {
        let start_date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let first_day_of_week = Weekday::Sun;
        let month = build_month_range(start_date, first_day_of_week, 3);

        insta::assert_snapshot!(month.print(), @r###"
           February 2024           March 2024            April 2024     
        Su Mo Tu We Th Fr Sa  Su Mo Tu We Th Fr Sa  Su Mo Tu We Th Fr Sa
                     1  2  3                  1  2      1  2  3  4  5  6
         4  5  6  7  8  9 10   3  4  5  6  7  8  9   7  8  9 10 11 12 13
        11 12 13 14 15 16 17  10 11 12 13 14 15 16  14 15 16 17 18 19 20
        18 19 20 21 22 23 24  17 18 19 20 21 22 23  21 22 23 24 25 26 27
        25 26 27 28 29        24 25 26 27 28 29 30  28 29 30            
                              31                                        
        "###);
    }

    #[test]
    fn test_determine_start_date_no_args() {
        let start_date = determine_start_date(None, None, None);

        assert_eq!(start_date, Local::now().date_naive().with_day(1).unwrap());
    }

    #[test]
    fn test_determine_start_date_with_months_before() {
        let start_date = determine_start_date(None, None, Some(1));

        assert_eq!(
            start_date,
            Local::now()
                .date_naive()
                .with_day(1)
                .unwrap()
                .checked_sub_months(chrono::Months::new(1))
                .unwrap()
        );
    }
}
