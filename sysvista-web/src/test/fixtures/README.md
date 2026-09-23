# Viewer fixtures

The projection and flow JSON fixtures are generated from the real CLI scan of
`sysvista-cli/tests/fixtures/hierarchy`. Run `./scripts/regen-schema.sh` from the
repository root to regenerate both fixtures and the schema/types. The script
normalizes the scan timestamp and root path so the committed files are stable.
Do not edit the JSON files by hand.
