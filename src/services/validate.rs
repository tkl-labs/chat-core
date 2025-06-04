use regex::Regex;

const LOWERCASE_REGEX: &str = "[a-z]";
const UPPERCASE_REGEX: &str = "[A-Z]";
const NUMERIC_REGEX: &str = "[0-9]";
const SPECIAL_REGEX: &str = "[^a-zA-Z0-9]";
const EMAIL_REGEX: &str = r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$";
const PHONE_NUMBER_REGEX: &str = r"^\+?[0-9]{7,15}$";

pub fn validate_username(username: String) -> bool {
    let valid_username = (username.len() >= 8 && username.len() <= 16)
        && (username.chars().all(char::is_alphanumeric));

    valid_username
}

pub fn validate_email(email: String) -> bool {
    let email_re = Regex::new(EMAIL_REGEX).unwrap();
    let valid_email = email_re.is_match(&email);

    valid_email
}

pub fn validate_phone_number(phone_number: String) -> bool {
    let phone_re = Regex::new(PHONE_NUMBER_REGEX).unwrap();
    let valid_phone_number = phone_re.is_match(&phone_number);

    valid_phone_number
}

pub fn validate_password(password: String) -> bool {
    // TODO: block emoji from password
    let lower_re = Regex::new(LOWERCASE_REGEX).unwrap();
    let upper_re = Regex::new(UPPERCASE_REGEX).unwrap();
    let num_re = Regex::new(NUMERIC_REGEX).unwrap();
    let special_re = Regex::new(SPECIAL_REGEX).unwrap();
    let valid_password = (password.len() >= 12 && password.len() <= 64)
        && (lower_re.is_match(&password))
        && (upper_re.is_match(&password))
        && (num_re.is_match(&password))
        && (special_re.is_match(&password));

    valid_password
}