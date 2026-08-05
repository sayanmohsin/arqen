# Test Plan

Module composition test coverage across 47 tests in 7 areas.

## Module graph validation (module.rs)

- test_validate_ok: valid graph passes
- test_validate_duplicate_module: duplicate names rejected
- test_validate_missing_dependency: missing deps rejected
- test_validate_dependency_cycle: cycles rejected (2-node and 3-node)
- test_topological_order_linear: linear dependency order correct
- test_topological_order_diamond: diamond dependency order correct
- test_detect_cycle_three_nodes: 3-node cycle detected
- test_get_module: module lookup by name
- test_module_graph_error_display: error messages are descriptive

## Module lifecycle (module.rs)

- test_module_init_shutdown: init/shutdown called without error
- test_shutdown_reverse_order: shutdown runs in reverse dependency order
- test_module_health_check: health checks return correct results
- test_modules_with_dependencies: dependency listing works

## Module registration (module.rs)

- test_module_register_tools: tools registered via ModuleContext
- test_module_register_health_checks: health checks auto-registered
- test_register_all_respects_dependency_order: registration order matches topology
- test_register_all_validation_error: missing dep caught at registration
- test_module_health_conversion: ModuleHealth → HealthStatus conversion

## ModuleBuilder (module.rs)

- test_module_builder_new: empty builder
- test_module_builder_register: registration works
- test_module_builder_all_routes: route collection
- test_module_builder_mixed: mixed module types

## ArqenApp (app.rs)

- test_arqen_app_builder_no_modules: empty app builds
- test_arqen_app_builder_with_module: single module
- test_arqen_app_builder_with_config: custom config
- test_arqen_app_builder_with_explicit_state: escape hatch
- test_arqen_app_builder_validation_error: invalid graph rejected
- test_arqen_app_module_builder_accessor: accessor works

## AppStateBuilder with modules (state.rs)

- test_app_state_builder_with_modules: tools and health registered
- test_app_state_builder_with_modules_validation_error: error propagated

## HTTP module composition (http/module.rs)

- test_http_module_router: router creation
- test_merge_module_routes_empty_list: empty list returns base
- test_merge_module_routes_single: single module merge
- test_merge_module_routes_multiple: multiple modules merge
- test_merge_module_routes_with_base_routes: merge with existing routes
