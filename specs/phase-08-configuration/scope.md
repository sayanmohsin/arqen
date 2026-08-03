# Scope

Configuration is loaded from environment variables and optional config files.
Secrets (API keys, tokens, passwords) are redacted in logs and error messages.
AppState is constructed explicitly via a builder pattern, not a DI container.
Feature flags control optional functionality (http-server, thingd-native, logging, http-client).
Storage adapter selection is config-driven (memory, persistent, http).
