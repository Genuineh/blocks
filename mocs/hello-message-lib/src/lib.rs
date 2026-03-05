pub fn hello_message() -> &'static str {
    "hello world"
}

#[cfg(test)]
mod tests {
    use super::hello_message;

    #[test]
    fn returns_expected_message() {
        assert_eq!(hello_message(), "hello world");
    }
}
