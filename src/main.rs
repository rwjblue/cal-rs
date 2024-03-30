use clap::{Parser, ValueEnum};

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

fn determine_default_first_day_of_week() -> FirstDayOfWeek {
    // TODO: figure out how to default to system preference
    // https://www.perplexity.ai/search/How-can-I-zngZ7lVUQMWV13U92fmJXQ
    FirstDayOfWeek::Monday
}

fn main() {
    let args = Arguments::parse();

    let first_day_of_week = args
        .first_day_of_week
        .unwrap_or_else(determine_default_first_day_of_week);
}

#[cfg(test)]
mod tests {
    use super::*;
}
