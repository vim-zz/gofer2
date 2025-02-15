# Gofer2

Gofer2 is a macOS menu bar application that provides quick translation lookups through double-copying text. It's designed to be lightweight, unobtrusive, and easily extensible through CSV files.

## Features

- Lives in your menu bar
- Double-copy text to trigger translation lookup
- Supports custom translations through CSV files
- System notifications for translation results
- Menu history of recent translations
- Click any translation to copy it to clipboard

## Installation

### From Source
```bash
# Clone the repository
git clone https://github.com/yourusername/gofer2.git
cd gofer2

# Build and run
cargo run

# Or build in release mode
cargo build --release
```

### Building for Distribution
```bash
cargo install cargo-bundle
cargo bundle --release
```

Copy the generated app bundle to your Applications folder.

### Requirements

- macOS 10.13 or later
- Rust 1.70 or later

## Usage

1. The app runs in your menu bar with a small icon
2. Double-copy (⌘C twice quickly) any text to look up its translation
3. If a translation is found:
   - A notification will appear showing the translation
   - The translation will be added to the menu bar history
4. Click any translation in the menu to copy it to clipboard

## Custom Translations

You can add your own translations by creating CSV files in `~/.gofer2/`:

1. Create the directory:
```bash
mkdir -p ~/.gofer2
```

2. Add CSV files with your translations:
```csv
# ~/.gofer2/my_translations.csv
en,fr
hello,bonjour
goodbye,au revoir
```

Requirements for CSV files:
- Must have exactly 2 columns
- First row must be headers (e.g., "en,fr")
- Values are trimmed automatically
- User translations override default translations

## License

MIT

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -am 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Credits

Created by Ofer Affias
