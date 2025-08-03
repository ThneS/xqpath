use xqpath::debugger::*;

#[cfg(test)]
mod enhanced_debugger_tests {
    use super::*;

    #[test]
    fn test_enhanced_debugger_creation() {
        let _debugger = XQPathDebugger::new();
        // 调试器创建应该成功
        assert!(true); // 基础检查：如果代码能运行到这里，创建就是成功的
    }

    #[test]
    fn test_command_parsing_enhanced() {
        // 测试基础命令
        assert!(matches!(
            DebugCommand::parse(":help"),
            Ok(DebugCommand::Help)
        ));
        assert!(matches!(DebugCommand::parse(":h"), Ok(DebugCommand::Help)));
        assert!(matches!(
            DebugCommand::parse(":quit"),
            Ok(DebugCommand::Quit)
        ));
        assert!(matches!(DebugCommand::parse(":q"), Ok(DebugCommand::Quit)));

        // 测试文件操作命令
        match DebugCommand::parse(":load test.json") {
            Ok(DebugCommand::Load { file }) => {
                assert_eq!(file.to_str().unwrap(), "test.json");
            }
            _ => panic!("Expected Load command"),
        }

        // 测试断点命令
        match DebugCommand::parse(":bp .users[0].name") {
            Ok(DebugCommand::SetBreakpoint { path, condition }) => {
                assert_eq!(path, ".users[0].name");
                assert!(condition.is_none());
            }
            _ => panic!("Expected SetBreakpoint command"),
        }

        // 测试监视点命令
        match DebugCommand::parse(":watch .users[*].age") {
            Ok(DebugCommand::SetWatchPoint {
                expression,
                condition,
            }) => {
                assert_eq!(expression, ".users[*].age");
                assert!(condition.is_none());
            }
            _ => panic!("Expected SetWatchPoint command"),
        }

        // 测试简短命令
        assert!(matches!(
            DebugCommand::parse(":vars"),
            Ok(DebugCommand::ListVariables)
        ));
        assert!(matches!(
            DebugCommand::parse(":v"),
            Ok(DebugCommand::ListVariables)
        ));
        assert!(matches!(
            DebugCommand::parse(":reset"),
            Ok(DebugCommand::Reset)
        ));

        // 测试直接查询
        match DebugCommand::parse(".users[*].name") {
            Ok(DebugCommand::Run { query }) => {
                assert_eq!(query, ".users[*].name");
            }
            _ => panic!("Expected Run command for direct query"),
        }
    }

    #[test]
    fn test_debugger_session_management() {
        let session = DebugSession::new();
        assert!(session.breakpoints.is_empty());
        assert!(session.watch_points.is_empty());
        assert!(session.current_data.is_none());
        assert!(matches!(session.execution_state, ExecutionState::Stopped));
    }

    #[test]
    fn test_query_evaluator() {
        let evaluator = QueryEvaluator::new();
        assert!(evaluator.current_query.is_none());
        assert!(evaluator.last_result.is_none());
    }

    #[test]
    fn test_data_inspector() {
        let inspector = DataInspector::default();
        assert!(inspector.inspect_target.is_none());
        assert!(inspector.inspect_path.is_none());
        assert!(inspector.type_info.is_none());
    }
}
