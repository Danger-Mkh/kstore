# Contributing

Thank you for considering contributing to the Simple Key-Value Store project! This document outlines how you can help improve this Rust-based server.

## How to Contribute

### Reporting Bugs
- Check the [Issues](https://github.com/behdanisohrab/kstore/issues) tab to see if the bug is already reported.
- If not, open a new issue with:
  - A clear title and description.
  - Steps to reproduce the bug.
  - Expected and actual behavior.
  - Your environment (Rust version, OS, etc.).

### Suggesting Features
- Open an issue with the label `enhancement`.
- Describe the feature, why it’s useful, and how it could work.

### Submitting Code
1. **Fork the Repository**:
   - Click "Fork" on the repo page and clone your fork:
     ```
     git clone https://github.com/behdanisohrab/kstore.git
     ```

2. **Create a Branch**:
   - Work on a new branch for your changes:
     ```
     git checkout -b feature-or-fix-name
     ```

3. **Make Changes**:
   - Follow Rust coding conventions (run `cargo fmt`).
   - Keep changes focused and well-documented.
   - Test your code with `cargo test` (add tests if applicable).

4. **Commit and Push**:
   - Write clear commit messages:
     ```
     git commit -m "Add feature X" -m "Details about the change"
     git push origin feature-or-fix-name
     ```

5. **Open a Pull Request**:
   - Go to the original repo and create a PR from your branch.
   - Link any related issues and describe your changes.

## Development Setup
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Clone the repo and run:

```bash
cargo build
cargo run
```
- The server will start at `http://127.0.0.1:8080`.

## Code Guidelines
- Use `cargo fmt` for consistent formatting.
- Add comments for complex logic.
- Keep dependencies minimal (currently only stdlib).
- Ensure thread safety with `Mutex` or similar where needed.


## Questions?
Feel free to open an issue or reach out directly inside github issues.

Happy coding!


