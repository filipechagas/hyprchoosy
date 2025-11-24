//! URL and client matching logic.
//!
//! This module provides functions to match URLs and clients against
//! configured rules to determine which browser should be used.

use crate::config::RuleSection;
use anyhow::{Context, Result};
use std::collections::HashMap;
use url::Url;

#[cfg(feature = "debug")]
use log::{debug, info};

pub fn parse_url_host(u: &str) -> Result<String> {
    let parsed = if u.contains("://") {
        Url::parse(u)
    } else {
        Url::parse(&format!("http://{}", u))
    }
    .with_context(|| format!("Invalid URL: {}", u))?;

    Ok(parsed
        .host_str()
        .map(|s| s.to_lowercase())
        .unwrap_or_default())
}

pub fn match_client<'a>(
    client: &str,
    sections: &'a HashMap<String, RuleSection>,
) -> Option<&'a RuleSection> {
    #[cfg(feature = "debug")]
    debug!("Matching client: '{}'", client);

    let c = client.to_lowercase();

    #[cfg(feature = "debug")]
    debug!(
        "Available sections: {:?}",
        sections.keys().collect::<Vec<_>>()
    );

    for (_section_name, sec) in sections.iter() {
        #[cfg(feature = "debug")]
        debug!(
            "  Checking section '{}' with clients: {:?}",
            _section_name, sec.clients
        );

        for needle in &sec.clients {
            let needle_lower = needle.to_lowercase();
            if c.contains(&needle_lower) {
                #[cfg(feature = "debug")]
                info!(
                    "Client '{}' matched rule '{}' (pattern: '{}')",
                    client, _section_name, needle
                );
                return Some(sec);
            }
        }
    }

    #[cfg(feature = "debug")]
    debug!("No client match found for '{}'", client);
    None
}

