# Config CRUD operations

Add full CRUD operations for the config.json file via the gRPC API. Currently only read_config is exposed via get_config endpoint. Need to add update_config endpoint to allow clients to update configuration including priority_levels, custom_fields, defaults, allowed_states, and default_state with proper validation.
