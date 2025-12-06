# Add favorite projects support to project registry

Add is_favorite field to ProjectInfo protobuf message. Implement storage of favorite status in the project registry. Add RPC method SetProjectFavorite(path, is_favorite). Persist favorites across daemon restarts.
