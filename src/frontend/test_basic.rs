#[cfg(test)]
mod tests {
    use crate::frontend::{Frontend, FrontendManager, NoOpFrontend};

    #[test]
    fn test_frontend_manager_instantiation() {
        let mut manager = FrontendManager::new();

        // Test registering a frontend
        let frontend = Box::new(NoOpFrontend);
        assert!(manager.register_frontend(frontend).is_ok());

        // Test listing frontends
        let frontends = manager.list_frontends();
        assert_eq!(frontends.len(), 1);
        assert_eq!(frontends[0], "noop");
    }

    #[test]
    fn test_noop_frontend() {
        let mut frontend = NoOpFrontend;
        assert_eq!(frontend.name(), "noop");
        assert!(frontend.initialize().is_ok());
        assert!(frontend.cleanup().is_ok());
    }
}
