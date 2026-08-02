# Scope

Use `thingd::MemoryEngine` for ephemeral mode and `thingd::PersistentEngine` for
single-process durable mode. Keep Axum independent of concrete engine types.
Do not add a second storage semantic layer or alter the thingd repository.
