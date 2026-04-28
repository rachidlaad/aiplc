# Memories

AIPLC can use local memories to carry forward useful context between threads.

- `Use memories` reads stored memories when starting future threads.
- `Generate memories` allows AIPLC to write new local memories from future threads.
- `Reset all memories` removes the local memory files and rollout summaries for the current `CODEX_HOME`.

Safety notes:

- Memories are local configuration and state, not a cloud sync feature.
- Resetting memories does not delete existing thread history.
- Review your current `~/.codex/config.toml` if you need to disable memory generation globally.
