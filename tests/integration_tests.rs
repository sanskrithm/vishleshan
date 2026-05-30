/// Comprehensive test suite for PANINI-RS compiler modules
///
/// This module provides integration tests covering:
/// - Lexical analysis (tokenization)
/// - Semantic analysis (graph building)
/// - Graph validation (DAG properties)
/// - Optimization (Paribhasha rewrites)
/// - Execution planning (code generation)
#[cfg(test)]
mod comprehensive_tests {
    use panini_rs::dhatu::*;
    use panini_rs::graph::*;
    use panini_rs::lexer::Lexer;
    use panini_rs::semantic::SemanticAnalyzer;
    use panini_rs::optimizer::QueryOptimizer;
    use panini_rs::planner::{ExecutionPlanner, ExecutionBackend};

    // ========== LEXICAL ANALYSIS TESTS ==========

    #[test]
    fn test_lexer_simple_morpheme() {
        let mut lexer = Lexer::new("sales-at");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() >= 1);
    }

    #[test]
    fn test_lexer_complete_program() {
        let mut lexer = Lexer::new("sales-at revenue-ena chid-tva yuj-tva drsh-ti");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() >= 5); // source, instrument, and at least 3 verbs
    }

    #[test]
    fn test_lexer_multiple_instruments() {
        let mut lexer = Lexer::new("data-at region-ena revenue-ena ci-tva drsh-ti");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() >= 5);
    }

    #[test]
    fn test_lexer_all_aggregation_dhatus() {
        let aggregations = vec!["yuj", "madh", "gan", "lagh", "mah"];
        for agg in aggregations {
            let input = format!("data-at {}-tva drsh-ti", agg);
            let mut lexer = Lexer::new(&input);
            let tokens = lexer.tokenize().unwrap();
            assert!(tokens.len() >= 3, "Failed for aggregation: {}", agg);
        }
    }

    #[test]
    fn test_lexer_all_transformation_dhatus() {
        let transforms = vec!["chid", "adhyaya", "kram", "vibhaj"];
        for trans in transforms {
            let input = format!("data-at {}-tva drsh-ti", trans);
            let mut lexer = Lexer::new(&input);
            let tokens = lexer.tokenize().unwrap();
            assert!(tokens.len() >= 3, "Failed for transformation: {}", trans);
        }
    }

    #[test]
    fn test_lexer_relational_dhatus() {
        let input = "data-at region-ena ci-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() >= 4);
    }

    // ========== SEMANTIC ANALYSIS TESTS ==========

    #[test]
    fn test_semantic_simple_filter_aggregate() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_semantic_group_average() {
        let input = "data-at region-ena ci-tva madh-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_semantic_multiple_instruments() {
        let input = "sales-at region-ena revenue-ena ci-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_semantic_no_source_error() {
        let input = "revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze(&tokens).is_err());
    }

    #[test]
    fn test_semantic_no_terminal_error() {
        let input = "sales-at revenue-ena chid-tva yuj-tva";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze(&tokens).is_err());
    }

    #[test]
    fn test_semantic_count_aggregation() {
        let input = "orders-at status-ena gan-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_semantic_all_aggregations() {
        let aggregations = vec![
            ("yuj-tva", "sum"),
            ("madh-tva", "average"),
            ("gan-tva", "count"),
            ("lagh-tva", "min"),
            ("mah-tva", "max"),
        ];

        for (dhatu, _name) in aggregations {
            let input = format!("data-at column-ena {}-tva drsh-ti", dhatu);
            let mut lexer = Lexer::new(&input);
            let tokens = lexer.tokenize().unwrap();

            let mut analyzer = SemanticAnalyzer::new();
            let graph = analyzer.analyze(&tokens).unwrap();

            assert!(graph.validate().is_ok(), "Failed for {}", dhatu);
        }
    }

    // ========== GRAPH ANALYSIS TESTS ==========

    #[test]
    fn test_graph_topological_order() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        let topo = graph.topological_order();
        assert!(topo.len() > 0);
    }

    #[test]
    fn test_graph_is_dag() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        assert!(graph.is_acyclic());
    }

    #[test]
    fn test_graph_depth() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        let depth = graph.depth();
        assert!(depth > 0);
    }

    #[test]
    fn test_graph_stats() {
        let input = "sales-at revenue-ena chid-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        let stats = graph.stats();
        assert!(stats.node_count > 0);
        assert!(stats.is_dag);
    }

    // ========== OPTIMIZATION TESTS ==========

    #[test]
    fn test_optimizer_creation() {
        let _optimizer = QueryOptimizer::new();
    }

    #[test]
    fn test_optimizer_empty_graph() {
        let mut graph = SemanticGraph::new();
        let optimizer = QueryOptimizer::new();

        // Should fail on invalid graph
        assert!(optimizer.optimize(&mut graph).is_err());
    }

    // ========== EXECUTION PLANNING TESTS ==========

    #[test]
    fn test_planner_creation_polars() {
        let _planner = ExecutionPlanner::new(ExecutionBackend::Polars);
    }

    #[test]
    fn test_planner_all_backends() {
        let backends = vec![
            ExecutionBackend::Polars,
            ExecutionBackend::Arrow,
            ExecutionBackend::DuckDB,
            ExecutionBackend::DataFusion,
        ];

        for backend in backends {
            let _planner = ExecutionPlanner::new(backend);
        }
    }

    #[test]
    fn test_planner_invalid_graph() {
        let mut graph = SemanticGraph::new();
        let planner = ExecutionPlanner::default();

        // Should fail on invalid graph (no entry/exit)
        assert!(planner.plan(&graph).is_err());
    }

    // ========== END-TO-END INTEGRATION TESTS ==========

    #[test]
    fn test_end_to_end_simple() {
        // Lexer
        let mut lexer = Lexer::new("sales-at revenue-ena chid-tva yuj-tva drsh-ti");
        let tokens = lexer.tokenize().unwrap();

        // Semantic analyzer
        let mut analyzer = SemanticAnalyzer::new();
        let mut graph = analyzer.analyze(&tokens).unwrap();

        // Validation
        assert!(graph.validate().is_ok());

        // Optimization
        let optimizer = QueryOptimizer::new();
        let _ = optimizer.optimize(&mut graph);

        // Planning
        let planner = ExecutionPlanner::default();
        let plan = planner.plan(&graph).unwrap();

        assert!(plan.steps.len() > 0);
    }

    #[test]
    fn test_end_to_end_complex() {
        let input = "sales-at region-ena revenue-ena ci-tva yuj-tva kram-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let mut graph = analyzer.analyze(&tokens).unwrap();

        assert!(graph.validate().is_ok());

        let optimizer = QueryOptimizer::new();
        let _ = optimizer.optimize(&mut graph);

        let planner = ExecutionPlanner::default();
        let plan = planner.plan(&graph).unwrap();

        assert!(plan.steps.len() > 0);
    }

    #[test]
    fn test_anuvrtti_propagation() {
        let input = "sales-at region-ena revenue-ena ci-tva yuj-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        // Verify graph was built (Anuvrtti propagation happens internally)
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_karaka_relationships() {
        let input = "data-at column-ena ci-tva drsh-ti";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let graph = analyzer.analyze(&tokens).unwrap();

        // Verify Karaka edges are present
        let stats = graph.stats();
        assert!(stats.edge_count > 0, "Graph should have Karaka edges");
    }
}
