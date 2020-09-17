pub mod error;
pub mod normalizer;

#[cfg(test)]
mod tests {
    use crate::normalizer;

    #[test]
    fn normalize_test() {
        let tainted_url = "https://example.com/main.php?c=1&b=2&a=5";
        let normalizer = normalizer::UrlNormalizer::new(tainted_url).unwrap();
        assert_eq!("https://example.com/main.php?a=5&b=2&c=1", normalizer.normalize(None).unwrap());
    }


    #[test]
    fn normalize_test_remove_param() {
        let tainted_url = "https://example.com:8080/main.php?c=1&b=2&a=5&utm_source=facebook&utm_medium=social&utm_campaign=seofanpage";
        let normalizer = normalizer::UrlNormalizer::new(tainted_url).unwrap();
        assert_eq!("https://example.com:8080/main.php?a=5&b=2&c=1", normalizer.normalize(Some(&["utm_.*"])).unwrap());
    }

    #[test]
    fn normalize_test_remove_dot() {
        let tainted_url = "https://example.com:8080/./main.php?c=1&b=2&a=5";
        let normalizer = normalizer::UrlNormalizer::new(tainted_url).unwrap();
        assert_eq!("https://example.com:8080/main.php?a=5&b=2&c=1", normalizer.normalize(None).unwrap());
    }
}
