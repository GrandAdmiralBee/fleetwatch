# 03. Protobug Contract between agent and control-plane

- **Status:** draft v0.1
- **Date:** 2026-10-10
- **Related documents:** 01-containers

## Who opens the stream?

The stream is opened by the agent via `Hello` message to control-plane.

Firstly, the `Hello` msg is sent to control-plane.
In response it receives the `Welcome` msg with `session_id` and heartbeat interval.

`Hello` and `Welcome` msgs contain version info about control-plane and agent.
If major versions of protocol do not match - disconnect with reason "version mismatch".

The end machine is to be considered **down** when hearbeat doesn't reach control-plane in 3 intervals.

If the same device tries to connect the second time, the second connection breaks the first one with reason "replaced".

On `SIGTERM` the agent closes the steam on its end, control plane answers with closing its end.

## Connection

```mermaid
sequenceDiagram
    autonumber
    participant A as Agent
    participant C as Control-plane

    A->>C: Open stream
    A->>C: Hello (protocol version, agent version)

    alt Major protocol versions do not match
        C-->>A: Disconnect (VERSION_MISMATCH)
    else Hello accepted
        opt Same device already has an active session
            C-->>C: Close old session with Disconnect (REPLACED)
        end
        C-->>A: Welcome (session_id, heartbeat interval, protocol version)
    end

    loop Every heartbeat interval
        A->>C: Heartbeat (seq)
        C-->>A: HeartbeatAck (seq)
    end
```

## Disconnection

```mermaid
sequenceDiagram
    autonumber
    participant A as Agent
    participant C as Control-plane

    Note over A,C: Active session, heartbeats flowing

    alt Agent receives SIGTERM
        A->>C: Close agent end of the stream
        C-->>A: Close control-plane end of the stream
        Note over A: Agent exits
    else Session is replaced by a newer connection
        C-->>A: Disconnect (REPLACED)
        Note over A: Agent reconnects after backoff
    else Heartbeat is missing for 3 intervals
        Note over C: Device is considered down
        C-->>C: Close session
        Note over A: Agent reconnects after backoff
    end
```

## Session states

```mermaid
stateDiagram-v2
    [*] --> AwaitingWelcome: stream opened, Hello sent
    AwaitingWelcome --> Active: Welcome received
    AwaitingWelcome --> Closed: Disconnect (VERSION_MISMATCH)
    Active --> Closed: SIGTERM, stream closed by agent
    Active --> Closed: Disconnect (REPLACED)
    Active --> Closed: no heartbeat for 3 intervals
    Closed --> [*]
```
