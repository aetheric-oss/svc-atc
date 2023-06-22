# Software Design Document (SDD) - `svc-atc` 

<center>

<img src="https://github.com/Arrow-air/tf-github/raw/main/src/templates/doc-banner-services.png" style="height:250px" />

</center>

## Overview

This document details the software implementation of `svc-atc`.

This service is an automated air traffic control service responsible for maintaining safe separation of VTOL aircraft.

Attribute | Description
--- | ---
Status | Draft

## Related Documents

Document | Description
--- | ---
[High-Level Concept of Operations (CONOPS)](https://github.com/Arrow-air/se-services/blob/develop/docs/conops.md) | Overview of Arrow microservices.
[High-Level Interface Control Document (ICD)](https://github.com/Arrow-air/se-services/blob/develop/docs/icd.md)  | Interfaces and frameworks common to all Arrow microservices.
[Requirements - `svc-atc`](https://nocodb.arrowair.com/dashboard/#/nc/view/1f06e270-d36d-41cb-85ea-25a5d5d60c77) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-atc`](./conops.md) | Defines the motivation and duties of this microservice.
[Interface Control Document (ICD) - `svc-atc`](./icd.md) | Defines the inputs and outputs of this microservice.

## Module Attributes

Attribute | Applies | Explanation
--- | --- | ---
Safety Critical | Y | Collision avoidance and rerouting.

## Logic

### Initialization

At initialization this service creates two servers on separate threads: a GRPC server and a REST server.

The REST server expects the following environment variables to be set:
- `DOCKER_PORT_REST` (default: `8000`)

The GRPC server expects the following environment variables to be set:
- `DOCKER_PORT_GRPC` (default: `50051`)

### Loop

As a REST and GRPC server, this service awaits requests and executes handlers.

Some handlers **require** the following environment variables to be set:
- `STORAGE_HOST_GRPC`
- `STORAGE_PORT_GRPC`

This information allows this service to connect to other microservices to obtain information requested by the client.

For detailed sequence diagrams regarding request handlers, see [Interface Handlers](#interface-handlers).

### Cleanup

No cleanup behavior.

## Interface Handlers

### `ack`

Aircraft will confirm that they've received a flight plan.

**Nominal - Carrier Confirms**
```mermaid
sequenceDiagram
    autonumber
    participant client as Networked Node
    participant service as svc-atc
    participant storage as svc-storage
    client-->>service: (REST) POST /atc/ack/flight confirmed
    service-->>storage: Update flight_plan.carrier_ack = NOW()
```

**Off-Nominal - Carrier Denies**
```mermaid
sequenceDiagram
    autonumber
    participant client as Networked Node
    participant service as svc-atc
    participant storage as svc-storage
    client-->>service: (REST) POST /atc/ack/flight denied
    service-->>scheduler: TODO(R4) Attempt Reroute
    alt Reroute Fails
        scheduler->>service:: Failed
        service-->>scheduler:: Cancel Flight
    end
```

## Common Actions

Attempt aircraft reroute

```mermaid
sequenceDiagram
    autonumber
    participant client as Networked Node
    participant service as svc-atc
    participant storage as svc-storage
    client-->>service: (REST) POST /atc/ack/flight denied
    alt Tier 1
        service-->>scheduler: TODO(R4) Attempt Reroute
    end
    alt Tier 2
        service-->>scheduler: Cancel flight
    end
    alt Tier 3
        service-->>storage: Cancel flights
    end
```