pub fn match_host<'a>(
    host: &str,
    sections: &'a HashMap<String, RuleSection>,
) -> Option<&'a RuleSection> {
    #[cfg(feature = "debug")]
    debug!("Matching host: '{}'", host);

    let h = host.to_lowercase();

    for (_section_name, sec) in sections.iter() {
        #[cfg(feature = "debug")]
        debug!(
            "  Checking section '{}' with URL patterns: {:?}",
            _section_name, sec.url
        );

        for pat in &sec.url {
            let p = pat.to_lowercase();
            let matches = h == p || h.ends_with(&format!(".{}", p));

            #[cfg(feature = "debug")]
            debug!(
                "    Pattern '{}' {} match host '{}'",
                pat,
                if matches { "DOES" } else { "does NOT" },
                host
            );

            if matches {
                #[cfg(feature = "debug")]
                info!(
                    "Host '{}' matched rule '{}' (pattern: '{}')",
                    host, _section_name, pat
                );
                return Some(sec);
            }
        }
    }

    #[cfg(feature = "debug")]
    debug!("No host match found for '{}'", host);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_host_with_scheme() {
        assert_eq!(parse_url_host("https://github.com").unwrap(), "github.com");
        assert_eq!(parse_url_host("http://example.com").unwrap(), "example.com");
        assert_eq!(
            parse_url_host("https://sub.example.com").unwrap(),
            "sub.example.com"
        );
    }

    #[test]
    fn test_parse_url_host_without_scheme() {
        assert_eq!(parse_url_host("github.com").unwrap(), "github.com");
        assert_eq!(parse_url_host("example.com").unwrap(), "example.com");
        assert_eq!(
            parse_url_host("sub.example.com").unwrap(),
            "sub.example.com"
        );
    }

    #[test]
    fn test_parse_url_host_with_path() {
        assert_eq!(
            parse_url_host("https://github.com/user/repo").unwrap(),
            "github.com"
        );
        assert_eq!(parse_url_host("example.com/path").unwrap(), "example.com");
    }

    #[test]
    fn test_parse_url_host_case_insensitive() {
        assert_eq!(parse_url_host("HTTPS://GITHUB.COM").unwrap(), "github.com");
        assert_eq!(parse_url_host("GitHub.COM").unwrap(), "github.com");
    }

    #[test]
    fn test_parse_url_host_invalid() {
        assert!(parse_url_host("not a url").is_err());
        assert!(parse_url_host("").is_err());
    }

    #[test]
    fn test_match_client_exact() {
        let mut sections = HashMap::new();
        sections.insert(
            "work".to_string(),
            RuleSection {
                browser: "chrome".to_string(),
                clients: vec!["slack".to_string()],
                url: vec![],
            },
        );

        let result = match_client("slack", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "chrome");
    }

    #[test]
    fn test_match_client_partial() {
        let mut sections = HashMap::new();
        sections.insert(
            "work".to_string(),
            RuleSection {
                browser: "chrome".to_string(),
                clients: vec!["slack".to_string()],
                url: vec![],
            },
        );

        let result = match_client("slack-desktop", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "chrome");
    }

    #[test]
    fn test_match_client_case_insensitive() {
        let mut sections = HashMap::new();
        sections.insert(
            "work".to_string(),
            RuleSection {
                browser: "chrome".to_string(),
                clients: vec!["Slack".to_string()],
                url: vec![],
            },
        );

        let result = match_client("SLACK", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "chrome");
    }

    #[test]
    fn test_match_client_no_match() {
        let mut sections = HashMap::new();
        sections.insert(
            "work".to_string(),
            RuleSection {
                browser: "chrome".to_string(),
                clients: vec!["slack".to_string()],
                url: vec![],
            },
        );

        let result = match_client("discord", &sections);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_host_exact() {
        let mut sections = HashMap::new();
        sections.insert(
            "dev".to_string(),
            RuleSection {
                browser: "firefox".to_string(),
                clients: vec![],
                url: vec!["github.com".to_string()],
            },
        );

        let result = match_host("github.com", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "firefox");
    }

    #[test]
    fn test_match_host_subdomain() {
        let mut sections = HashMap::new();
        sections.insert(
            "dev".to_string(),
            RuleSection {
                browser: "firefox".to_string(),
                clients: vec![],
                url: vec!["github.com".to_string()],
            },
        );

        let result = match_host("api.github.com", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "firefox");

        let result = match_host("gist.github.com", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "firefox");
    }

    #[test]
    fn test_match_host_case_insensitive() {
        let mut sections = HashMap::new();
        sections.insert(
            "dev".to_string(),
            RuleSection {
                browser: "firefox".to_string(),
                clients: vec![],
                url: vec!["GitHub.COM".to_string()],
            },
        );

        let result = match_host("github.com", &sections);
        assert!(result.is_some());
        assert_eq!(result.unwrap().browser, "firefox");
    }

    #[test]
    fn test_match_host_no_match() {
        let mut sections = HashMap::new();
        sections.insert(
            "dev".to_string(),
            RuleSection {
                browser: "firefox".to_string(),
                clients: vec![],
                url: vec!["github.com".to_string()],
            },
        );

        let result = match_host("gitlab.com", &sections);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_host_no_partial_match() {
        let mut sections = HashMap::new();
        sections.insert(
            "dev".to_string(),
            RuleSection {
                browser: "firefox".to_string(),
                clients: vec![],
                url: vec!["github.com".to_string()],
            },
        );

        let result = match_host("notgithub.com", &sections);
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_sections() {
        let mut sections = HashMap::new();
        sections.insert(
            "work".to_string(),
            RuleSection {
                browser: "chrome".to_string(),
                clients: vec!["slack".to_string()],
                url: vec![],
            },
        );
        sections.insert(
            "personal".to_string(),
            RuleSection {
                browser: "firefox".to_string(),
                clients: vec!["discord".to_string()],
                url: vec![],
            },
        );

        assert!(match_client("slack", &sections).is_some());
        assert!(match_client("discord", &sections).is_some());
        assert_eq!(match_client("slack", &sections).unwrap().browser, "chrome");
        assert_eq!(
            match_client("discord", &sections).unwrap().browser,
            "firefox"
        );
    }
}
