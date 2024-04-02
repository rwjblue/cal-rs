# cal-rs

```cal-rs``` is a simple command-line calendar application written in Rust. It
allows you to display a calendar for a specific month and year, with options to
customize the first day of the week.

```text
     April 2024     
Mo Tu We Th Fr Sa Su
 1  2  3  4  5  6  7
 8  9 10 11 12 13 14
15 16 17 18 19 20 21
22 23 24 25 26 27 28
29 30               
```

## Features

- Display a calendar for a given month and year
- Customize the first day of the week (Sunday or Monday)
- On macOS the first day of the week is determined by the system preference (via System Settings > General > Language & Region > First day of week)
- Automatically defaults to the current month and year if not specified

## Usage

```text
Usage: cal [OPTIONS]

Options:
  -f, --first-day-of-week <FIRST_DAY_OF_WEEK>
          Sets the first day of the week. If not set, defaults to the system preference [possible values: sunday, monday]
  -y, --year <YEAR>
          
  -m, --month <MONTH>
          
  -A <MONTHS_AFTER>
          Display the number of months after the current month
  -B <MONTHS_BEFORE>
          Display the number of months before the current month
  -h, --help
          Print help
  -V, --version
          Print version
```

## Examples

Display the calendar for the current month:

```text
> cal
     April 2024     
Mo Tu We Th Fr Sa Su
 1  2  3  4  5  6  7
 8  9 10 11 12 13 14
15 16 17 18 19 20 21
22 23 24 25 26 27 28
29 30               
```

Display the calendar for the current month, and include one month before and after:

```text
> cal -A 1 -B 1
     March 2024            April 2024             May 2024      
Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su  Mo Tu We Th Fr Sa Su
             1  2  3   1  2  3  4  5  6  7         1  2  3  4  5
 4  5  6  7  8  9 10   8  9 10 11 12 13 14   6  7  8  9 10 11 12
11 12 13 14 15 16 17  15 16 17 18 19 20 21  13 14 15 16 17 18 19
18 19 20 21 22 23 24  22 23 24 25 26 27 28  20 21 22 23 24 25 26
25 26 27 28 29 30 31  29 30                 27 28 29 30 31      
```

Display the calendar for a specific month and year:

```text
> cal -y 2024 -m 3
     March 2024     
Mo Tu We Th Fr Sa Su
             1  2  3
 4  5  6  7  8  9 10
11 12 13 14 15 16 17
18 19 20 21 22 23 24
25 26 27 28 29 30 31
```

Set the first day of the week to Sunday:

```text
> cal -f sunday
     April 2024     
Su Mo Tu We Th Fr Sa
    1  2  3  4  5  6
 7  8  9 10 11 12 13
14 15 16 17 18 19 20
21 22 23 24 25 26 27
28 29 30            
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
