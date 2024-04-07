use clap::{Parser, ValueEnum};
use itertools::Itertools;
use std::fmt;
use std::io::IsTerminal;

use chrono::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Display a specific year, quarter, or month.
    ///
    /// Examples: 2024, Q1, 2024Q1, FY2024, FYQ2, FY2024Q1
    ///
    /// Disables usage of `--year` and `--month` flags.
    #[arg(value_parser = parse_date_input, conflicts_with_all = ["year", "month"])]
    date_input: Option<DateInput>,

    /// Sets the first day of the week. If not set, defaults to the system preference.
    #[arg(short, long, value_enum)]
    first_day_of_week: Option<FirstDayOfWeek>,

    /// The year to display.
    #[arg(short, long, conflicts_with = "date_input")]
    year: Option<i32>,

    /// The month to display.
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..=12), conflicts_with = "date_input", requires = "year")]
    month: Option<u32>,

    /// Display the number of months after the current month.
    #[arg(short = 'A', long, value_parser = clap::value_parser!(u32).range(1..=12))]
    months_after: Option<u32>,

    /// Display the number of months before the current month.
    #[arg(short = 'B', long, value_parser = clap::value_parser!(u32).range(1..=12))]
    months_before: Option<u32>,

    /// Enable or disable colored output.
    #[arg(
            long,
            require_equals = true,
            value_name = "WHEN",
            num_args = 0..=1,
            default_value_t = ColorWhen::Auto,
            default_missing_value = "always",
            value_enum
        )]
    color: ColorWhen,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
enum DateInput {
    Year(Year),
    YearMonth(Year, u32),
    YearQuarter(Year, Quarter),
}

#[derive(Clone, Debug, PartialEq)]
enum Quarter {
    Q1,
    Q2,
    Q3,
    Q4,
}

#[derive(Clone, Debug, PartialEq)]
struct Year {
    style: YearStyle,
    year: i32,
}

#[derive(Clone, Debug, PartialEq, Copy)]
enum YearStyle {
    Calendar,
    Fiscal,
}

fn parse_date_input(s: &str) -> Result<DateInput, String> {
    // default to calendar year style
    let style = YearStyle::Calendar;

    // support bare Q1, Q2, Q3, Q4 format
    if let Some(date) = parse_bare_quarter(s, style) {
        return Ok(date);
    }

    // support anything prefixed with FY
    if let Some(fiscal_year_stripped) = s.to_uppercase().strip_prefix("FY") {
        let style = YearStyle::Fiscal;
        if let Ok(year) = fiscal_year_stripped.parse::<i32>() {
            return Ok(DateInput::Year(Year { style, year }));
        }

        // support bare Q1, Q2, Q3, Q4 format
        if let Some(date) = parse_bare_quarter(fiscal_year_stripped, style) {
            return Ok(date);
        }

        // support FY2024-Q1 format
        if fiscal_year_stripped.contains("-Q") {
            if let Some(date) = parse_year_quarter(fiscal_year_stripped, "-Q", style) {
                return Ok(date);
            }
        }
        // support FY2024Q1 format
        if fiscal_year_stripped.contains('Q') {
            if let Some(date) = parse_year_quarter(fiscal_year_stripped, "Q", style) {
                return Ok(date);
            }
        }
    }

    if let Ok(year) = s.parse::<i32>() {
        match s.len() {
            // support 24 format
            // support 2024 format
            2 | 4 => {
                return Ok(DateInput::Year(Year { style, year }));
            }

            // support 202401 format
            6 => {
                let (year, month) = s.split_at(4);

                if let (Ok(year), Ok(month)) = (year.parse::<i32>(), month.parse::<u32>()) {
                    if (1..=12).contains(&month) {
                        return Ok(DateInput::YearMonth(Year { style, year }, month));
                    }

                    return Err(format!(
                        "Invalid month detected (must be 1 - 12): {}",
                        month
                    ));
                }
            }

            // fall through to the error case below
            _ => {}
        }

        return Err(format!("Invalid date format: {}", s));
    }

    // support 2024-Q1 format
    if s.contains("-Q") {
        if let Some(date) = parse_year_quarter(s, "-Q", style) {
            return Ok(date);
        }
    }
    // support 2024Q1 format
    if s.contains('Q') {
        if let Some(date) = parse_year_quarter(s, "Q", style) {
            return Ok(date);
        }
    }

    // support 2024-01 format
    if let Some((year, month)) = s.split_once('-') {
        if let (Ok(year), Ok(month)) = (year.parse::<i32>(), month.parse::<u32>()) {
            if (1..=12).contains(&month) {
                return Ok(DateInput::YearMonth(Year { style, year }, month));
            }
        }
    }

    Err(format!("Invalid date format: {}", s))
}

