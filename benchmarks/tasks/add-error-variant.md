Add a new error variant `Suspended` to AuthError. A suspended account is different from a locked account: locked is automatic (too many failed attempts), suspended is manual (admin action).

Requirements:

- Add the Suspended variant to the AuthError type
- The authenticate function should check for suspension before checking credentials
- Add a `check_suspension` helper that queries the database
- Update all callers of authenticate to handle the new variant
- Ensure all contracts and effects are consistent

Do not modify any existing tests.
