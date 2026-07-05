# Retry

Retry policy is defined now and wired into request and reconnect structures, but automatic retry execution is reserved for future work.

## Policies

- `None`
- `Linear`
- `Exponential`
- `Custom`

## Defaults

The default is exponential backoff:

- Base delay: `500ms`
- Max delay: `30000ms`
- Factor: `2`
- Max attempts: `5`

## Usage

Retry is a policy object, not a hidden loop. Future reconnect managers and request senders should consult `delay_for_attempt()` or `should_retry()` and keep all retry logs attached to the communication logger.
