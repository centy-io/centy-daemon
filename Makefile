.PHONY: generate generate-mcp

generate: generate-mcp

generate-mcp:
	buf generate --template mcp/buf.gen.yaml
