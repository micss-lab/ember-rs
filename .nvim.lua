-- Project-local Neovim config (requires `exrc` to be enabled and this file
-- to be trusted via `:trust`, see `:h exrc`).
--
-- This workspace has two mutually-incompatible cargo targets (std and
-- no_std/xtensa), so a plain `cargo check`/`clippy` run against the default
-- target fails with thousands of errors. `cargo xtask lsp` runs clippy once
-- per target and concatenates the JSON diagnostics, which is what
-- rust-analyzer expects from `check.overrideCommand`.
--
-- A workspace-root `rust-analyzer.toml` with the same setting already
-- exists, but rust-analyzer's ratoml support silently ignores
-- workspace-root configs for "virtual" workspaces (a `[workspace]` with no
-- root `[package]`, which is what this repo is) on rust-analyzer versions
-- before the fix in https://github.com/rust-lang/rust-analyzer/pull/21704.
-- Setting it here via the LSP client config sidesteps that bug.
vim.lsp.config("rust_analyzer", {
    settings = {
        ["rust-analyzer"] = {
            check = {
                overrideCommand = { "cargo", "xtask", "lsp" },
            },
        },
    },
})
