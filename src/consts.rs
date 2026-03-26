// OTP codes configuration
pub const CONFIRMATION_CODE_LENGTH: usize = 6;
pub const CONFIRMATION_CODE_CHAR_POOL: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];

// Defaults for email contents 
pub const DEFAULT_REGISTRATION_EMAIL_SUBJECT: &str = "Verify your account";
pub const DEFAULT_REGISTRATION_EMAIL_TEXT_BODY: &str = "Verify your account using the code: %code%.";
pub const DEFAULT_REGISTRATION_EMAIL_HTML_BODY: &str = "Verify your account using the code <b>%code%</b>";

pub const DEFAULT_LOGIN_EMAIL_SUBJECT: &str = "Confirm login to your account";
pub const DEFAULT_LOGIN_EMAIL_TEXT_BODY: &str = "Confirm login to your account using the code: %code%.";
pub const DEFAULT_LOGIN_EMAIL_HTML_BODY: &str = "Confirm login to your account using the code <b>%code%</b>";

pub const DEFAULT_USER_PASSWORD_UPDATE_EMAIL_SUBJECT: &str = "Confirm password update";
pub const DEFAULT_USER_PASSWORD_UPDATE_EMAIL_TEXT_BODY: &str = "Confirm password update on your account using the following confirmation code: %code%.";
pub const DEFAULT_USER_PASSWORD_UPDATE_EMAIL_HTML_BODY: &str = "Confirm password update on your account using the following confirmation code <b>%code%</b>.";