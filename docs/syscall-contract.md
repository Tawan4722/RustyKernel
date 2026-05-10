# Syscall Contract (v1)

Target: x86_64 ring3 to ring0 boundary.

## Number Table

- `1`: `write`
- `2`: `exit`
- `3`: `yield` (reserved)

## Error Contract

- Success: `0` or positive values.
- Failure: negative errno-style values.
  - `-22` (`EINVAL`) invalid argument
  - `-38` (`ENOSYS`) unknown syscall

## Current v1 Behavior

- Smoke path uses `int 0x80`.
- The first trap is interpreted as `write`.
- The second trap is interpreted as `exit`.

This intentionally keeps the ABI narrow while validating user->kernel control transfer.
