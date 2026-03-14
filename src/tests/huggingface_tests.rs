#[cfg(test)]
mod tests {
    use clouder_core::external::huggingface::HfModel;

    fn make_model(id: &str) -> HfModel {
        HfModel {
            id: id.to_string(),
            author: None,
            downloads: 1000,
            likes: 50,
            pipeline_tag: Some("text-generation".to_string()),
            tags: vec![
                "transformers".to_string(),
                "pytorch".to_string(),
                "en".to_string(),
                "license:apache-2.0".to_string(),
                "text-generation".to_string(),
            ],
            last_modified: Some("2024-03-15T12:00:00.000Z".to_string()),
            card_data: None,
        }
    }

    #[test]
    fn test_short_name_with_author() {
        let m = make_model("mistralai/Mistral-7B");
        assert_eq!(m.short_name(), "Mistral-7B");
    }

    #[test]
    fn test_short_name_without_author() {
        let m = make_model("bert-base-uncased");
        assert_eq!(m.short_name(), "bert-base-uncased");
    }

    #[test]
    fn test_resolved_author_from_field() {
        let mut m = make_model("org/model");
        m.author = Some("org".to_string());
        assert_eq!(m.resolved_author(), Some("org"));
    }

    #[test]
    fn test_resolved_author_from_id() {
        let m = make_model("org/model");
        assert_eq!(m.resolved_author(), Some("org"));
    }

    #[test]
    fn test_resolved_author_none_for_bare_id() {
        let m = make_model("bare-model");
        assert_eq!(m.resolved_author(), None);
    }

    #[test]
    fn test_relevant_tags_filters_noise() {
        let m = make_model("x/y");
        let tags = m.relevant_tags(10);
        assert!(!tags.contains(&"transformers"));
        assert!(!tags.contains(&"pytorch"));
        assert!(!tags.contains(&"license:apache-2.0"));
        assert!(tags.contains(&"en"));
        assert!(tags.contains(&"text-generation"));
    }

    #[test]
    fn test_relevant_tags_respects_limit() {
        let m = make_model("x/y");
        let tags = m.relevant_tags(1);
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn test_deserialize_full() {
        let json = r#"{
            "id": "mistralai/Mistral-7B",
            "author": "mistralai",
            "downloads": 5200000,
            "likes": 8700,
            "pipeline_tag": "text-generation",
            "tags": ["transformers", "en"],
            "lastModified": "2024-03-15T00:00:00.000Z"
        }"#;
        let m: HfModel = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, "mistralai/Mistral-7B");
        assert_eq!(m.downloads, 5_200_000);
        assert_eq!(m.pipeline_tag.as_deref(), Some("text-generation"));
        assert_eq!(m.last_modified.as_deref(), Some("2024-03-15T00:00:00.000Z"));
    }

    #[test]
    fn test_deserialize_minimal() {
        let json = r#"{"id":"some-model"}"#;
        let m: HfModel = serde_json::from_str(json).unwrap();
        assert_eq!(m.downloads, 0);
        assert_eq!(m.likes, 0);
        assert!(m.tags.is_empty());
        assert!(m.pipeline_tag.is_none());
    }
}
