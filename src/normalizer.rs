//! url_normalizer
//! Normalize url path and remove parameter from url via regular expression.
//! <br/>
//! Example usage:
//! ```rust
//! use url_normalizer::normalizer;
//!
//! fn main() {
//!     let tainted_url1 = "https://example.com/main.php?c=1&b=2&a=5";
//!     let normalizer1 = normalizer::UrlNormalizer::new(tainted_url1).unwrap();
//!     assert_eq!("https://example.com/main.php?a=5&b=2&c=1", normalizer1.normalize(None).unwrap());
//!
//!     let tainted_url2 = "https://example.com:8080/main.php?c=1&b=2&a=5&utm_source=facebook&utm_medium=social&utm_campaign=seofanpage";
//!     let normalizer2 = normalizer::UrlNormalizer::new(tainted_url2).unwrap();
//!     assert_eq!("https://example.com:8080/main.php?a=5&b=2&c=1", normalizer2.normalize(Some(&["utm_.*"])).unwrap());
//!
//!     let tainted_url3 = "https://example.com:8080/./main.php?c=1&b=2&a=5";
//!     let normalizer3 = normalizer::UrlNormalizer::new(tainted_url3).unwrap();
//!     assert_eq!("https://example.com:8080/main.php?a=5&b=2&c=1", normalizer3.normalize(None).unwrap());
//! }
//! ```

use std::collections::BTreeMap;

use regex::Regex;
use url::Url;
use urlencoding::{decode, encode};

use crate::error::NormalizeError;

pub struct UrlNormalizer {
    url: Url
}

impl UrlNormalizer {
    pub fn new(tainted_url: &str) -> Result<Self, NormalizeError> {
        let url = Url::parse(tainted_url.trim()).map_err(|_| NormalizeError::UrlParseError)?;
        Ok(Self {
            url
        })
    }

    /// Normalizes URL
    pub fn normalize(&self, remove_param_regex: Option<&[&str]>) -> Result<String, NormalizeError> {
        let url = self.normalize_url()?;
        let mut normalized_path = Vec::<u8>::new();
        let urls = url.path().split("/").collect::<Vec<&str>>();
        for (i, u) in urls.iter().enumerate() {
            normalized_path.extend_from_slice(encode(u).as_bytes());
            if i < urls.len() - 1 {
                normalized_path.push(b'/');
            }
        }
        let normalized_path = String::from_utf8(normalized_path).map_err(|_| NormalizeError::UrlEncodeError)?;
        let params = Self::create_parameter_map(url.query(), remove_param_regex)?;
        Ok(Self::to_normalized_url(&url, params, normalized_path))
    }

    fn to_normalized_url(url: &Url, params: BTreeMap<String, String>, normalized_path: String) -> String {
        let host = if let Some(h) = url.host_str() {
            h
        } else {
            ""
        };
        let port = if let Some(p) = url.port() {
            if p == 80 {
                "".to_owned()
            } else {
                format!(":{}", p)
            }
        } else {
            "".to_owned()
        };
        let mut query_string = Vec::new();
        for p in params.iter() {
            query_string.push(format!("{}={}", p.0, p.1));
        }
        let mut query_string_result = query_string.join("&");
        if !query_string.is_empty() {
            query_string_result = format!("?{}", query_string_result);
        }
        format!("{}://{}{}{}{}", url.scheme(), host, port, normalized_path, query_string_result)
    }

    fn split_token(pair: &str, tokens: Vec<String>) -> Option<(String, String)> {
        match tokens.len() {
            1 => {
                let token_0 = &tokens[0];
                if pair.chars().nth(0).unwrap() == '=' {
                    Some(("".to_owned(), token_0.clone()))
                } else {
                    Some((token_0.clone(), "".to_owned()))
                }
            }
            2 => {
                Some((tokens[0].clone(), tokens[1].clone()))
            }
            _ => None
        }
    }

