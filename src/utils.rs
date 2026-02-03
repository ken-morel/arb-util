/// stringe function takes in a description function and a Result<T, Error> and it
/// will convert the error to the string if it is one, making some jobs easier and
/// without rust having to tell me I am converting thibgs to std::error::Error.
pub fn stringe<T, K: std::error::Error>(msg: &str, val: Result<T, K>) -> Result<T, String> {
    match val {
        Ok(v) => Ok(v),
        Err(e) => Err(String::from(msg) + ": " + e.to_string().as_str()),
    }
}
