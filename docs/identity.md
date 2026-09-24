# Stable v2 identifiers

`stable_id` hashes each input part with an eight-byte big-endian length prefix, then the bytes. The first 24 hexadecimal SHA-256 digits are prefixed by the domain name. Length framing keeps different part lists distinct.

The scanner runs `git remote get-url origin` at the scan root. It normalizes URL schemes, SSH `git@` syntax and a trailing `.git`. If origin is unavailable, `[repository] name` is used. Otherwise a `repository-path` hash of the scan root is used and the scanner warns that IDs will differ across roots. The fallback hash keeps the absolute path out of bundle records.

`FileId` hashes repository identity and a normalized relative path. `EntityId` hashes `FileId`, ownership chain, declaration kind, and the per-file ordinal for that chain and kind. Line edits therefore do not change IDs. `ScopeId` derives from a file ID; nested owner scopes derive from their owner entity ID. `RelationshipId` hashes source, target, kind and origin. Local declarations retain IDs and `is_local: true`; the viewer hides them in its default projection.

The writer sorts ID collections and nested ID lists. `graph.json` excludes scan timestamps and absolute roots, allowing equal trees with one origin identity to produce equal graph output across checkouts.
