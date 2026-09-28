//! Framework-agnostic glue shared by the Leptos and Dioxus examples.
//!
//! Two halves:
//!
//! - [`rpc`]: a tiny surfpool / Solana JSON-RPC client built on `fetch`.
//! - [`wallet`]: a complete Wallet Standard wallet implemented in Rust and
//!   registered into the page through
//!   `wallet_standard_browser::register_wallet`, so the dApp side of the
//!   example talks to it exactly like it would talk to Phantom or any other
//!   injected wallet.
//!
//! - [`tx`]: helpers to build and serialize transfer transactions.

pub mod rpc;
pub mod tx;
pub mod wallet;

pub const DEFAULT_RPC_URL: &str = "http://127.0.0.1:8899";

/// Demo recipient used by the transfer buttons. Throwaway address that only
/// ever receives funds on a local surfpool instance.
pub const DEMO_RECIPIENT: &str = "9R4BL32WojriHd4kDpDD9z9giuDFwEa52kVvvSCwxBRD";
