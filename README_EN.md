# Lottery Number Generator

A Rust program that generates new, non-duplicate lottery number combinations based on historical winning numbers from Korean Lotto 6/45.

## Key Features

- **Duplicate Prevention**: Generates number combinations that don't duplicate previous 1st and 2nd place winning numbers
- **Excel File Parsing**: Automatically parses existing lottery winning number data from Excel files
- **Data Management**: Saves and loads winning number data in text file format
- **New Draw Addition**: Manually add winning numbers for new draws

## Project Structure

```
lottery/
├── src/
│   └── main.rs          # Main program source code
├── static/
│   ├── 1-600.xls        # Winning numbers data for draws 1-600
│   └── 601-1187.xls     # Winning numbers data for draws 601-1187
├── lottery_data.txt     # Parsed winning numbers text data
├── Cargo.toml           # Project configuration and dependencies
└── README.md            # Project documentation
```

## Installation and Execution

### Requirements

- Rust 1.70.0 or higher
- Cargo (Rust package manager)

### Dependencies

- `scraper`: HTML/Excel file parsing
- `serde`: Data serialization/deserialization
- `rand`: Random number generation
- `encoding_rs`: EUC-KR encoding support

### How to Run

```bash
# Clone the project
git clone <repository-url>
cd lottery

# Build and run
cargo run
```

## Usage

When you run the program, the following menu is displayed:

```
=== Lottery Number Generator ===
1. Generate new lottery numbers
2. Add new draw
3. Exit
```

### 1. Generate New Lottery Numbers

Automatically generates 6 numbers that don't duplicate existing 1st and 2nd place winning numbers.

### 2. Add New Draw

You can manually add winning numbers for a new draw:
- Enter draw number
- Enter 6 winning numbers (separated by spaces)
- Enter bonus number

### 3. Exit

Terminates the program.

## Data Structure

### LotteryDrawing

```rust
struct LotteryDrawing {
    round: u32,      // Draw number
    numbers: [u8; 6], // 6 winning numbers (1-45)
    bonus: u8,       // Bonus number (1-45)
}
```

### Text File Format

```
Round,Number1,Number2,Number3,Number4,Number5,Number6,Bonus
1,10,23,29,33,37,40,16
2,9,13,21,25,32,42,2
...
```

## Algorithm

1. **1st Place Duplicate Removal**: Store existing 1st place winning number combinations in a HashSet
2. **2nd Place Duplicate Removal**: Also exclude combinations where one 1st place number is replaced with the bonus number
3. **Random Generation**: Randomly select 6 numbers from the range 1-45
4. **Duplicate Check**: Repeat until the generated combination doesn't duplicate existing winning numbers

## License

MIT License

## Contributing

Please submit bug reports or feature suggestions through Issues.