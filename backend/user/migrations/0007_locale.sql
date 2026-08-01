-- user module — persisted preferred locale (change: persist-user-locale, D1).
--
-- `locale` is the account's preferred language, written by the identity system
-- (set on any authenticated call that carries a non-empty locale, updated
-- last-writer-wins). It is a **system-relevant** field the backend reads on the
-- transactional-email path as the fallback language when a request carries none
-- (precedence: request locale → stored locale → English) — kept as a dedicated
-- column rather than under the client-owned `preferences` JSONB (D1).
--
-- NULL means "no recorded locale" (= English). Additive + backward compatible:
-- existing rows stay NULL until their next locale-carrying call.

ALTER TABLE users ADD COLUMN locale TEXT;
