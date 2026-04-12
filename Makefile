.PHONY: build build-daemon build-cli build-mcp test coverage

IGNORE_REGEX ?= server/handlers|server/trait_impl|server/startup|server/resolve|server/action_builders|server/actions|server/config_to_proto|server/convert_infra|server/proto_to_config|server/validate_config|app\.rs|cors\.rs|run/core|run/mod|logging/init|logging/mod|workspace/create|workspace/data|workspace/standalone|user/git|user/sync|common/org_sync|common/git/branch|common/git/git_remote|cleanup/mod|main\.rs|server/mod\.rs|crud/move_io|crud/move_issue|crud/org_sync|org_registry/mod|assets/copy|hooks/runner/post_hooks|crud/get_matchers|crud/migrate|hooks/config/pattern|hook_pattern_segment_matching|item/core/metadata|issue/assets/get\.rs|issue/assets/list\.rs|issue/assets/list_shared|create/helpers\.rs|crud/list\.rs|crud/read\.rs|crud/update_helpers|link/crud_fns/create\.rs|link/crud_fns/update\.rs|link/storage/io|managed_files_catalog|registry/ignore\.rs|organizations/assignment\.rs|org_issues/config\.rs|org_issues/crud_list\.rs|org_issues/paths|organizations/sync\.rs|registry/storage\.rs|tracking/enrich\.rs|tracking/ops\.rs|server/structured_error|user/crud/update\.rs|user_config/loader\.rs|utils/display_and_temp_dir|utils/mod\.rs|item/entities/issue/assets/helpers\.rs|item/entities/issue/create|item/entities/issue/crud/get\.rs|item/entities/issue/crud/parse\.rs|item/entities/issue/crud/update|item/entities/issue/planning\.rs|item/entities/issue/reconcile/|item/entities/issue/status\.rs|item/generic/storage/create_and_get\.rs|item/generic/storage/crud_ops\.rs|item/generic/storage/crud_search\.rs|item/generic/storage/deletion_constraints\.rs|item/generic/storage/helpers\.rs|item/generic/storage/move_item\.rs|item/generic/storage/move_ops\.rs|item/generic/storage/priority_validation\.rs|link/crud_fns/delete|link/crud_read\.rs|manifest/mod\.rs|metrics\.rs|reconciliation/execute|reconciliation/managed_file_template_struct\.rs|reconciliation/managed_files_merge\.rs|reconciliation/plan/|registry/inference/|registry/migrations\.rs|registry/organizations/create\.rs|registry/organizations/delete\.rs|registry/organizations/org_file\.rs|registry/organizations/org_issues/crud|registry/organizations/query\.rs|registry/organizations/update\.rs|registry/tracking/counts\.rs|registry/tracking/enrich_fn\.rs|registry/tracking/enrich_lookups\.rs|registry/tracking/set_ops\.rs|registry/validation\.rs|template/engine\.rs|user/crud/delete\.rs|user/storage\.rs|user/user_serialization\.rs|utils/path_utilities\.rs|cleanup/parse\.rs|cleanup/project\.rs|common/remote\.rs|config/io\.rs|config/item_type_config/io|config/item_type_config/migrate\.rs|config/item_type_config/registry\.rs|config/migrate\.rs|config/read_config_normalization\.rs|hooks/executor\.rs|hooks/hook_phase_and_operation\.rs|hooks/runner/pre_hooks\.rs|item/core/error/impls\.rs|item/entities/issue/assets/add\.rs|item/entities/issue/assets/delete\.rs|registry/org_repo\.rs

build: build-daemon build-cli build-mcp

build-daemon:
	cargo build --release

build-cli:
	$(MAKE) -C cli build-cli

build-mcp:
	$(MAKE) -C mcp build-mcp

test:
	cargo test --all-targets

coverage:
	cargo llvm-cov --all-targets --ignore-filename-regex "$(IGNORE_REGEX)" --fail-under-lines 100
