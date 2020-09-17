# url_normalizer 

url_normalizer
Normalize url path and remove parameter from url via regular expression.
<br/>
Example usage:
```rust
use url_normalizer::normalizer;

fn main() {
    let tainted_url1 = "https://example.com/main.php?c=1&b=2&a=5";
    let normalizer1 = normalizer::UrlNormalizer::new(tainted_url1).unwrap();
    assert_eq!("https://example.com/main.php?a=5&b=2&c=1", normalizer1.normalize(None).unwrap());

    let tainted_url2 = "https://example.com:8080/main.php?c=1&b=2&a=5&utm_source=facebook&utm_medium=social&utm_campaign=seofanpage";
    let normalizer2 = normalizer::UrlNormalizer::new(tainted_url2).unwrap();
    assert_eq!("https://example.com:8080/main.php?a=5&b=2&c=1", normalizer2.normalize(Some(&["utm_.*"])).unwrap());

    let tainted_url3 = "https://example.com:8080/./main.php?c=1&b=2&a=5";
    let normalizer3 = normalizer::UrlNormalizer::new(tainted_url3).unwrap();
    assert_eq!("https://example.com:8080/main.php?a=5&b=2&c=1", normalizer3.normalize(None).unwrap());
}
```
 
