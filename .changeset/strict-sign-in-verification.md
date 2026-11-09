---
wallet_standard: patch
---

# Reject sign-in messages that only embed expected fields

`verify_sign_in` used to accept a signed message when the requested fields appeared anywhere in the text. A wallet could put a different domain on the first line, or hide a transfer inside the statement, and still pass. Verification now parses the message the way `@solana/wallet-standard-util` does: the preamble must be the first line, every field the app set must match, and a field the app left unset must be absent. Domain and address may still be omitted; the wallet fills them in and the account is checked against the message.
