![Aetheric Banner](https://github.com/aetheric-oss/.github/raw/main/assets/doc-banner.png)

# Interface Control Document (ICD) - `svc-atc`

## :telescope: Overview

This document defines the gRPC and REST interfaces unique to the `svc-atc` microservice.

Attribute | Description
--- | ---
Status | Draft

## :books: Related Documents

Document | Description
--- | ---
[High-Level Concept of Operations (CONOPS)](https://github.com/aetheric-oss/se-services/blob/develop/docs/conops.md) | Overview of Aetheric microservices.
[High-Level Interface Control Document (ICD)](https://github.com/aetheric-oss/se-services/blob/develop/docs/icd.md)  | Interfaces and frameworks common to all Aetheric microservices.
[Requirements - `svc-atc`](https://nocodb.aetheric.nl/dashboard/#/nc/view/1f06e270-d36d-41cb-85ea-25a5d5d60c77) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-atc`](./conops.md) | Defines the motivation and duties of this microservice.
[Software Design Document (SDD) - `svc-atc`](./sdd.md) | Specifies the internal activity of this microservice.

## :hammer: Frameworks

See the High-Level ICD.

## :speech_balloon: REST

See the High-Level ICD for common interfaces.

### Files

| File Location | Description |
--- | ---
`server/src/rest/api.rs` | Implements the REST endpoints.

### Authentication

See the High-Level ICD.

### Endpoints

See the [hosted REST interface documentation](https://www.arrowair.com/docs/documentation/services/api/rest/develop).

## :speech_balloon: gRPC

### Files

These interfaces are defined in a protocol buffer file, `proto/grpc.proto`.

### Integrated Authentication & Encryption

See the High-Level ICD.

### Endpoints

This microservice currently only implements the `is_ready` endpoint, which returns true if the microservice has completed booting and is ready for other requests.
