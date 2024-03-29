use clap::Parser;
use std::fmt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    // TODO: figure out how to default to system preference
    // https://www.perplexity.ai/search/How-can-I-zngZ7lVUQMWV13U92fmJXQ
    /// Sets the first day of the week. Accepts full names ('Sunday', 'Monday', etc.),
    /// their two-letter abbreviations ('Su', 'Mo', etc.), or numbers from 1 (Sunday) to 7 (Saturday).
    /// If not set, defaults to the system preference.
    #[arg(short, long, value_parser = parse_day)]
    first_day_of_week: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn parse_day(s: &str) -> Result<DayOfWeek, String> {
    match s.to_lowercase().as_str() {
        "1" | "su" | "sunday" => Ok(DayOfWeek::Sunday),
        "2" | "mo" | "monday" => Ok(DayOfWeek::Monday),
        "3" | "tu" | "tuesday" => Ok(DayOfWeek::Tuesday),
        "4" | "we" | "wednesday" => Ok(DayOfWeek::Wednesday),
        "5" | "th" | "thursday" => Ok(DayOfWeek::Thursday),
        "6" | "fr" | "friday" => Ok(DayOfWeek::Friday),
        "7" | "sa" | "saturday" => Ok(DayOfWeek::Saturday),
        _ => Err(format!("Invalid day: {}. Please provide a day of the week, its abbreviation, or a number from 1 to 7.", s)),
    }
}

fn main() {
    let cli = Arguments::parse();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_names() {
        assert_eq!(parse_day("Sunday").unwrap(), DayOfWeek::Sunday);
        assert_eq!(parse_day("sunday").unwrap(), DayOfWeek::Sunday);
        assert_eq!(parse_day("SunDay").unwrap(), DayOfWeek::Sunday);

        assert_eq!(parse_day("Monday").unwrap(), DayOfWeek::Monday);
        assert_eq!(parse_day("monday").unwrap(), DayOfWeek::Monday);
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(parse_day("su").unwrap(), DayOfWeek::Sunday);
        assert_eq!(parse_day("Su").unwrap(), DayOfWeek::Sunday);
        assert_eq!(parse_day("tu").unwrap(), DayOfWeek::Tuesday);
        assert_eq!(parse_day("th").unwrap(), DayOfWeek::Thursday);
    }

    #[test]
    fn test_numbers() {
        assert_eq!(parse_day("1").unwrap(), DayOfWeek::Sunday);
        assert_eq!(parse_day("2").unwrap(), DayOfWeek::Monday);
        assert_eq!(parse_day("5").unwrap(), DayOfWeek::Thursday);
    }
}