fn parse_year_quarter(s: &str, delimiter: &str, style: YearStyle) -> Option<DateInput> {
    if let Some((year, quarter)) = s.split_once(delimiter) {
        // FIXME: Convert this to an error (change return type to Result<Option>)
        if let (Ok(year), Some(quarter)) = (
            year.parse::<i32>(),
            match quarter {
                "1" => Some(Quarter::Q1),
                "2" => Some(Quarter::Q2),
                "3" => Some(Quarter::Q3),
                "4" => Some(Quarter::Q4),
                _ => None,
            },
        ) {
            return Some(DateInput::YearQuarter(Year { style, year }, quarter));
        }
    }

    None
}

fn normalize_short_year(current_date: NaiveDate, year: i32) -> i32 {
    match year {
        0..=99 => {
            let current_year = current_date.year();
            let current_century = current_year / 100;

            current_century * 100 + year
        }
        _ => year,
    }
}

fn parse_bare_quarter(s: &str, style: YearStyle) -> Option<DateInput> {
    if let Some(quarter) = match s.to_uppercase().as_str() {
        "Q1" => Some(Quarter::Q1),
        "Q2" => Some(Quarter::Q2),
        "Q3" => Some(Quarter::Q3),
        "Q4" => Some(Quarter::Q4),
        _ => None,
    } {
        let year = determine_current_year(style);
        let year = Year { style, year };
        return Some(DateInput::YearQuarter(year, quarter));
    }

    None
}

