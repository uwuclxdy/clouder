-- 013: Store the dashboard API key in encrypted form so users can view it
-- without regenerating. The hash column stays for constant-time lookup on
-- inbound requests; the ciphertext is decrypted only on the user's own
-- profile page using OAUTH_ENCRYPTION_KEY (AES-256-GCM, nonce-prefixed).
--
-- Existing rows keep api_key_ciphertext NULL until the user regenerates;
-- the profile page falls back to a "regenerate to view" placeholder.

ALTER TABLE dashboard_users ADD COLUMN api_key_ciphertext TEXT;
