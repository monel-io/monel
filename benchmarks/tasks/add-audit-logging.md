Add audit logging to all database write operations in the auth module.

Requirements:

- Every function that has Db.write in its effects should also log what was written
- Use the Log.write effect for audit logging
- Log entries should include: timestamp, operation name, affected entity ID
- Add an AuditLog type to hold log entries
- Functions that previously only had Db.write should now also declare Log.write in their effects
- Do not change any function's behavior — only add logging

Verify that all effects declarations are updated to include Log.write where needed.