fn determine_current_year(style: YearStyle) -> i32 {
    let today = chrono::Local::now().date_naive();
    let current_year = today.year();

    match style {
        YearStyle::Calendar => current_year,
        YearStyle::Fiscal => {
            let current_month = today.month();

            if current_month <= 6 {
                current_year
            } else {
                current_year + 1
            }
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

#[derive(Debug)]
struct MonthRange {
    months: Vec<Month>,
}

impl MonthRange {
    fn print(&self, color: ColorWhen, current_date: NaiveDate) -> String {
        let mut output = String::new();

        for (chunk_index, chunk) in self.months.chunks(3).enumerate() {
            if chunk_index > 0 {
                output.push('\n');
            }

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
                        Some(week) => {
                            week.print(color, current_date, month.first_day_of_week, &mut output)
                        }
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

    fn print(&self, color: ColorWhen, current_date: NaiveDate) -> String {
        let mut output = String::new();

        self.print_header(&mut output);
        output.push('\n');
        self.print_weekday_header(&mut output);
        output.push('\n');

        for week in &self.weeks {
            week.print(color, current_date, self.first_day_of_week, &mut output);
            output.push('\n');
        }

        output
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let today = chrono::Local::now().date_naive();
        write!(f, "{}", self.print(ColorWhen::Auto, today))
    }
}

fn format_date(color: ColorWhen, current_date: NaiveDate, date: Option<NaiveDate>) -> String {
    match date {
        Some(d) => {
            if show_color(color) && d == current_date {
                let highlight_on = "\x1B[7m"; // ANSI code for reverse video on
                let highlight_off = "\x1B[27m"; // ANSI code for reverse video off

                format!("{}{:2}{}", highlight_on, d.day(), highlight_off)
            } else {
                format!("{:2}", d.day())
            }
        }
        None => "  ".to_string(),
    }
}

fn show_color(color: ColorWhen) -> bool {
    // Check for the environment variable override first
    if let Ok(val) = std::env::var("FORCE_COLOR") {
        match val.as_str() {
            "1" | "true" => return true,
            "0" | "false" => return false,
            _ => {}
        }
    }

    match color {
        ColorWhen::Always => true,
        ColorWhen::Auto => is_interactive(),
        ColorWhen::Never => false,
    }
}

fn is_interactive() -> bool {
    std::io::stdout().is_terminal()
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

    fn print(
        &self,
        color: ColorWhen,
        current_date: NaiveDate,
        first_day_of_week: Weekday,
        output: &mut String,
    ) {
        match first_day_of_week {
            Weekday::Mon => {
                output.push_str(&format!(
                    "{} {} {} {} {} {} {}",
                    format_date(color, current_date, self.monday),
                    format_date(color, current_date, self.tuesday),
                    format_date(color, current_date, self.wednesday),
                    format_date(color, current_date, self.thursday),
                    format_date(color, current_date, self.friday),
                    format_date(color, current_date, self.saturday),
                    format_date(color, current_date, self.sunday)
                ));
            }
            Weekday::Sun => {
                output.push_str(&format!(
                    "{} {} {} {} {} {} {}",
                    format_date(color, current_date, self.sunday),
                    format_date(color, current_date, self.monday),
                    format_date(color, current_date, self.tuesday),
                    format_date(color, current_date, self.wednesday),
                    format_date(color, current_date, self.thursday),
                    format_date(color, current_date, self.friday),
                    format_date(color, current_date, self.saturday),
                ));
            }

            _ => {
                panic!("Invalid first day of week specified: {}", first_day_of_week);
            }
        };
    }
}

fn build_month(days: Vec<NaiveDate>, first_day_of_week: Weekday) -> Month {
    let start_date = *days.first().expect("no days in month");
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
    end_date: NaiveDate,
    first_day_of_week: Weekday,
) -> MonthRange {
    let months: Vec<Month> = date_range(start_date, end_date)
        .group_by(|&date| (date.year(), date.month()))
        .into_iter()
        .map(|((_year, _month), group)| build_month(group.collect(), first_day_of_week))
        .collect();

    MonthRange { months }
}

fn date_range(start: NaiveDate, end: NaiveDate) -> impl Iterator<Item = NaiveDate> {
    std::iter::successors(Some(start), move |&d| {
        if d < end {
            let next = d.succ_opt().unwrap();

            Some(next)
        } else {
            None
        }
    })
}

fn normalize_date_input_for_two_digit_year(
    current_date: NaiveDate,
    date_input: Option<DateInput>,
) -> Option<DateInput> {
    if let Some(date_input) = date_input {
        match date_input {
            DateInput::Year(year) => {
                let updated_year = normalize_short_year(current_date, year.year);

                return Some(DateInput::Year(Year {
                    year: updated_year,
                    ..year
                }));
            }
            DateInput::YearMonth(year, month) => {
                let updated_year = normalize_short_year(current_date, year.year);

                return Some(DateInput::YearMonth(
                    Year {
                        year: updated_year,
                        ..year
                    },
                    month,
                ));
            }
            DateInput::YearQuarter(year, quarter) => {
                let updated_year = normalize_short_year(current_date, year.year);

                return Some(DateInput::YearQuarter(
                    Year {
                        year: updated_year,
                        ..year
                    },
                    quarter,
                ));
            }
        }
    }

    date_input
}

fn determine_date_range(current_date: NaiveDate, args: Arguments) -> (NaiveDate, NaiveDate) {
    // `--year` and `--month` are mutually exclusive with the date_input field, so we can safely
    // normalize `--year` and `--month` into DateInput::YearMonth without issue
    let args = match (args.year, args.month) {
        (Some(year), Some(month)) => {
            let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| {
                panic!("Invalid year and month combination: {}-{:02}", year, month)
            });

            let date_input = Some(DateInput::YearMonth(
                Year {
                    style: YearStyle::Calendar,
                    year: date.year(),
                },
                date.month(),
            ));

            Arguments { date_input, ..args }
        }
        (Some(year), None) => {
            let date = NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap_or_else(|| panic!("Invalid year: {}", year));

            let date_input = Some(DateInput::Year(Year {
                style: YearStyle::Calendar,
                year: date.year(),
            }));

            Arguments { date_input, ..args }
        }
        _ => args,
    };

    // Now populate `date_input` if it isn't present already
    let args = if args.date_input.is_none() {
        let date_input = Some(DateInput::YearMonth(
            Year {
                style: YearStyle::Calendar,
                year: current_date.year(),
            },
            current_date.month(),
        ));

        Arguments { date_input, ..args }
    } else {
        args
    };

    let (start_date, end_date) = match args.date_input.expect("Date input is required") {
        DateInput::Year(year) => (
            NaiveDate::from_ymd_opt(year.year, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(year.year, 12, 31).unwrap(),
        ),
        DateInput::YearMonth(year, month) => {
            let start_date = NaiveDate::from_ymd_opt(year.year, month, 1).unwrap();
            let end_date = last_day_of_month_for(start_date);

            (start_date, end_date)
        }
        DateInput::YearQuarter(year, quarter) => {
            let (start_month, end_month) = match (year.style, quarter) {
                (YearStyle::Calendar, Quarter::Q1) => (1, 3),
                (YearStyle::Calendar, Quarter::Q2) => (4, 6),
                (YearStyle::Calendar, Quarter::Q3) => (7, 9),
                (YearStyle::Calendar, Quarter::Q4) => (10, 12),
                (YearStyle::Fiscal, Quarter::Q1) => (7, 9),
                (YearStyle::Fiscal, Quarter::Q2) => (10, 12),
                (YearStyle::Fiscal, Quarter::Q3) => (1, 3),
                (YearStyle::Fiscal, Quarter::Q4) => (4, 6),
            };

            let start_date = NaiveDate::from_ymd_opt(year.year, start_month, 1).unwrap();
            let first_day_of_end_month = NaiveDate::from_ymd_opt(year.year, end_month, 1).unwrap();
            let end_date = last_day_of_month_for(first_day_of_end_month);

            (start_date, end_date)
        }
    };

    let start_date = if let Some(months_before) = args.months_before {
        if start_date.month() <= months_before {
            NaiveDate::from_ymd_opt(
                start_date.year() - 1,
                12 - months_before + start_date.month(),
                1,
            )
        } else {
            NaiveDate::from_ymd_opt(start_date.year(), start_date.month() - months_before, 1)
        }
        .expect("couldn't determine a valid start date")
    } else {
        start_date
    };

    let end_date = if let Some(months_after) = args.months_after {
        let end_date = if end_date.month() + months_after > 12 {
            NaiveDate::from_ymd_opt(end_date.year() + 1, end_date.month() + months_after - 12, 1)
        } else {
            NaiveDate::from_ymd_opt(end_date.year(), end_date.month() + months_after, 1)
        }
        .expect("couldn't determine a valid end date");

        last_day_of_month_for(end_date)
    } else {
        end_date
    };

    (start_date, end_date)
}

fn last_day_of_month_for(date: NaiveDate) -> NaiveDate {
    let (next_month_year, next_month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let next_month_start_date = NaiveDate::from_ymd_opt(next_month_year, next_month, 1).unwrap();

    next_month_start_date.pred_opt().unwrap()
}

fn print(args: Arguments, current_date: NaiveDate) -> String {
    let color = args.color;
    let date_input = normalize_date_input_for_two_digit_year(current_date, args.date_input);

    let args = Arguments { date_input, ..args };
    let first_day_of_week = determine_default_first_day_of_week(args.first_day_of_week);
    let (start_date, end_date) = determine_date_range(current_date, args);

    let months = build_month_range(start_date, end_date, first_day_of_week);

    months.print(color, current_date)
}

fn main() {
    let args = Arguments::parse();
    let today = chrono::Local::now().date_naive();

    println!("{}", print(args, today));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn args<I, T>(itr: I) -> Arguments
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Arguments::parse_from(itr)
    }

    #[test]
    fn test_parse_date_input_year() {
        let style = YearStyle::Calendar;

        assert_eq!(
            parse_date_input("2024"),
            Ok(DateInput::Year(Year { style, year: 2024 }))
        );
        assert_eq!(
            parse_date_input("2000"),
            Ok(DateInput::Year(Year { style, year: 2000 }))
        );
    }

    #[test]
    fn test_parse_date_input_quarter() {
        let style = YearStyle::Calendar;
        let year = chrono::Local::now().year();

        assert_eq!(
            parse_date_input("Q1"),
            Ok(DateInput::YearQuarter(Year { style, year }, Quarter::Q1))
        );
        assert_eq!(
            parse_date_input("Q2"),
            Ok(DateInput::YearQuarter(Year { style, year }, Quarter::Q2))
        );
        assert_eq!(
            parse_date_input("Q3"),
            Ok(DateInput::YearQuarter(Year { style, year }, Quarter::Q3))
        );
        assert_eq!(
            parse_date_input("Q4"),
            Ok(DateInput::YearQuarter(Year { style, year }, Quarter::Q4))
        );
    }

    #[test]
    fn test_parse_date_input_fiscal_year() {
        let style = YearStyle::Fiscal;

        assert_eq!(
            parse_date_input("FY2024"),
            Ok(DateInput::Year(Year { style, year: 2024 }))
        );
        assert_eq!(
            parse_date_input("FY1900"),
            Ok(DateInput::Year(Year { style, year: 1900 }))
        );
    }

    #[test]
    fn test_parse_date_input_year_quarter() {
        let style = YearStyle::Calendar;

        assert_eq!(
            parse_date_input("2024Q1"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 2024 },
                Quarter::Q1
            ))
        );
        assert_eq!(
            parse_date_input("2000Q3"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 2000 },
                Quarter::Q3
            ))
        );
        assert_eq!(
            parse_date_input("1900Q2"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 1900 },
                Quarter::Q2
            ))
        );
    }

    #[test]
    fn test_parse_date_input_fiscal_quarter() {
        let style = YearStyle::Fiscal;

        assert_eq!(
            parse_date_input("FY2024Q1"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 2024 },
                Quarter::Q1
            ))
        );
        assert_eq!(
            parse_date_input("FY2000Q2"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 2000 },
                Quarter::Q2
            ))
        );
        assert_eq!(
            parse_date_input("FY1900Q3"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 1900 },
                Quarter::Q3
            ))
        );
        assert_eq!(
            parse_date_input("FY2024Q4"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 2024 },
                Quarter::Q4
            ))
        );
        assert_eq!(
            parse_date_input("FY2024-Q1"),
            Ok(DateInput::YearQuarter(
                Year { style, year: 2024 },
                Quarter::Q1
            ))
        );
    }

    #[test]
    fn test_parse_date_input_year_month() {
        let style = YearStyle::Calendar;

        assert_eq!(
            parse_date_input("2024-01"),
            Ok(DateInput::YearMonth(Year { style, year: 2024 }, 1))
        );
        assert_eq!(
            parse_date_input("202401"),
            Ok(DateInput::YearMonth(Year { style, year: 2024 }, 1))
        );
        assert_eq!(
            parse_date_input("2000-06"),
            Ok(DateInput::YearMonth(Year { style, year: 2000 }, 6))
        );
        assert_eq!(
            parse_date_input("200006"),
            Ok(DateInput::YearMonth(Year { style, year: 2000 }, 6))
        );
        assert_eq!(
            parse_date_input("1900-12"),
            Ok(DateInput::YearMonth(Year { style, year: 1900 }, 12))
        );
        assert_eq!(
            parse_date_input("190012"),
            Ok(DateInput::YearMonth(Year { style, year: 1900 }, 12))
        );
    }

    #[test]
    #[ignore]
    fn test_parse_date_two_digit_year() {
        assert_eq!(
            parse_date_input("FY24Q1"),
            Ok(DateInput::YearQuarter(
                Year {
                    style: YearStyle::Fiscal,
                    year: 2024
                },
                Quarter::Q1
            ))
        );

        assert_eq!(
            parse_date_input("FY25Q2"),
            Ok(DateInput::YearQuarter(
                Year {
                    style: YearStyle::Fiscal,
                    year: 2025
                },
                Quarter::Q2
            ))
        );

        assert_eq!(
            parse_date_input("FY24"),
            Ok(DateInput::Year(Year {
                style: YearStyle::Fiscal,
                year: 2024
            },))
        );

        assert_eq!(
            parse_date_input("FY25"),
            Ok(DateInput::Year(Year {
                style: YearStyle::Fiscal,
                year: 2025
            },))
        );

        assert_eq!(
            parse_date_input("25Q2"),
            Ok(DateInput::YearQuarter(
                Year {
                    style: YearStyle::Calendar,
                    year: 2025
                },
                Quarter::Q2
            ))
        );

        assert_eq!(
            parse_date_input("24"),
            Ok(DateInput::Year(Year {
                style: YearStyle::Calendar,
                year: 2024
            },))
        );
    }

    #[test]
    fn test_parse_date_input_invalid() {
        assert!(parse_date_input("").is_err());
        assert!(parse_date_input("invalid").is_err());
        assert!(parse_date_input("2024-13").is_err());
        assert!(parse_date_input("FY").is_err());
        assert!(parse_date_input("Q5").is_err());
    }

    #[test]
    fn test_month_print_simple() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        let args = args(["cal"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
             March 2024     
        Mo Tu We Th Fr Sa Su
                     1  2  3
         4  5  6  7  8  9 10
        11 12 13 14 15 16 17
        18 19 20 21 22 23 24
        25 26 27 28 29 30 31
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_print_quarter() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let args = args(["cal", "Q1"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
            January 2024         February 2024           March 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7            1  2  3  4               1  2  3
         8  9 10 11 12 13 14   5  6  7  8  9 10 11   4  5  6  7  8  9 10
        15 16 17 18 19 20 21  12 13 14 15 16 17 18  11 12 13 14 15 16 17
        22 23 24 25 26 27 28  19 20 21 22 23 24 25  18 19 20 21 22 23 24
        29 30 31              26 27 28 29           25 26 27 28 29 30 31
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_print_quarter_lowercase() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let args = args(["cal", "q1"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
            January 2024         February 2024           March 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7            1  2  3  4               1  2  3
         8  9 10 11 12 13 14   5  6  7  8  9 10 11   4  5  6  7  8  9 10
        15 16 17 18 19 20 21  12 13 14 15 16 17 18  11 12 13 14 15 16 17
        22 23 24 25 26 27 28  19 20 21 22 23 24 25  18 19 20 21 22 23 24
        29 30 31              26 27 28 29           25 26 27 28 29 30 31
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_print_fiscal_quarter() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let args = args(["cal", "FYQ3"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
            January 2024         February 2024           March 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7            1  2  3  4               1  2  3
         8  9 10 11 12 13 14   5  6  7  8  9 10 11   4  5  6  7  8  9 10
        15 16 17 18 19 20 21  12 13 14 15 16 17 18  11 12 13 14 15 16 17
        22 23 24 25 26 27 28  19 20 21 22 23 24 25  18 19 20 21 22 23 24
        29 30 31              26 27 28 29           25 26 27 28 29 30 31
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_print_fiscal_quarter_lowercase() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let args = args(["cal", "fyq3"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
            January 2024         February 2024           March 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7            1  2  3  4               1  2  3
         8  9 10 11 12 13 14   5  6  7  8  9 10 11   4  5  6  7  8  9 10
        15 16 17 18 19 20 21  12 13 14 15 16 17 18  11 12 13 14 15 16 17
        22 23 24 25 26 27 28  19 20 21 22 23 24 25  18 19 20 21 22 23 24
        29 30 31              26 27 28 29           25 26 27 28 29 30 31
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_print_year() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let args = args(["cal", "2024"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
            January 2024         February 2024           March 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7            1  2  3  4               1  2  3
         8  9 10 11 12 13 14   5  6  7  8  9 10 11   4  5  6  7  8  9 10
        15 16 17 18 19 20 21  12 13 14 15 16 17 18  11 12 13 14 15 16 17
        22 23 24 25 26 27 28  19 20 21 22 23 24 25  18 19 20 21 22 23 24
        29 30 31              26 27 28 29           25 26 27 28 29 30 31
                                                                        

             April 2024             May 2024             June 2024      
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7         1  2  3  4  5                  1  2
         8  9 10 11 12 13 14   6  7  8  9 10 11 12   3  4  5  6  7  8  9
        15 16 17 18 19 20 21  13 14 15 16 17 18 19  10 11 12 13 14 15 16
        22 23 24 25 26 27 28  20 21 22 23 24 25 26  17 18 19 20 21 22 23
        29 30                 27 28 29 30 31        24 25 26 27 28 29 30
                                                                        

             July 2024            August 2024          September 2024   
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
         1  2  3  4  5  6  7            1  2  3  4                     1
         8  9 10 11 12 13 14   5  6  7  8  9 10 11   2  3  4  5  6  7  8
        15 16 17 18 19 20 21  12 13 14 15 16 17 18   9 10 11 12 13 14 15
        22 23 24 25 26 27 28  19 20 21 22 23 24 25  16 17 18 19 20 21 22
        29 30 31              26 27 28 29 30 31     23 24 25 26 27 28 29
                                                    30                  

            October 2024         November 2024         December 2024    
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
            1  2  3  4  5  6               1  2  3                     1
         7  8  9 10 11 12 13   4  5  6  7  8  9 10   2  3  4  5  6  7  8
        14 15 16 17 18 19 20  11 12 13 14 15 16 17   9 10 11 12 13 14 15
        21 22 23 24 25 26 27  18 19 20 21 22 23 24  16 17 18 19 20 21 22
        28 29 30 31           25 26 27 28 29 30     23 24 25 26 27 28 29
                                                    30 31               
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_print_future_fiscal_quarter() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 5, 20).unwrap();
        let args = args(["cal", "FY2090Q3"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
            January 2090         February 2090           March 2090     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
                           1         1  2  3  4  5         1  2  3  4  5
         2  3  4  5  6  7  8   6  7  8  9 10 11 12   6  7  8  9 10 11 12
         9 10 11 12 13 14 15  13 14 15 16 17 18 19  13 14 15 16 17 18 19
        16 17 18 19 20 21 22  20 21 22 23 24 25 26  20 21 22 23 24 25 26
        23 24 25 26 27 28 29  27 28                 27 28 29 30 31      
        30 31                                                           
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_month_print_sun_first() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        let args = args(["cal", "--first-day-of-week", "sunday"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
             March 2024     
        Su Mo Tu We Th Fr Sa
                        1  2
         3  4  5  6  7  8  9
        10 11 12 13 14 15 16
        17 18 19 20 21 22 23
        24 25 26 27 28 29 30
        31                  
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_build_month_leap_february() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 2, 20).unwrap();
        let args = args(["cal"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
           February 2024    
        Mo Tu We Th Fr Sa Su
                  1  2  3  4
         5  6  7  8  9 10 11
        12 13 14 15 16 17 18
        19 20 21 22 23 24 25
        26 27 28 29         
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_month_range_print_simple() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        let args = args(["cal", "-B", "1", "-A", "1"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
           February 2024           March 2024            April 2024     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
                  1  2  3  4               1  2  3   1  2  3  4  5  6  7
         5  6  7  8  9 10 11   4  5  6  7  8  9 10   8  9 10 11 12 13 14
        12 13 14 15 16 17 18  11 12 13 14 15 16 17  15 16 17 18 19 20 21
        19 20 21 22 23 24 25  18 19 20 21 22 23 24  22 23 24 25 26 27 28
        26 27 28 29           25 26 27 28 29 30 31  29 30               
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_month_range_print_long_args() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2023, 3, 20).unwrap();
        let args = args(["cal", "--months-before", "1", "--months-after", "1"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
           February 2023           March 2023            April 2023     
        Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
               1  2  3  4  5         1  2  3  4  5                  1  2
         6  7  8  9 10 11 12   6  7  8  9 10 11 12   3  4  5  6  7  8  9
        13 14 15 16 17 18 19  13 14 15 16 17 18 19  10 11 12 13 14 15 16
        20 21 22 23 24 25 26  20 21 22 23 24 25 26  17 18 19 20 21 22 23
        27 28                 27 28 29 30 31        24 25 26 27 28 29 30
        "###);

        std::env::remove_var("FORCE_COLOR");
    }

    #[test]
    fn test_month_range_print_sun_first() {
        std::env::set_var("FORCE_COLOR", "0");

        let current_date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        let args = args(["cal", "--first-day-of-week", "sunday", "-B", "1", "-A", "1"]);

        insta::assert_snapshot!(print(args, current_date), @r###"
           February 2024           March 2024            April 2024     
        Su Mo Tu We Th Fr Sa  Su Mo Tu We Th Fr Sa  Su Mo Tu We Th Fr Sa
                     1  2  3                  1  2      1  2  3  4  5  6
         4  5  6  7  8  9 10   3  4  5  6  7  8  9   7  8  9 10 11 12 13
        11 12 13 14 15 16 17  10 11 12 13 14 15 16  14 15 16 17 18 19 20
        18 19 20 21 22 23 24  17 18 19 20 21 22 23  21 22 23 24 25 26 27
        25 26 27 28 29        24 25 26 27 28 29 30  28 29 30            
                              31                                        
        "###);

        std::env::remove_var("FORCE_COLOR");
    }
}
