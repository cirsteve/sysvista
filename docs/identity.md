# v2 identity and canonical output

SysVista v2 identifiers are content-addressed strings with a domain prefix. The hash input is length-separated by a zero byte so concatenated fields cannot collide.

## Repository and file identity

Repository identity uses `git remote get-url origin` when available. URL schemes, an SSH `git@` prefix, and a trailing `.git` are removed. If there is no origin, SysVista uses `repository.name` from `sysvista.toml`; if that is also absent, it uses the scanned directory basename and emits a warning because those IDs are not portable across renamed checkouts.

`FileId` hashes the repository identity and the normalized, slash-separated repository-relative path. It never includes an absolute checkout path.

## Entity identity

`EntityId` hashes the `FileId`, qualified ownership chain, declaration kind, and an overload discriminator. The discriminator is the zero-based source-order ordinal among declarations with the same name and kind under the same owner. Consequently, inserting lines does not change IDs. Reordering same-name declarations can change their IDs by design.

`ScopeId` derives from its owning file. `RelationshipId` derives from source ID, target ID, relationship kind, and origin.

## Canonical bundle

The v2 writer emits:

- `manifest.json` for run metadata and discovery inventory;
- `graph.json` for stable graph content;
- `diagnostics.json` for diagnostics;
- `index/scopes.json` for sorted scope children, owner mappings, and crossing relationship IDs.

Collections and nested identifier lists are sorted before serialization. `graph.json` contains neither scan timestamps nor absolute roots, so identical source trees and repository identities produce byte-identical graph output.
