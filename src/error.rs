pub type AnyError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, AnyError>;

pub fn notify_maintainers_on_error(error: &AnyError) {
    eprintln!("Error occurred: {error}");
}