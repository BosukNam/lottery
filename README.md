# Korean Lottery Number Generator

A Rust-based lottery number generator that analyzes historical Korean lottery data to generate new number combinations that exclude previous winning combinations.

## Features

- **Historical Data Analysis**: Parses lottery data from HTML files (static/1-600.xls, static/601-1186.xls)
- **Smart Number Generation**: Generates 6-number combinations (1-45) that avoid:
  - Previous 1st place winning combinations
  - Previous 2nd place winning combinations (5 winning numbers + bonus)
- **Interactive CLI**: User-friendly command-line interface with menu options
- **ESC Key Support**: Press ESC during round input to return to main menu
- **Data Persistence**: Automatically saves and loads lottery data from `lottery_data.txt`
- **New Round Entry**: Add new lottery rounds with date, winning numbers, and bonus number

## Installation

### Prerequisites
- Rust (latest stable version)
- Cargo package manager

### Build from source

1. Clone the repository:
```bash
git clone <repository-url>
cd lottery
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run
```

## Usage

The application provides an interactive menu with three main options:

### 1. Generate New Lottery Numbers
- Generates 6 unique numbers between 1-45
- Excludes all previous 1st place winning combinations
- Excludes all previous 2nd place winning combinations
- Displays recommended numbers for play

### 2. Add New Drawing Round
- Enter round number (with ESC support to return to main menu)
- Enter drawing date (format: YYYY.MM.DD)
- Enter 6 winning numbers (space-separated)
- Enter bonus number
- Automatically saves to data file

### 3. Exit Program
- Safely exits the application

## Data Format

The application works with lottery data in the following format:
- **Round**: Sequential drawing number
- **Date**: Drawing date (YYYY.MM.DD format)
- **Winning Numbers**: 6 numbers between 1-45
- **Bonus Number**: Single bonus number between 1-45

## File Structure

```
lottery/
├── src/
│   └── main.rs              # Main application code
├── static/
│   ├── 1-600.xls           # Historical lottery data (rounds 1-600)
│   └── 601-1186.xls        # Historical lottery data (rounds 601-1186)
├── lottery_data.txt         # Processed lottery data (auto-generated)
├── Cargo.toml              # Rust project configuration
└── README.md               # This file
```

## Dependencies

- `scraper` - HTML parsing for legacy data files
- `serde` - Serialization/deserialization
- `rand` - Random number generation
- `encoding_rs` - EUC-KR encoding support for Korean HTML files
- `crossterm` - Terminal input/output and keyboard event handling

## Technical Details

### Data Processing
- Parses HTML files with EUC-KR encoding
- Extracts lottery drawing information from table structures
- Validates number ranges (1-45 for lottery numbers)
- Sorts and stores data chronologically

### Number Generation Algorithm
- Uses cryptographically secure random number generation
- Maintains a HashSet of used combinations for O(1) lookup
- Generates combinations until a unique one is found
- Excludes both 1st place (6 numbers) and 2nd place (5 numbers + bonus) combinations

### User Interface
- Raw terminal mode for ESC key detection
- Real-time input feedback
- Clear menu navigation
- Error handling with user-friendly messages

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is open source and available under the MIT License.

## Disclaimer

This tool is for entertainment purposes only. Lottery numbers are random, and this generator does not guarantee winning outcomes. Please gamble responsibly.