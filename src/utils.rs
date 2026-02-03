/// stringe function takes in a description function and a Result<T, Error> and it
/// will convert the error to the string if it is one, making some jobs easier and
/// without rust having to tell me I am converting thibgs to std::error::Error.
pub fn stringe<T, K: std::error::Error>(msg: &str, val: Result<T, K>) -> Result<T, String> {
    match val {
        Ok(v) => Ok(v),
        Err(e) => Err(String::from(msg) + ": " + e.to_string().as_str()),
    }
}

pub fn localization_getter(n: &String) -> String {
    format!("AppLocalizations.of(context)!.{n}")
}

pub fn id_string(s: &str) -> String {
    let mut temp = String::new();

    for c in s.chars() {
        // Each match arm now manages its own borrowing, which is short-lived and non-conflicting.
        match c {
            '*' | '\'' | '-' | '!' | '?' | '.' | ',' | ':' | ';' | '(' | ')' | '[' | ']' | '{'
            | '}' | '<' | '>' | '/' | '\\' | '|' | '#' | '@' | '&' | '%' | '$' | '^' | '+'
            | '=' | '~' | '`' | '"' => {
                let word = match c {
                    '*' => "asterix",
                    '\'' => "apostrophe",
                    '-' => "hyphen",
                    '!' => "exclamation_mark",
                    '?' => "question_mark",
                    '.' => "period",
                    ',' => "comma",
                    ':' => "colon",
                    ';' => "semicolon",
                    '(' => "open_parenthesis",
                    ')' => "close_parenthesis",
                    '[' => "open_bracket",
                    ']' => "close_bracket",
                    '{' => "open_brace",
                    '}' => "close_brace",
                    '<' => "less_than",
                    '>' => "greater_than",
                    '/' => "forward_slash",
                    '\\' => "backslash",
                    '|' => "pipe",
                    '#' => "hash",
                    '@' => "at",
                    '&' => "ampersand",
                    '%' => "percent",
                    '$' => "dollar",
                    '^' => "caret",
                    '+' => "plus",
                    '=' => "equals",
                    '~' => "tilde",
                    '`' => "backtick",
                    '"' => "double_quote",
                    _ => unreachable!(),
                };
                if !temp.is_empty() && !temp.ends_with(' ') {
                    temp.push(' ');
                }
                temp.push_str(word);
                temp.push(' ');
            }

            c if c.is_alphanumeric() || c == ' ' || c == '_' => temp.push(c),

            _ => {
                let w = format!("char_{:x}", c as u32);
                if !temp.is_empty() && !temp.ends_with(' ') {
                    temp.push(' ');
                }
                temp.push_str(&w);
                temp.push(' ');
            }
        }
    }

    let words: Vec<&str> = temp.split_whitespace().collect();

    if words.is_empty() {
        return String::new();
    }

    // Build camelCase:
    let mut out = String::new();
    for (i, &word) in words.iter().enumerate() {
        if word.is_empty() {
            continue;
        }
        if i == 0 {
            out.push_str(&word.to_lowercase());
        } else {
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                out.extend(first_char.to_uppercase());
                out.push_str(&chars.as_str().to_lowercase());
            }
        }
    }

    out
}
