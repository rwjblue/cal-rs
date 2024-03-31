# cal-rs

```cal-rs``` is a simple command-line calendar application written in Rust. It
allows you to display a calendar for a specific month and year, with options to
customize the first day of the week.

## Features

- Display a calendar for a given month and year
- Customize the first day of the week (Sunday or Monday)
- Automatically defaults to the current month and year if not specified
- Supports leap years and correctly displays February with 29 days when applicable

## Usage

```
Usage: cal [OPTIONS]

Options:
  -f, --first-day-of-week <FIRST_DAY_OF_WEEK>
          Sets the first day of the week. If not set, defaults to the system preference [possible values: sunday, monday]
  -y, --year <YEAR>
          
  -m, --month <MONTH>
          
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

Display the calendar for the current month:
```
cal
```

Display the calendar for a specific month and year:
```
cal -y 2024 -m 3
```

Set the first day of the week to Sunday:
```
cal -f sunday
```

## Development

This project is primarily a learning exercise for exploring Rust programming.
It utilizes several Rust libraries and concepts, including:

- ```clap``` for parsing command-line arguments
- ```chrono``` for date and time handling
- ```insta``` for snapshot testing
- Rust's module system and project structure
- Rust's ownership and borrowing system
- Rust's ```Option``` and ```Result``` types for handling absence and errors

Feel free to explore the code, suggest improvements, or use it as a reference
for your own Rust learning journey!

## License

This project is open-source and available under the [MIT License](LICENSE).
