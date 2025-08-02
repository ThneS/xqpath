#[cfg(test)]
mod debug_tests {

    #[test]
    fn test_basic_query() {
        let data = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#;
        let result = crate::query!(data, ".users[*].name").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], serde_json::json!("Alice"));
        assert_eq!(result[1], serde_json::json!("Bob"));
    }

    #[cfg(feature = "debug")]
    #[test]
    fn test_debug_functionality() {
        use crate::debug::{DebugContext, LogLevel};

        let ctx = DebugContext::new()
            .with_timing(true)
            .with_memory_tracking(true)
            .with_log_level(LogLevel::Debug);

        assert!(ctx.get_config().timing_enabled);
        assert!(ctx.get_config().memory_tracking);
        assert_eq!(ctx.get_config().log_level, LogLevel::Debug);
    }
    #[cfg(feature = "debug")]
    #[test]
    fn test_trace_query_macro() {
        let data = r#"{"users": [{"name": "Alice"}]}"#;
        let result = crate::trace_query!(data, ".users[*].name");

        assert!(result.is_ok());
        let (values, stats) = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], serde_json::json!("Alice"));
        assert!(stats.duration.as_nanos() > 0);
    }

    #[cfg(feature = "debug")]
    #[test]
    fn test_query_debug_macro() {
        let data = r#"{"user": {"name": "Charlie"}}"#;
        let mut debug_called = false;

        let result = crate::query_debug!(
            data,
            ".user.name",
            |debug_info: &crate::debug::DebugInfo| {
                debug_called = true;
                assert!(debug_info.parse_duration.is_some());
                assert!(debug_info.execution_duration.is_some());
                assert_eq!(debug_info.execution_path, "query(.user.name)");
                assert_eq!(debug_info.queries_executed, 1);
            }
        );

        assert!(result.is_ok());
        assert!(debug_called);
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], serde_json::json!("Charlie"));
    }
}
