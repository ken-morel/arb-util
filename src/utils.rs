/// stringe function takes in a description function and a Result<T, Error> and it
/// will convert the error to the string if it is one, making some jobs easier and
/// without rust having to tell me I am converting thibgs to std::error::Error.
pub fn stringe<T, K: std::error::Error>(msg: &str, val: Result<T, K>) -> Result<T, String> {
    val.map_err(|e| String::from(msg) + ": " + e.to_string().as_str())
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
                    '*' => "_asterix_",
                    '\'' => "_apostrophe_",
                    '-' => "_hyphen_",
                    '!' => "_exclamation_mark_",
                    '?' => "_question_mark_",
                    '.' => "_period_",
                    ',' => "_comma_",
                    ':' => "_colon_",
                    ';' => "_semicolon_",
                    '(' => "_open_parenthesis_",
                    ')' => "_close_parenthesis_",
                    '[' => "_open_bracket_",
                    ']' => "_close_bracket_",
                    '{' => "_open_brace_",
                    '}' => "_close_brace_",
                    '<' => "_less_than_",
                    '>' => "_greater_than_",
                    '/' => "_forward_slash_",
                    '\\' => "_backslash_",
                    '|' => "_pipe_",
                    '#' => "_hash_",
                    '@' => "_at_",
                    '&' => "_ampersand_",
                    '%' => "_percent_",
                    '$' => "_dollar_",
                    '^' => "_caret_",
                    '+' => "_plus_",
                    '=' => "_equals_",
                    '~' => "_tilde_",
                    '`' => "_backtick_",
                    '"' => "_double_quote_",
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
