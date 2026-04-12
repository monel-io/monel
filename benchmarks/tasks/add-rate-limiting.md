Add rate limiting to the authenticate function. Requirements:

- Maximum 10 authentication attempts per IP address per minute
- When the limit is exceeded, return a RateLimited error
- The rate limit check should happen before credential verification (fail fast)
- Log rate limit violations
- Do not modify any existing tests — only add new behavior

After making the changes, verify that:
1. The new error variant is properly declared
2. All effects are declared
3. The function's contracts are updated to reflect the new behavior
