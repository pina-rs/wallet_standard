---
wallet_standard_browser: minor
---

# Add subscription guards and wallet-following streams

`Subscription` owns a listener closure and its unsubscribe function and detaches on drop, replacing the `on_register` / `on_unregister` API whose closure and dispose handle could drift apart into a leaked listener or a silently dead one. `BrowserWallet::account_changes` streams the wallet's account changes through `standard:events` so apps stop going stale when the user switches accounts inside the wallet, `BrowserWallet::on_change_with` exposes the raw change notifications, and `watch_wallets` streams the page's wallet registry including late registrations. The leptos example follows account changes through the stream.
