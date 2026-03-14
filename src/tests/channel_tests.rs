#[cfg(test)]
mod tests {
    #[test]
    fn test_clone_name_suffix() {
        let name = "general";
        assert_eq!(format!("{}-copy", name), "general-copy");
    }

    #[test]
    fn test_confirmation_ids_are_unique() {
        let id_a = format!("confirm_{}", 1234u64);
        let id_b = format!("confirm_{}", 5678u64);
        assert_ne!(id_a, id_b);
    }

    #[test]
    fn test_manage_channels_permission() {
        use serenity::all::Permissions;
        let required = Permissions::MANAGE_CHANNELS;
        let admin = Permissions::ADMINISTRATOR;
        assert!(clouder_core::utils::has_permission(admin, required));
        assert!(clouder_core::utils::has_permission(required, required));
        assert!(!clouder_core::utils::has_permission(
            Permissions::SEND_MESSAGES,
            required
        ));
    }
}
