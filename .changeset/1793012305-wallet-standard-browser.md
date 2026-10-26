---
wallet_standard_browser: feat
---

# Add Subscription guards and wallet-following streams for events and registry changes

Subscription owns a listener closure and its unsubscribe function, detaching on drop, and replaces the on_register/on_unregister API that leaked or silently died when the closure outlived its dispose handle. BrowserWallet::account_changes streams the wallet's own account changes through standard:events, on_change_with exposes the raw notifications, and watch_wallets streams the page registry including late registrations. The leptos example now follows account changes through the stream, and both examples build with codegen-units 1 and stripped release profiles.
