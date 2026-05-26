use colored::Colorize;

use super::is_plain;

/// Highlight a name/value (cyan).
pub fn highlight(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.cyan().to_string()
    }
}

/// Bold text.
pub fn bold(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.bold().to_string()
    }
}

/// Dimmed text for secondary info.
pub fn dim(s: &str) -> String {
    if is_plain() {
        s.to_string()
    } else {
        s.dimmed().to_string()
    }
}
