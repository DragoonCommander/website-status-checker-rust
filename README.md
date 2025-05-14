#  Website Status Checker (Rust)

This project is a concurrent command-line tool that checks the availability of websites in parallel using threads and Rust’s `reqwest` HTTP client.

---

##  Build Instructions

1. Clone the repository:

```bash
git clone https://github.com/DragoonCommander/website-status-checker-rust.git
cd website-status-checker-rust/status_checker
```

2. Build the project in release mode:

```bash
cargo build --release
```

The executable will be located at `target/release/status_checker`.

---

##  Usage Examples

###  Check URLs from a File

Create a text file `sites.txt`:

```
https://google.com
https://example.com
https://bad.url.that.will.fail
# This is a comment
```

Run the checker:

```bash
cargo run --release -- --file sites.txt --workers 4 --timeout 5 --retries 1
```

###  Check Inline URLs

```bash
cargo run --release -- https://google.com https://github.com --timeout 3 --workers 2
```

---

##  Features

- Multi-threaded URL processing (`--workers N`)
- Timeout for each request (`--timeout S`)
- Optional retry mechanism (`--retries N`)
- Accepts input from both a file and command-line arguments
- Outputs live results to the terminal
- Saves results as a structured JSON file: `status.json`

### JSON Output Fields

Each result contains:
- `url`: The original URL
- `status` or `error`: HTTP code or error string
- `time_ms`: Total response time in milliseconds
- `timestamp`: When the check was completed

---

##  Bonus Features (Optional)

This version implements **retry logic** via the `--retries` flag, which retries failed requests with a 100ms delay. Additional bonuses such as periodic checking and summary statistics can be added.

---

This tool is built using only the Rust standard library and the `reqwest` crate (with `blocking` feature), following academic constraints.