    fn create_parameter_map(query: Option<&str>, remove_param_regex: Option<&[&str]>) -> Result<BTreeMap<String, String>, NormalizeError> {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let query_string = match query {
            Some(q) => {
                q
            }
            None => {
                return Ok(params);
            }
        };
        let mut remove_rules = Vec::new();
        if let Some(remove_param_regex) = remove_param_regex {
            for &r in remove_param_regex {
                let regex = Regex::new(r).map_err(|_| NormalizeError::RegexParseError(r.to_owned()))?;
                remove_rules.push(regex);
            }
        };
        let pairs = query_string.split("&");
        'pair: for pair in pairs {
            if pair.len() < 1 {
                continue;
            }
            let token = pair.splitn(2, "=")
                .map(|t| {
                    decode(t)
                })
                .take_while(|t| t.is_ok())
                .map(|t| t.unwrap()).collect::<Vec<String>>();

            if let Some(token) = Self::split_token(pair, token) {
                for regex in &remove_rules {
                    if regex.is_match(&token.0) {
                        continue 'pair;
                    }
                }
                params.insert(token.0, token.1);
            }
        }
        Ok(params)
    }

    fn normalize_url(&self) -> Result<Url, NormalizeError> {
        let url = &self.url;
        if self.is_opaque() {
            return Ok((*url).clone());
        }
        let np = Self::normalize_path(url.path())?.replace("/$", "");
        if np == url.path() {
            return Ok((*url).clone());
        }
        let mut nu = (*url).clone();
        nu.set_path(&np);
        Ok(nu)
    }

    fn normalize_path(url_path: &str) -> Result<String, NormalizeError> {
        let mut path = url_path.chars().collect::<Vec<char>>();
        let ns = Self::needs_normalization(&path);
        if ns < 0 {
            return Ok(url_path.to_string());
        }
        let mut segs = Self::split(&mut path, ns)?;
        Self::remove_dots(&mut path, &mut segs);
        println!("2-> segs::{:?} path:{:?}", segs, path);
        Self::maybe_add_leading_dot(&mut path, &mut segs);
        println!("3-> segs::{:?} path:{:?}", segs, path);
        let p = Self::join(&mut path, &mut segs)?;
        let res: String = path.into_iter().take(p).collect();
        Ok(res)
    }

    fn is_opaque(&self) -> bool {
        self.url.path().is_empty()
    }

    fn needs_normalization(path: &Vec<char>) -> isize {
        let mut ns = 0;
        let mut p = 0;
        let end = path.len() - 1;
        while p <= end {
            if path[p] != '/' {
                break;
            }
            p += 1;
        }
        let mut normal = if p > 1 {
            false
        } else {
            true
        };
        while p <= end {
            if path[p] == '.'
                && ((p == end)
                || ((path[p + 1] == '/')
                || ((path[p + 1] == '.')
                && ((p + 1 == end)
                || (path[p + 2] == '/'))))) {
                normal = false;
            }
            ns += 1;
            while p <= end {
                if path[p] != '/' {
                    p += 1;
                    continue;
                }
                p += 1;
                while p <= end {
                    if path[p] != '/' {
                        break;
                    }
                    normal = false;
                    p += 1;
                }
                break;
            }
        }
        if normal {
            -1
        } else {
            ns
        }
    }


    fn split(path: &mut Vec<char>, seg_size: isize) -> Result<Vec<isize>, NormalizeError> {
        let mut segs: Vec<isize> = vec![0; seg_size as usize];//Vec::with_capacity(seg_size as usize);
        let end = path.len() as isize - 1;
        let mut p = 0;
        let mut i = 0;
        while p <= end {
            let p_p = &mut path[p as usize];
            if *p_p != '/' {
                break;
            }
            *p_p = '\0';
            p += 1;
        }
        while p <= end {
            segs[i] = p;
            i += 1;
            p += 1;
            while p <= end {
                if path[p as usize] != '/' {
                    p += 1;
                    continue;
                }
                p += 1;
                path[p as usize - 1] = '\0';
                while p <= end {
                    if path[p as usize] != '/' {
                        break;
                    }
                    path[p as usize] = '\0';
                    p += 1;
                }
                break;
            }
        }
        if i != segs.len() {
            Err(NormalizeError::InternalError)
        } else {
            Ok(segs)
        }
    }

    fn remove_dots(path: &mut Vec<char>, segs: &mut Vec<isize>) {
        let ns = segs.len();
        let end = path.len() as isize - 1;
        for mut i in 0..ns {
            let mut dots = 0;
            loop {
                let p = segs[i];
                let p = p as usize;
                if path[p] == '.' {
                    if p == end as usize {
                        dots = 1;
                        break;
                    } else if path[p + 1] == '\0' {
                        dots = 1;
                        break;
                    } else if (path[p + 1] == '.') && ((p + 1 == end as usize) || (path[p + 2] == '\0')) {
                        dots = 2;
                        break;
                    }
                }
                i += 1;
                if i >= ns {
                    break;
                }
            }
            if (i > ns) || (dots == 0) {
                break;
            }
            if dots == 1 {
                segs[i] = -1;
            } else {
                let mut j = 0;
                for k in (0..=i - 1).rev().step_by(1) {
                    if segs[k as usize] != -1 {
                        break;
                    }
                    j = k;
                }
                let j = j as usize;
                let q = segs[j];
                let q = q as usize;
                if !((path[q] == '.')
                    && path[q + 1] == '.'
                    && path[q + 2] == '\0') {
                    segs[i] = -1;
                    segs[j] = -1;
                }
            }
        }
    }

    fn maybe_add_leading_dot(path: &mut Vec<char>, segs: &mut Vec<isize>) {
        if path[0] == '\0' {
            return;
        }

        let ns = segs.len();
        let mut f = 0;
        while f < ns {
            if segs[f] >= 0 {
                break;
            }
            f += 1;
        }
        if (f >= ns) || (f == 0) {
            return;
        }
        let mut p = segs[f] as usize;
        while p < path.len() && (path[p] != ':') && path[p] != '\0' {
            p += 1;
        }
        if p >= path.len() || path[p] == '\0' {
            return;
        }
        path[0] = '.';
        path[1] = '\0';
        segs[0] = 0;
    }

    fn join(path: &mut Vec<char>, segs: &mut Vec<isize>) -> Result<usize, NormalizeError> {
        let ns = segs.len();
        let end = if path.is_empty() {
            0
        } else {
            path.len() as isize - 1
        };

        let mut p = 0;
        let path_p = &mut path[p];
        if *path_p == '\0' {
            *path_p = '/';
            p += 1;
        }

        for i in 0..ns {
            let mut q = segs[i];
            if q == -1 {
                continue;
            }
            if p == q as usize {
                while (p <= end as usize) && (path[p] != '\0') {
                    p += 1;
                }
                if p <= end as usize {
                    path[p] = '/';
                    p += 1;
                }
            } else if p < q as usize {
                while q <= end as isize && path[q as usize] != '\0' {
                    path[p] = path[q as usize];

                    p += 1;
                    q += 1;
                }
                if q <= end as isize {
                    path[p] = '/';
                    p += 1;
                }
            } else {
                return Err(NormalizeError::InternalError);
            }
        }
        Ok(p)
    }
}