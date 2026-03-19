#[cfg(test)]
mod tests {
    use clouder_core::external::github::{GhRepo, GhUser};

    fn make_user() -> GhUser {
        GhUser {
            login: "octocat".to_string(),
            name: Some("The Octocat".to_string()),
            bio: Some("GitHub mascot".to_string()),
            avatar_url: "https://avatars.githubusercontent.com/u/583231".to_string(),
            html_url: "https://github.com/octocat".to_string(),
            public_repos: 8,
            followers: 15000,
            following: 9,
            location: Some("San Francisco, CA".to_string()),
            blog: Some("https://github.blog".to_string()),
            company: Some("@github".to_string()),
        }
    }

    fn make_repo() -> GhRepo {
        GhRepo {
            full_name: "octocat/Hello-World".to_string(),
            description: Some("My first repository".to_string()),
            html_url: "https://github.com/octocat/Hello-World".to_string(),
            stargazers_count: 2000,
            forks_count: 1200,
            open_issues_count: 5,
            language: Some("Python".to_string()),
            pushed_at: Some("2024-03-15T12:00:00Z".to_string()),
            topics: vec!["demo".to_string(), "example".to_string()],
            license: Some(clouder_core::external::github::GhLicense {
                name: "MIT License".to_string(),
            }),
            owner: clouder_core::external::github::GhOwner {
                avatar_url: "https://avatars.githubusercontent.com/u/583231".to_string(),
            },
        }
    }

    #[test]
    fn test_user_display_name_prefers_name() {
        let u = make_user();
        assert_eq!(u.display_name(), "The Octocat");
    }

    #[test]
    fn test_user_display_name_falls_back_to_login() {
        let mut u = make_user();
        u.name = None;
        assert_eq!(u.display_name(), "octocat");
    }

    #[test]
    fn test_repo_pushed_date_truncated() {
        let r = make_repo();
        assert_eq!(r.pushed_date(), Some("2024-03-15"));
    }

    #[test]
    fn test_repo_pushed_date_none() {
        let mut r = make_repo();
        r.pushed_at = None;
        assert_eq!(r.pushed_date(), None);
    }

    #[test]
    fn test_deserialize_user_minimal() {
        let json = r#"{
            "login": "octocat",
            "avatar_url": "https://example.com/avatar",
            "html_url": "https://github.com/octocat",
            "public_repos": 5,
            "followers": 100,
            "following": 50
        }"#;
        let u: GhUser = serde_json::from_str(json).unwrap();
        assert_eq!(u.login, "octocat");
        assert!(u.name.is_none());
        assert!(u.bio.is_none());
    }

    #[test]
    fn test_deserialize_repo_minimal() {
        let json = r#"{
            "full_name": "octocat/hello",
            "html_url": "https://github.com/octocat/hello",
            "stargazers_count": 10,
            "forks_count": 2,
            "open_issues_count": 0,
            "owner": { "avatar_url": "https://example.com/avatar" }
        }"#;
        let r: GhRepo = serde_json::from_str(json).unwrap();
        assert_eq!(r.full_name, "octocat/hello");
        assert!(r.language.is_none());
        assert!(r.license.is_none());
    }

    #[test]
    fn test_deserialize_repo_with_license() {
        let json = r#"{
            "full_name": "octocat/hello",
            "html_url": "https://github.com/octocat/hello",
            "stargazers_count": 0,
            "forks_count": 0,
            "open_issues_count": 0,
            "license": { "key": "mit", "name": "MIT License" },
            "owner": { "avatar_url": "https://example.com/avatar" }
        }"#;
        let r: GhRepo = serde_json::from_str(json).unwrap();
        assert_eq!(r.license.unwrap().name, "MIT License");
    }
}
