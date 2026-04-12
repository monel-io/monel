Refactor the authenticate function: extract the credential verification logic into a separate `verify_credentials` helper function.

Requirements:

- The new function should take credentials and return Result<User, AuthError>
- Move the database lookup and password verification into the new function
- The authenticate function should call verify_credentials
- Effects should be properly split between the two functions
- Contracts should be properly split — verify_credentials gets the credential-checking contracts, authenticate keeps the session-creation contracts

Verify that all contracts and effects are consistent after the refactoring.
