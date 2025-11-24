use hyprchoosy::{match_client, match_host, parse_url_host, Config, RuleSection};
use std::collections::HashMap;

#[test]
fn test_url_host_with_scheme() {
    assert_eq!(parse_url_host("https://github.com").unwrap(), "github.com");
    assert_eq!(parse_url_host("http://example.com").unwrap(), "example.com");
    assert_eq!(
        parse_url_host("https://sub.example.com").unwrap(),
        "sub.example.com"
    );
}

#[test]
fn test_url_host_without_scheme() {
    assert_eq!(parse_url_host("github.com").unwrap(), "github.com");
    assert_eq!(parse_url_host("example.com").unwrap(), "example.com");
    assert_eq!(
        parse_url_host("sub.example.com").unwrap(),
        "sub.example.com"
    );
}

#[test]
fn test_url_host_with_path() {
    assert_eq!(
        parse_url_host("https://github.com/user/repo").unwrap(),
        "github.com"
    );
    assert_eq!(parse_url_host("example.com/path").unwrap(), "example.com");
}

#[test]
fn test_url_host_case_insensitive() {
    assert_eq!(
        parse_url_host("HTTPS://GITHUB.COM").unwrap(),
        "github.com"
    );
    assert_eq!(parse_url_host("GitHub.COM").unwrap(), "github.com");
}

#[test]
fn test_url_host_invalid() {
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
fn test_config_deserialization() {
    let toml_str = r#"
[default]
browser = "firefox"

[work]
browser = "chrome"
clients = ["slack", "teams"]
url = ["company.com"]
"#;

    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.default.browser, "firefox");
    assert_eq!(config.sections.len(), 1);

    let work = config.sections.get("work").unwrap();
    assert_eq!(work.browser, "chrome");
    assert_eq!(work.clients.len(), 2);
    assert_eq!(work.url.len(), 1);
}

#[test]
fn test_config_default_browser() {
    let toml_str = r#"
[default]

[work]
browser = "chrome"
"#;

    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.default.browser, "firefox");
}

#[test]
fn test_config_no_default_section() {
    let toml_str = r#"
[work]
browser = "chrome"
"#;

    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.default.browser, "");
}

#[test]
fn test_config_empty_lists() {
    let toml_str = r#"
[section]
browser = "chrome"
"#;

    let config: Config = toml::from_str(toml_str).unwrap();
    let section = config.sections.get("section").unwrap();
    assert_eq!(section.clients.len(), 0);
    assert_eq!(section.url.len(), 0);
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
