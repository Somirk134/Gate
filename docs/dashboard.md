# Dashboard

Dashboard is the operator landing page. It should answer three questions quickly: is Gate connected, which tunnels are active, and where should I look next?

## Primary Signals

| Signal | Purpose |
| --- | --- |
| Server status | Shows whether the selected Gate server is reachable |
| Tunnel count | Shows active, stopped, and error tunnels |
| Traffic | Shows current upload and download movement |
| Activity | Shows recent tunnel, server, and authentication events |
| Quick actions | Creates common next steps without digging through navigation |

## Expected Layout

1. Header with selected workspace and server health.
2. Compact KPI row.
3. Active tunnel overview.
4. Traffic chart.
5. Recent activity.
6. Quick actions.

## Operational Use

- Check Dashboard immediately after starting a tunnel.
- Use the traffic chart to confirm callbacks or test requests.
- Open Log Center from the latest event when troubleshooting.
- Treat `error` or `disconnected` as action-required states.

## Screenshot

![Dashboard screenshot](../assets/screenshots/dashboard.png)
