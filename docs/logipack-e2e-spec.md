# LogiPack End-to-End System Specification

This document defines the authoritative MVP execution/design specification for the LogiPack diploma project.

Terminology used in this document:

- `Current reality` means behavior or schema shape already visible in the current repository.
- `Recommended final MVP` means the target contract the system should converge to for implementation completion, review, and diploma defense.
- When the two differ, the difference is called out explicitly and a decision is recommended.

# 1. System Purpose and Scope

LogiPack is a web information system for a logistics company.

- `LogiPack Hub` is the user-facing platform: SvelteKit web UI plus Rust REST API.
- `LogiCore` is the backend engine: domain rules, application use-cases, relational persistence, immutable timeline append, and audit emission.

The system exists to manage the operational lifecycle of shipments across offices and employees while preserving two distinct audit views:

- a mutable operational snapshot for daily work and reporting
- an immutable shipment event timeline for traceability

Primary users:

- administrators who manage master data and oversee operations
- office employees who create and advance shipments within their assigned offices

MVP scope:

- client management
- office management
- employee management
- user-to-role mapping from Auth0 identities
- shipment creation and status progression
- office-scoped employee permissions
- reports for shipments by status, client, office, and period
- immutable shipment transition timeline backed by Strata packages
- operational audit log for admin and shipment actions

Explicitly out of scope for MVP:

- mobile UI
- maps or geolocation visualization
- email or SMS notifications
- route optimization
- pricing, invoicing, weight calculation, parcel dimensions
- hot/db/cold archival retrieval layers as a production-ready subsystem
- distributed scaling beyond a single PostgreSQL-backed deployment
- Docker Compose orchestration
- external BI/export tooling beyond basic API/UI reports

# 2. Actors and Roles

## Admin

Permissions:

- access all admin and employee data areas
- create, view, update, and deactivate clients
- create, view, update, and deactivate offices
- create, view, update, assign offices to, and deactivate employees
- create shipments
- change shipment status regardless of office assignment
- view all shipments
- view all reports
- view audit events
- view shipment immutable timelines

Accessible areas:

- `/{lang}/app/admin/**`
- admin dashboard
- admin shipment console
- clients, offices, employees, reports, audit pages

Forbidden actions:

- none inside MVP scope, except actions explicitly blocked by domain rules such as invalid shipment transitions

## Employee

Permissions:

- access employee console
- view shipments that belong to offices assigned to the employee
- create shipments only for assigned offices
- advance shipment status only when the current responsible office is assigned to the employee
- send a shipment to `IN_TRANSIT` toward any active destination office
- read immutable timeline for shipments they are authorized to view

Accessible areas:

- `/{lang}/app/employee/**`
- employee dashboard
- shipment list, create, detail, and profile pages

Forbidden actions:

- client, office, employee, role, report, and audit administration
- shipment actions outside assigned office scope
- any admin-only API endpoint

## Authenticated User Without Role

Definition:

- valid Auth0 identity exists
- local user row exists
- no application role is linked through `user_roles`

Permissions:

- complete login
- be provisioned locally through `/ensure-user`
- query `/me`
- reach the no-access page

Accessible areas:

- login/callback/logout flow
- no-access page

Forbidden actions:

- all business operations
- all shipment CRUD/actions
- all admin areas
- all reports and audit views

Recommended behavior:

- UI must redirect such a user to `/{lang}/app/no-access`
- API must return `403 Forbidden` for protected business endpoints

## Optional Future Roles

Future roles that fit the current M:N design but are not part of MVP:

- `report_viewer`
- `auditor`
- `office_manager`

Recommended rule:

- do not implement role-specific behavior beyond `admin` and `employee` in MVP
- keep the schema extensible, but keep authorization logic hard-coded to the two MVP roles

# 3. Domain Model

## User

Purpose:

- represents the authenticated application principal linked to Auth0

Important fields:

- `id`
- `auth0_sub`
- `email`
- `name`
- `created_at`
- `password_hash`

Relationships:

- M:N with `roles` via `user_roles`
- 1:0..1 with `employees`
- 1:N with `shipment_status_history` through `actor_user_id`
- 1:N logical relationship with `audit_events.actor_user_id`

Lifecycle notes:

- created or linked during Auth0 callback via `ensure_user`
- may exist without any role
- should remain even if the employee record is later soft-deleted

Mutable or immutable:

- mutable: `name`, `email`, `auth0_sub` link at provision time
- immutable enough for audit identity: `id`

UI/API appearance:

- mostly indirect through `/me`, employee views, and audit metadata

Current reality:

- `email` and `password_hash` are nullable in code
- `password_hash` is effectively unused under Auth0

Recommended final MVP:

- treat `auth0_sub` as the primary external identity key
- keep `password_hash` nullable and unused
- require a non-empty email for any Auth0-provisioned user

## Role

Purpose:

- grants application permissions independently of Auth0

Important fields:

- `id`
- `name`

Relationships:

- M:N with `users` via `user_roles`

Lifecycle notes:

- seeded on demand or explicitly via migration
- only `admin` and `employee` are valid MVP values

Mutable or immutable:

- effectively immutable reference data

UI/API appearance:

- surfaced via `/me`
- influences route guards and navigation

## Employee

Purpose:

- marks a user as operational staff and stores office assignment context

Important fields:

- `id`
- `user_id`
- `created_at`
- `updated_at`
- `deleted_at`

Relationships:

- 1:1 to `users`
- M:N with `offices` via `employee_offices`

Lifecycle notes:

- created only for an existing local user
- soft-deleted rather than hard-deleted
- office assignments determine shipment permissions

Mutable or immutable:

- mutable: active/inactive state, office assignments
- immutable identity: `id`

UI/API appearance:

- admin employee list/detail/edit/assignment pages
- `/employees` API

Current reality:

- no employee-specific editable business fields exist beyond office assignment
- current `PUT /employees/{id}` is effectively a timestamp touch

Recommended final MVP:

- employee editing should mean activation state plus office assignment, not free-text profile editing
- user display name should continue to come from `users.name`

## Office

Purpose:

- operational location where shipments are accepted, processed, or delivered

Important fields:

- `id`
- `name`
- `city`
- `address`
- `created_at`
- `updated_at`
- `deleted_at`

Relationships:

- M:N with `employees`
- 1:N with `shipments.current_office_id`
- 1:N with `shipment_status_history.office_id`
- logical relationship with `audit_events.office_id`

Lifecycle notes:

- soft-deleted
- should remain referenceable by historical data

Mutable or immutable:

- mutable: descriptive fields and active/inactive state

UI/API appearance:

- admin office CRUD
- shipment creation/status forms
- reports and audit filters

## Client

Purpose:

- shipment customer record used for shipment ownership and reporting

Important fields:

- `id`
- `name`
- `phone`
- `email`
- `created_at`
- `updated_at`
- `deleted_at`

Relationships:

- 1:N with `shipments`

Lifecycle notes:

- soft-deleted
- must remain visible historically if referenced by shipments

Mutable or immutable:

- mutable: name/contact fields and active/inactive state

UI/API appearance:

- admin client CRUD
- shipment creation form
- client-based reporting

Current reality:

- `shipments` references only one client

Recommended final MVP:

- interpret `client_id` as the contracting client or sender-of-record
- do not claim separate sender/receiver support in MVP

## Shipment

Purpose:

- mutable operational snapshot of a shipment’s current state

Important fields:

- `id`
- `client_id`
- `current_status`
- `current_office_id`
- `created_at`
- `updated_at`

Relationships:

- N:1 to `clients`
- N:1 to `offices` through `current_office_id`
- 1:N to `shipment_status_history`
- 1:1 logical mapping to `streams` by shared identifier in recommended MVP

Lifecycle notes:

- created in `NEW`
- updated whenever a valid transition occurs
- never hard-deleted in MVP

Mutable or immutable:

- mutable snapshot

UI/API appearance:

- shipment lists
- shipment detail
- status change form
- reports

Recommended final MVP semantic rule:

- `current_office_id` means the office currently responsible for the next operational action
- during `IN_TRANSIT`, this is the destination office, not the source office

## ShipmentStatusHistory

Purpose:

- relational, report-friendly projection of shipment state transitions

Important fields:

- `id`
- `shipment_id`
- `from_status`
- `to_status`
- `changed_at`
- `actor_user_id`
- `office_id`
- `notes`

Relationships:

- N:1 to `shipments`
- N:1 to `users`
- N:1 to `offices`

Lifecycle notes:

- append-only
- one row per shipment state change including creation as `NULL -> NEW`

Mutable or immutable:

- immutable once inserted

UI/API appearance:

- shipment detail status history
- report source for period/office/status analysis

Current reality:

- only one office column exists

Recommended final MVP:

- add `from_office_id` and `to_office_id` as nullable columns
- keep the existing single-office value only as a compatibility field if migration avoidance is necessary

## Stream

Purpose:

- event stream root for immutable shipment event storage

Important fields:

- `id`
- `kind`
- `head_hash`
- `created_at`

Relationships:

- 1:N to `packages`

Lifecycle notes:

- one stream per shipment in MVP
- created at shipment creation time

Mutable or immutable:

- mutable only in `head_hash`
- otherwise append-only metadata

UI/API appearance:

- indirect; used to read shipment timeline

## Package

Purpose:

- immutable Strata package containing a shipment timeline event

Important fields:

- `hash`
- `stream_id`
- `prev_hash`
- `seq`
- `event_type`
- `scb`
- `created_at`

Relationships:

- N:1 to `streams`

Lifecycle notes:

- appended in strict sequence order
- linked by `prev_hash`
- forms the tamper-evident timeline chain

Mutable or immutable:

- immutable

UI/API appearance:

- shipment timeline view
- optional raw package panel for diploma demonstration

Current reality:

- the code already uses `seq`, even though the brief’s abbreviated schema omitted it

Recommended final MVP:

- treat `seq` as mandatory and part of the authoritative schema

## AuditEvent

Purpose:

- operational audit row for significant user actions across the system

Important fields:

- `id`
- `occurred_at`
- `actor_user_id`
- `actor_display_name`
- `action_key`
- `entity_type`
- `entity_id`
- `entity_label`
- `office_id`
- `office_label`
- `target_route`
- `metadata_json`
- `request_id`

Relationships:

- denormalized by design
- may logically reference users, offices, shipments, clients, or employees

Lifecycle notes:

- append-only
- written for admin CRUD actions and shipment actions

Mutable or immutable:

- immutable

UI/API appearance:

- admin audit page
- pagination and filtering endpoint

# 4. Database Specification

## 4.1 Current Schema Reality

The repository currently implies this authoritative schema reality:

- `users.email` is nullable
- `users.password_hash` is nullable
- `users.auth0_sub` exists and is nullable but unique when present
- `employees.full_name` no longer exists; display name now lives in `users.name`
- `clients`, `offices`, and `employees` have `deleted_at`, `created_at`, `updated_at`
- `shipments` is still single-client
- `shipment_status_history` uses `actor_user_id`, not `actor_employee_id`
- `streams` and `packages` are implemented in a separate event-store crate
- `packages.seq` exists and is required
- `audit_events` exists and is append-only

## 4.2 Recommended Final MVP Shape

The final MVP should keep the existing table set, with targeted corrections rather than a redesign.

### Tables, Keys, and Relationships

| Table                     | Purpose                          | PK                         | Important FKs / Relations                                    | Mutability      |
| ------------------------- | -------------------------------- | -------------------------- | ------------------------------------------------------------ | --------------- |
| `users`                   | local app principal              | `id`                       | to `employees`, `user_roles`, `shipment_status_history`      | mutable         |
| `roles`                   | role reference data              | `id`                       | to `user_roles`                                              | reference data  |
| `user_roles`              | users-to-roles M:N               | `(user_id, role_id)`       | `user_id -> users.id`, `role_id -> roles.id`                 | mutable links   |
| `clients`                 | customer master data             | `id`                       | referenced by `shipments.client_id`                          | soft-deletable  |
| `employees`               | operational staff marker         | `id`                       | `user_id -> users.id` unique                                 | soft-deletable  |
| `offices`                 | operational location             | `id`                       | referenced by `employee_offices`, `shipments`, `history`     | soft-deletable  |
| `employee_offices`        | employees-to-offices M:N         | `(employee_id, office_id)` | `employee_id -> employees.id`, `office_id -> offices.id`     | mutable links   |
| `shipments`               | mutable shipment snapshot        | `id`                       | `client_id -> clients.id`, `current_office_id -> offices.id` | mutable         |
| `shipment_status_history` | relational transition projection | `id`                       | `shipment_id -> shipments.id`, actor/office refs             | append-only     |
| `streams`                 | immutable stream root            | `id`                       | one stream per shipment                                      | append metadata |
| `packages`                | immutable event packages         | `hash`                     | `stream_id -> streams.id`                                    | append-only     |
| `audit_events`            | operational audit log            | `id`                       | logical refs only                                            | append-only     |

Required M:N relations for diploma criteria:

- `users <-> roles` via `user_roles`
- `employees <-> offices` via `employee_offices`

## 4.3 Table-by-Table Constraints

### `users`

PK:

- `id UUID`

Required columns:

- `name TEXT NOT NULL`
- `created_at TIMESTAMPTZ NOT NULL`

Nullable columns:

- `email TEXT NULL`
- `password_hash TEXT NULL`
- `auth0_sub TEXT NULL`

Uniqueness:

- unique on `auth0_sub` when not null
- unique on `lower(email)` when not null

Recommended indexes:

- unique index on `lower(email)` for case-insensitive lookup
- unique index on `auth0_sub`

Recommended final MVP decision:

- keep `email` nullable at DB level for compatibility
- require email at application level for Auth0-provisioned users

### `roles`

PK:

- `id UUID`

Columns:

- `name TEXT NOT NULL UNIQUE`

Allowed values:

- `admin`
- `employee`

Recommended index:

- unique index on `name`

### `user_roles`

PK:

- composite PK `(user_id, role_id)`

FKs:

- `user_id -> users.id ON DELETE CASCADE`
- `role_id -> roles.id ON DELETE CASCADE`

Integrity rules:

- duplicate role links are forbidden by PK

### `clients`

PK:

- `id UUID`

Columns:

- `name TEXT NOT NULL`
- `phone TEXT NULL`
- `email TEXT NULL`
- `deleted_at TIMESTAMPTZ NULL`
- `created_at TIMESTAMPTZ NOT NULL`
- `updated_at TIMESTAMPTZ NOT NULL`

Soft delete behavior:

- row remains in DB
- list/get/update APIs must exclude rows where `deleted_at IS NOT NULL` unless explicitly requested by an admin maintenance endpoint

Recommended indexes:

- index on `deleted_at`
- index on `lower(name)`
- optional index on `lower(email)` if client search by email is needed

Integrity rules:

- do not hard-delete if referenced historically
- recommended business rule: soft-delete only if no active shipment references the client

### `employees`

PK:

- `id UUID`

Columns:

- `user_id UUID NOT NULL UNIQUE`
- `deleted_at TIMESTAMPTZ NULL`
- `created_at TIMESTAMPTZ NOT NULL`
- `updated_at TIMESTAMPTZ NOT NULL`

FKs:

- `user_id -> users.id ON DELETE RESTRICT`

Soft delete behavior:

- set `deleted_at`
- employee no longer appears in active lists
- office links should be removed or ignored for active authorization

Recommended indexes:

- unique on `user_id`
- index on `deleted_at`

Important gap in current reality:

- current code removes all `user_roles` when deleting an employee

Recommended fix:

- remove only the `employee` role
- preserve unrelated roles such as `admin`
- delete `employee_offices` links on employee deactivation

### `offices`

PK:

- `id UUID`

Columns:

- `name TEXT NOT NULL`
- `city TEXT NOT NULL`
- `address TEXT NOT NULL`
- `deleted_at TIMESTAMPTZ NULL`
- `created_at TIMESTAMPTZ NOT NULL`
- `updated_at TIMESTAMPTZ NOT NULL`

Soft delete behavior:

- same approach as `clients`

Recommended indexes:

- index on `deleted_at`
- index on `lower(city)`
- unique partial index on `(lower(name), lower(city), lower(address)) WHERE deleted_at IS NULL`

Recommended business rule:

- do not soft-delete an office while it is referenced by any non-terminal shipment or any active employee office assignment

### `employee_offices`

PK:

- composite PK `(employee_id, office_id)`

FKs:

- `employee_id -> employees.id ON DELETE CASCADE`
- `office_id -> offices.id ON DELETE CASCADE`

Recommended indexes:

- PK is sufficient for duplicates
- secondary index on `office_id` for reverse lookup

### `shipments`

PK:

- `id UUID`

Columns:

- `client_id UUID NOT NULL`
- `current_status TEXT NOT NULL`
- `current_office_id UUID NULL` in current schema
- `created_at TIMESTAMPTZ NOT NULL`
- `updated_at TIMESTAMPTZ NOT NULL`

FKs:

- `client_id -> clients.id ON DELETE RESTRICT`
- `current_office_id -> offices.id ON DELETE SET NULL`

Recommended indexes:

- index on `client_id`
- index on `current_status`
- composite index on `(current_office_id, current_status)`
- composite index on `(created_at, current_status)`

Recommended final MVP decision:

- treat `current_office_id` as application-required for all active shipments
- leave the DB column nullable only if migration timing is constrained

### `shipment_status_history`

PK:

- `id BIGINT GENERATED`

Columns in current reality:

- `shipment_id UUID NOT NULL`
- `from_status TEXT NULL`
- `to_status TEXT NOT NULL`
- `changed_at TIMESTAMPTZ NOT NULL`
- `actor_user_id UUID NULL`
- `office_id UUID NULL`
- `notes TEXT NULL`

FKs:

- `shipment_id -> shipments.id ON DELETE CASCADE`
- `actor_user_id -> users.id ON DELETE SET NULL`
- `office_id -> offices.id ON DELETE SET NULL`

Recommended indexes:

- `(shipment_id, changed_at)`
- `(to_status, changed_at)`
- `(office_id, changed_at)`
- `(actor_user_id, changed_at)`

Recommended final MVP adjustment:

- add `from_office_id UUID NULL`
- add `to_office_id UUID NULL`
- keep `actor_user_id`
- keep `notes`

Reason:

- shipment movement between offices must be queryable without decoding Strata SCB

### `streams`

PK:

- `id UUID`

Columns:

- `kind TEXT NOT NULL`
- `head_hash BYTEA NULL`
- `created_at TIMESTAMPTZ NOT NULL`

Recommended indexes:

- PK only is sufficient for MVP
- optional index on `kind`

Recommended mapping:

- `streams.id = shipments.id`
- `streams.kind = 'shipment'`

### `packages`

PK:

- `hash BYTEA`

Columns:

- `stream_id UUID NOT NULL`
- `prev_hash BYTEA NULL`
- `seq BIGINT NOT NULL`
- `event_type TEXT NOT NULL`
- `scb BYTEA NOT NULL`
- `created_at TIMESTAMPTZ NOT NULL`

FKs:

- `stream_id -> streams.id ON DELETE CASCADE`

Required uniqueness:

- unique `(stream_id, seq)`

Recommended indexes:

- `(stream_id, seq)`
- `(stream_id, prev_hash)`

Recommended event type normalization:

- current reality: mixed values such as `shipment`, `ShipmentCreated`, `StatusChanged`
- recommended final MVP: normalize to lowercase dot notation:
  - `shipment.stream_initialized`
  - `shipment.created`
  - `shipment.status_changed`

### `audit_events`

PK:

- `id UUID`

Columns:

- `occurred_at TIMESTAMPTZ NOT NULL`
- `actor_user_id UUID NULL`
- `actor_display_name TEXT NULL`
- `action_key TEXT NOT NULL`
- `entity_type TEXT NULL`
- `entity_id TEXT NULL`
- `entity_label TEXT NULL`
- `office_id UUID NULL`
- `office_label TEXT NULL`
- `target_route TEXT NULL`
- `metadata_json JSONB NULL`
- `request_id TEXT NULL`

Current reality:

- no FK constraints are defined

Recommended final MVP decision:

- keep this denormalized and append-only
- optional FKs may be skipped deliberately because audit rows must survive referenced entity deactivation

Recommended indexes:

- `(occurred_at DESC, id DESC)`
- `action_key`
- `(entity_type, entity_id)`
- `office_id`
- `actor_user_id`
- `request_id`

## 4.4 Enum Recommendations

`shipment_status`:

- `NEW`
- `ACCEPTED`
- `PROCESSED`
- `IN_TRANSIT`
- `DELIVERED`
- `CANCELLED`

`role.name`:

- `admin`
- `employee`

`streams.kind`:

- `shipment`

`packages.event_type`:

- `shipment.stream_initialized`
- `shipment.created`
- `shipment.status_changed`

`audit_events.entity_type`:

- `shipment`
- `client`
- `office`
- `employee`
- `user`
- `role`
- `system`

Representative `audit_events.action_key` values:

- `shipment.created`
- `shipment.status_updated`
- `client.created`
- `client.updated`
- `client.deleted`
- `office.created`
- `office.updated`
- `office.deleted`
- `employee.created`
- `employee.updated`
- `employee.deleted`
- `employee.assigned_to_office`
- `employee.removed_from_office`

## 4.5 Explicit Design Questions and Recommended Fixes

### `shipments.client_id`

Current reality:

- one shipment references one client

Implication:

- the system cannot distinguish sender and receiver as separate managed clients

Recommended final MVP:

- keep `client_id` as the single customer-of-record field
- label it in UI as `Client`
- state clearly in diploma documentation that sender/receiver split is deferred

Future evolution:

- preferred path is `sender_client_id` plus `receiver_client_id`
- alternative generalized path is a `shipment_parties` table with party roles

Reason for not changing MVP:

- current reports, UI, and schema are all built around one client relation
- sender/receiver split adds model, validation, UI, and report complexity without being required for the diploma MVP

### `shipment_status_history.actor_user_id` user-based vs employee-based

Current reality:

- transition history stores `actor_user_id`

Recommended final MVP:

- keep `actor_user_id` as the authoritative actor reference
- do not replace it with `employee_id`
- optionally add nullable `actor_employee_id` later if HR-specific reporting becomes necessary

Reason:

- admins may perform shipment actions but may not have an employee row
- user identity is the authentication boundary
- employee can still be derived by joining `employees.user_id`

### Mapping `streams` and `packages` to shipments

Current reality:

- event store stream id already equals shipment id in code

Recommended final MVP:

- formalize one shipment stream per shipment
- `streams.id = shipments.id`
- `streams.kind = 'shipment'`
- default UI timeline hides the stream-initialization package and shows business events only

### Questionable areas in current schema/code

- `shipment_status_history` lacks separate `from_office_id` and `to_office_id`
- `shipments.current_office_id` is nullable, which weakens office-scoped authorization
- `audit_events.request_id` exists but is not yet consistently populated
- employee soft-delete currently removes all roles instead of only the employee role
- office/client soft-delete can leave active shipments referencing hidden master data

# 5. Shipment Lifecycle Specification

## 5.1 State Machine

Recommended final MVP state machine:

| From           | To             | Allowed | Notes                                                                   |
| -------------- | -------------- | ------- | ----------------------------------------------------------------------- |
| `NEW`          | `ACCEPTED`     | yes     | shipment accepted at current office                                     |
| `ACCEPTED`     | `PROCESSED`    | yes     | shipment processed at current office                                    |
| `PROCESSED`    | `IN_TRANSIT`   | yes     | requires destination office                                             |
| `IN_TRANSIT`   | `ACCEPTED`     | yes     | arrival at destination office; controlled loop for multi-office routing |
| `IN_TRANSIT`   | `DELIVERED`    | yes     | final delivery from responsible office                                  |
| `NEW`          | `CANCELLED`    | yes     | allowed                                                                 |
| `ACCEPTED`     | `CANCELLED`    | yes     | allowed                                                                 |
| `PROCESSED`    | `CANCELLED`    | yes     | allowed                                                                 |
| `IN_TRANSIT`   | `CANCELLED`    | yes     | allowed                                                                 |
| any other pair | any other pair | no      | reject                                                                  |

Terminal states:

- `DELIVERED`
- `CANCELLED`

Terminal rule:

- no transitions are allowed from a terminal state

Current reality:

- the code already implements the `IN_TRANSIT -> ACCEPTED` loop even though the brief’s simplified progression omits it

Recommended decision:

- keep the loop and document it explicitly
- without it, multi-office shipment handling is not operationally credible

## 5.2 Transition Rules

### Creation

On creation:

- shipment is inserted with `current_status = NEW`
- `current_office_id` must be the intake office
- one `shipment_status_history` row is inserted with `from_status = NULL`, `to_status = NEW`
- one shipment stream is ensured
- one package is appended for stream initialization if that pattern is retained
- one package is appended for `shipment.created`
- one `audit_events` row is inserted with action `shipment.created`

### Non-office-hop transitions

For transitions that do not move the shipment between offices:

- `NEW -> ACCEPTED`
- `ACCEPTED -> PROCESSED`
- `IN_TRANSIT -> DELIVERED`
- any allowed cancellation

Rules:

- `to_office_id` must be omitted
- `shipments.current_office_id` remains unchanged

### Office-hop transitions

For `PROCESSED -> IN_TRANSIT`:

- `to_office_id` is required
- `to_office_id` must reference an active office
- `to_office_id` may equal the current office for same-office final delivery dispatch, or differ for inter-office transfer
- snapshot `current_office_id` becomes the destination office because that office becomes responsible for the next action
- immutable event payload must include both `from_office_id` and `to_office_id`

For `IN_TRANSIT -> ACCEPTED`:

- this represents receiving the shipment at the destination office
- `to_office_id` should be omitted in final MVP because the destination is already the responsible office in the snapshot
- if accepted for compatibility, it must equal the current snapshot office

## 5.3 Cancellation Rules

Allowed cancellation sources:

- `NEW`
- `ACCEPTED`
- `PROCESSED`
- `IN_TRANSIT`

Forbidden:

- cancelling `DELIVERED`
- cancelling `CANCELLED`
- reopening a cancelled shipment

Snapshot behavior on cancellation:

- `current_status = CANCELLED`
- `current_office_id` remains the last responsible office

## 5.4 Validation Rules for Transitions

Required validations before a transition is committed:

- shipment exists
- shipment is not terminal
- target status is a valid enum
- target transition is allowed by the state machine
- target office rules are satisfied
- actor is authorized for the shipment’s current responsible office unless admin
- destination office exists and is active when required

Recommended final MVP request rule:

- `to_office_id` is required only when `to_status = IN_TRANSIT`
- `to_office_id` is forbidden for all other transitions

Current reality:

- the current code accepts `to_office_id` on more transitions than necessary

Recommended fix:

- tighten the API contract to reduce ambiguity

## 5.5 Data Captured Per Transition

Every shipment transition must capture:

- `shipment_id`
- `from_status`
- `to_status`
- `occurred_at`
- `actor_user_id`
- `actor_display_name` in audit log
- `from_office_id`
- `to_office_id`
- `notes` if provided

Where each datum lives:

- `shipments`: current status and current responsible office only
- `shipment_status_history`: report-friendly projection row
- `packages`: immutable detailed event payload
- `audit_events`: operational cross-entity audit row

## 5.6 Relational Projection Updates

Recommended final MVP transaction order for `POST /shipments/{id}/status`:

1. lock shipment snapshot row
2. validate actor, state, office, and target data
3. append immutable package to the shipment stream
4. insert `shipment_status_history` row
5. update `shipments.current_status` and `shipments.current_office_id`
6. insert `audit_events` row
7. commit

Reason for this order:

- the immutable event should exist before projection rows reflect it
- the whole unit must be atomic

Current reality:

- event append, snapshot update, history insert, and audit insert are not wrapped in one end-to-end transaction

Recommended fix:

- refactor event-store append to accept an existing DB transaction

## 5.7 Immutable Event Append Behavior

Each successful shipment create or transition appends exactly one business package.

Package guarantees:

- `seq` increments by 1 within the stream
- `prev_hash` matches the prior `head_hash`
- `streams.head_hash` is updated to the new package hash
- package payload is stored as SCB and can be decoded

No package is appended when:

- validation fails
- authorization fails
- the transition is forbidden

## 5.8 Timeline Read Behavior

Timeline API returns packages ordered by ascending `seq`.

Recommended final MVP default:

- hide the stream-initialization package from normal business UI
- show only business events by default
- allow an optional debug mode to reveal the raw full chain

Timeline item fields to expose:

- `seq`
- `event_type`
- `hash`
- `prev_hash`
- `created_at`
- decoded `payload`
- `scb_base64` for diploma demonstration/debug

Current reality:

- current API returns `seq`, `event_type`, `scb`, and decoded payload, but not `hash`, `prev_hash`, or `created_at`

Recommended fix:

- extend the timeline DTO

## 5.9 Example End-to-End Scenarios

### Scenario A: Simple same-office delivery

1. employee in Office A creates shipment for Client X
2. shipment snapshot becomes `NEW`, `current_office_id = A`
3. employee changes `NEW -> ACCEPTED`
4. employee changes `ACCEPTED -> PROCESSED`
5. employee changes `PROCESSED -> IN_TRANSIT` with `to_office_id = A`
6. employee or admin changes `IN_TRANSIT -> DELIVERED`

### Scenario B: Inter-office transfer

1. Office A creates shipment in `NEW`
2. Office A accepts and processes it
3. Office A changes `PROCESSED -> IN_TRANSIT` with destination Office B
4. shipment snapshot now points to Office B as responsible office
5. Office B employee changes `IN_TRANSIT -> ACCEPTED`
6. Office B changes `ACCEPTED -> PROCESSED`
7. Office B changes `IN_TRANSIT -> DELIVERED` after dispatch if it is the delivery office

### Scenario C: Cancellation

1. shipment exists in `ACCEPTED`
2. authorized actor submits `CANCELLED`
3. immutable package is appended
4. history row is appended
5. snapshot becomes `CANCELLED`
6. timeline shows cancellation as the final event

# 6. Authorization Specification

## 6.1 Identity Flow

Authentication authority:

- Auth0

Application authorization authority:

- local PostgreSQL tables `users`, `roles`, and `user_roles`

Recommended login sequence:

1. user authenticates with Auth0
2. frontend receives Auth0 callback and gets bearer token
3. frontend calls `POST /ensure-user` with token-derived identity profile
4. backend upserts or links local `users` row by `auth0_sub` or email
5. frontend calls `GET /me`
6. frontend chooses admin console, employee console, or no-access page based on local roles

## 6.2 Mapping Auth0 Identity to Local Users

Rules:

- Auth0 `sub` maps to `users.auth0_sub`
- if no matching `auth0_sub` exists, the system may link by case-insensitive email if an existing local row has `auth0_sub IS NULL`
- if the same email is already linked to a different `auth0_sub`, the request must fail with conflict
- roles are never assigned automatically during `ensure_user`

Current reality:

- this linking flow is already tested in the API suite

## 6.3 Role Loading

Recommended final MVP:

- load all roles for the local user via `user_roles -> roles`
- resolve `roles[]`
- derive `primary_role` with precedence `admin > employee`

Current reality:

- current `/me` effectively returns one role string

Recommended fix:

- return both `roles[]` and `primary_role`

## 6.4 Role Checks

Authorization is enforced in two places:

- handler/API layer for route-level access
- application/use-case layer for business rules and office scope

Rules:

- unauthenticated request: `401 Unauthorized`
- authenticated but no matching business role: `403 Forbidden`
- authenticated employee operating outside assigned office scope: `403 Forbidden`
- admin bypasses office scope but not domain rules

## 6.5 Valid Auth0 User With No Local Role

Required behavior:

- `/ensure-user` creates or links the `users` row
- `/me` returns the user context with empty role set
- frontend redirects to `/{lang}/app/no-access`
- user cannot access shipment or admin endpoints

## 6.6 Endpoint and Page Role Matrix

Admin only:

- clients CRUD
- offices CRUD
- employees CRUD and office assignment
- reports
- audit events

Admin and employee:

- `/me`
- shipment list/detail/create
- shipment status change
- shipment timeline

Employee console only:

- employee dashboard
- employee shipment pages

## 6.7 Employee Office Assignment Effects

Rules:

- employee can create a shipment only if `current_office_id` is among assigned offices
- employee can change a shipment only if the shipment’s current responsible office is among assigned offices
- employee may send `PROCESSED -> IN_TRANSIT` to any active destination office
- employee may not set non-transit transitions to a different office
- employee with zero assigned offices can authenticate but cannot create or change shipments

# 7. Backend Architecture Specification

## 7.1 Layered Structure

### API / Handler Layer

Responsibilities:

- parse HTTP input
- extract authenticated actor context
- map request DTOs to use-case inputs
- map use-case errors to HTTP status codes and response bodies
- never contain domain transition logic

### Application / Use-Case Layer

Responsibilities:

- orchestrate feature-specific workflows
- enforce authorization decisions not expressible in pure domain rules
- coordinate repositories, event append, and audit emission
- define transaction boundaries

### Domain Layer

Responsibilities:

- pure business rules
- shipment state machine
- transition invariants

### Data / Repository Layer

Responsibilities:

- SeaORM-based relational persistence
- query composition
- soft-delete filtering
- report query execution

### Event / Audit Append Layer

Responsibilities:

- ensure event stream existence
- append Strata packages
- maintain `head_hash`
- insert audit rows
- expose timeline reads

## 7.2 Transaction Boundary Rule

Recommended final MVP:

- each write use-case must run in one database transaction when it updates business data and audit/event data

Critical use-cases:

- create shipment
- change shipment status
- create/update/delete client
- create/update/delete office
- create/update/delete employee
- assign/remove employee office

Reason:

- auditability is weaker if projection rows and immutable packages can diverge

## 7.3 Use-Case Specification by Feature Area

## Users / Me

### `ensure_user`

Inputs:

- `auth0_sub`
- `name`
- `email`

Validation:

- non-empty `auth0_sub`
- valid name and email
- email linking conflict check

Authorization:

- authenticated identity required
- no application role required

DB writes/reads:

- read by `auth0_sub`
- read by email
- insert or update `users`

Audit behavior:

- no business audit event required for MVP

Output DTO:

- `204 No Content`

### `get_me`

Inputs:

- authenticated request context

Validation:

- user must exist locally

Authorization:

- authenticated only

DB reads:

- `users`
- `user_roles`
- `employees`
- `employee_offices`

Audit behavior:

- none

Output DTO:

- local user identity summary
- `roles[]`
- `primary_role`
- `employee_id`
- `office_ids`
- `offices[]` for assigned office summaries

## Clients

### `list_clients`

Inputs:

- optional search/include_deleted filters

Validation:

- filter sanity only

Authorization:

- admin only

DB reads:

- `clients` where active unless explicitly included

Audit behavior:

- none

Output DTO:

- array of client summaries

### `create_client`

Inputs:

- `name`
- `phone`
- `email`

Validation:

- name length
- email format if present
- phone format if present
- at least one contact method required in final MVP

Authorization:

- admin only

DB writes:

- insert `clients`

Audit behavior:

- insert `audit_events` with `client.created`

Output DTO:

- created client object

### `get_client`

Inputs:

- `client_id`

Validation:

- valid UUID

Authorization:

- admin only

DB reads:

- active `clients` row by id

Audit behavior:

- none

Output DTO:

- full client object

### `update_client`

Inputs:

- `client_id`
- full replacement fields for `PUT`

Validation:

- same as create
- active row must exist

Authorization:

- admin only

DB writes:

- update `clients`

Audit behavior:

- insert `audit_events` with `client.updated`

Output DTO:

- updated client object

## Offices

Use-cases mirror clients, with office-specific validation and office references in audit.

Additional rule:

- deactivation must be blocked if active shipments or employee assignments still depend on the office

## Employees

### `list_employees`

Inputs:

- optional include_inactive

Authorization:

- admin only

DB reads:

- active `employees`
- joined `users`
- `employee_offices`

Audit behavior:

- none

Output DTO:

- employee summary with office assignments

### `create_employee`

Inputs:

- `email` of an existing local user
- optional `office_ids[]`

Validation:

- matching user must exist
- user must not already have another active employee row
- office ids must exist if provided

Authorization:

- admin only

DB writes:

- insert or reactivate `employees`
- ensure `employee` role link
- insert `employee_offices` links if provided

Audit behavior:

- `employee.created`
- `employee.assigned_to_office` for each initial assignment

Output DTO:

- employee detail

### `get_employee`

Inputs:

- `employee_id`

Authorization:

- admin only

DB reads:

- active employee + user + office assignments

Audit behavior:

- none

### `update_employee`

Recommended final MVP meaning:

- update active/inactive state and office assignment metadata only

Inputs:

- `employee_id`
- `is_active`
- optional `office_ids[]`

Authorization:

- admin only

DB writes:

- update `deleted_at` when activation state changes
- replace office assignments if `office_ids[]` supplied
- ensure only employee role is added/removed

Audit behavior:

- `employee.updated`

Output DTO:

- employee detail

Current reality:

- current implementation does not yet support meaningful employee updates

## Shipments

### `list_shipments`

Inputs:

- optional filters: status, office, client, created range

Validation:

- filter types only

Authorization:

- admin sees all
- employee sees only office-scoped shipments

DB reads:

- `shipments`
- joins to `clients` and `offices` if response is expanded

Audit behavior:

- none

Output DTO:

- shipment summary list

### `create_shipment`

Inputs:

- `client_id`
- `current_office_id`
- optional `notes`

Validation:

- active client exists
- active office exists
- office scope for employee

Authorization:

- admin or employee

DB writes:

- insert `shipments`
- insert `shipment_status_history`
- ensure `streams`
- append `packages`
- insert `audit_events`

Output DTO:

- created shipment summary

### `get_shipment`

Inputs:

- `shipment_id`

Authorization:

- admin or office-scoped employee

DB reads:

- `shipments`
- `clients`
- `offices`

Audit behavior:

- none

Output DTO:

- shipment detail snapshot

### `change_shipment_status`

Inputs:

- `shipment_id`
- `to_status`
- optional `to_office_id`
- optional `notes`

Validation:

- state machine
- office requirements
- employee office scope

Authorization:

- admin or office-scoped employee

DB writes:

- append immutable package
- insert history row
- update shipment snapshot
- insert audit row

Output DTO:

- `204 No Content` or updated shipment summary; recommended MVP keeps `204` for simplicity

## Reports

### `shipments_by_status`

- read-only aggregated query
- admin only
- reads snapshot table and optional history date filters
- no audit event

### `shipments_by_office`

- read-only aggregated query
- admin only
- reads snapshot table joined with offices

### `shipments_by_client`

- read-only aggregated query
- admin only
- reads snapshot table joined with clients

### `shipments_by_period`

- read-only aggregated query
- admin only
- groups shipments by `created_at`

## Timeline

### `read_timeline`

Inputs:

- `shipment_id`
- optional `include_meta`

Authorization:

- admin or office-scoped employee

DB reads:

- `packages` ordered by `seq`

Audit behavior:

- none

Output DTO:

- ordered immutable timeline items

## Audit Log

### `list_audit_events`

Inputs:

- pagination cursor
- optional filters by actor, action, entity, office, period

Authorization:

- admin only

DB reads:

- `audit_events`

Audit behavior:

- listing audit does not itself emit audit in MVP

Output DTO:

- paginated audit page

# 8. REST API Specification

## 8.1 API Shape Decision

Current reality:

- shipment endpoints already exist at root paths such as `/shipments`
- admin CRUD endpoints currently live under `/admin/clients`, `/admin/offices`, `/admin/employees`, `/admin/audit`

Recommended final MVP:

- expose the business API at the root resource paths listed below
- enforce role checks server-side rather than encoding admin-ness in the URL
- keep backward-compatible `/admin/*` aliases only if needed during transition

## 8.2 Common Conventions

Authentication:

- bearer token from Auth0

Content type:

- JSON for request and response bodies

Date format:

- ISO-8601 UTC timestamps

Identifiers:

- UUID strings

Errors:

- see section 13

## 8.3 `GET /me`

Purpose:

- return the authenticated user’s local application context

Required role:

- authenticated only

Response:

- `200 OK`
- body:
  - `user: { id, auth0_sub, name, email }`
  - `roles: string[]`
  - `primary_role: "admin" | "employee" | null`
  - `employee_id: string | null`
  - `office_ids: string[]`
  - `offices: [{ id, name, city, address }]`
  - `current_office_id: string | null`

Failure cases:

- `401` invalid/missing token
- `404` local user not provisioned

Notes:

- current implementation returns only a single role string and office ids; final MVP should expand this

## 8.4 Clients

### `GET /clients`

Purpose:

- list active clients

Required role:

- admin

Query params:

- `search?: string`
- `include_deleted?: boolean` default `false`

Success response:

- `200 OK`
- `{ items: [{ id, name, phone, email, created_at, updated_at, deleted_at? }] }`

Failure cases:

- `403`

Notes:

- if `include_deleted=false`, soft-deleted rows are excluded

### `POST /clients`

Purpose:

- create client

Required role:

- admin

Request body:

- `{ name, phone?: string | null, email?: string | null }`

Success response:

- `201 Created`
- `{ item: { id, name, phone, email, created_at, updated_at } }`

Failure cases:

- `400` validation
- `403`
- `409` duplicate if a future uniqueness rule is added

Notes:

- final MVP requires at least one contact method: phone or email

### `GET /clients/{id}`

Purpose:

- fetch one client

Required role:

- admin

Success response:

- `200 OK`
- `{ item: { id, name, phone, email, created_at, updated_at, deleted_at? } }`

Failure cases:

- `400` invalid UUID
- `403`
- `404`

### `PUT /clients/{id}`

Purpose:

- update client

Required role:

- admin

Request body:

- `{ name, phone?: string | null, email?: string | null }`

Success response:

- `200 OK`
- `{ item: { ...updated client... } }`

Failure cases:

- `400`
- `403`
- `404`
- `409`

Notes:

- `PUT` is full replacement at the field level for editable properties

## 8.5 Offices

### `GET /offices`

Purpose:

- list active offices

Required role:

- admin

Query params:

- `search?: string`
- `city?: string`
- `include_deleted?: boolean`

Success response:

- `{ items: [{ id, name, city, address, created_at, updated_at, deleted_at? }] }`

### `POST /offices`

Purpose:

- create office

Required role:

- admin

Request body:

- `{ name, city, address }`

Success response:

- `201 Created`
- `{ item: { id, name, city, address, created_at, updated_at } }`

Failure cases:

- `400`
- `403`
- `409` exact duplicate active office

### `GET /offices/{id}`

Required role:

- admin

Success response:

- `{ item: { id, name, city, address, created_at, updated_at, deleted_at? } }`

Failure cases:

- `400`
- `403`
- `404`

### `PUT /offices/{id}`

Purpose:

- update office

Required role:

- admin

Request body:

- `{ name, city, address }`

Success response:

- `{ item: { ...updated office... } }`

Failure cases:

- `400`
- `403`
- `404`
- `409`

## 8.6 Employees

### `GET /employees`

Purpose:

- list active employees

Required role:

- admin

Query params:

- `include_inactive?: boolean`

Success response:

- `{ items: [{ id, user_id, name, email, office_ids, offices, created_at, updated_at, deleted_at? }] }`

### `POST /employees`

Purpose:

- create or reactivate an employee for an existing local user

Required role:

- admin

Request body:

- `{ email: string, office_ids?: string[] }`

Success response:

- `201 Created`
- `{ item: { id, user_id, name, email, office_ids, offices, created_at, updated_at } }`

Failure cases:

- `400`
- `403`
- `404` user not provisioned
- `409` conflicting active employee

Notes:

- the employee user must already exist in `users`, usually because they logged in once through Auth0

### `GET /employees/{id}`

Purpose:

- fetch employee detail

Required role:

- admin

Success response:

- `{ item: { id, user_id, name, email, office_ids, offices, created_at, updated_at, deleted_at? } }`

Failure cases:

- `400`
- `403`
- `404`

### `PUT /employees/{id}`

Purpose:

- update employee activation state and optionally office assignments

Required role:

- admin

Request body:

- `{ is_active: boolean, office_ids?: string[] }`

Success response:

- `{ item: { ...updated employee... } }`

Failure cases:

- `400`
- `403`
- `404`
- `409`

Notes:

- current code does not yet implement this final behavior

### `PUT /employees/{id}/offices`

Purpose:

- replace the full office assignment set for an employee

Required role:

- admin

Request body:

- `{ office_ids: string[] }`

Success response:

- `200 OK`
- `{ employee_id, office_ids, offices }`

Failure cases:

- `400`
- `403`
- `404` employee or office not found

Notes:

- current reality uses nested `POST` and `DELETE` under `/admin/employees/{id}/offices`
- recommended final MVP replaces that with one idempotent set-replacement endpoint

## 8.7 Shipments

### `GET /shipments`

Purpose:

- list shipments visible to the actor

Required role:

- admin or employee

Query params:

- `status?: shipment_status`
- `office_id?: uuid`
- `client_id?: uuid`
- `created_from?: timestamp`
- `created_to?: timestamp`

Success response:

- `200 OK`
- `{ items: [{ id, client_id, client_name, current_status, current_office_id, current_office_name, created_at, updated_at }] }`

Failure cases:

- `403`

Notes:

- employee result set is office-scoped automatically

### `POST /shipments`

Purpose:

- create shipment

Required role:

- admin or employee

Request body:

- `{ client_id: uuid, current_office_id: uuid, notes?: string | null }`

Success response:

- `201 Created`
- `{ item: { id, client_id, current_status: "NEW", current_office_id, created_at, updated_at } }`

Failure cases:

- `400`
- `403`
- `404` client or office missing
- `409` if a future duplicate business rule is added

Notes:

- final MVP requires `current_office_id`

### `GET /shipments/{id}`

Purpose:

- fetch shipment snapshot detail

Required role:

- admin or employee

Success response:

- `{ item: { id, client: { id, name }, current_status, current_office: { id, name, city, address } | null, created_at, updated_at } }`

Failure cases:

- `400`
- `403`
- `404`

Notes:

- employee access depends on office scope

### `POST /shipments/{id}/status`

Purpose:

- apply one shipment transition

Required role:

- admin or employee

Request body:

- `{ to_status: shipment_status, to_office_id?: uuid | null, notes?: string | null }`

Success response:

- `204 No Content`

Failure cases:

- `400` invalid status, invalid transition, invalid office usage
- `403` office-scope violation
- `404` shipment not found
- `409` optional if invalid state is modeled as conflict instead of bad request

Notes:

- `to_office_id` is required only when transitioning to `IN_TRANSIT`
- `to_office_id` must be omitted for all other statuses in the final contract

### `GET /shipments/{id}/timeline`

Purpose:

- return immutable timeline packages for one shipment

Required role:

- admin or employee

Query params:

- `include_meta?: boolean` default `false`
- `include_scb?: boolean` default `true`

Success response:

- `{ items: [{ seq, event_type, hash, prev_hash, created_at, payload, scb_base64? }] }`

Failure cases:

- `400`
- `403`
- `404`

Notes:

- employee authorization uses the same shipment visibility rule as `GET /shipments/{id}`

## 8.8 Reports

All report endpoints are admin-only.

### `GET /reports/shipments-by-status`

Purpose:

- count shipments by current status

Query params:

- `office_id?: uuid`
- `client_id?: uuid`
- `created_from?: timestamp`
- `created_to?: timestamp`

Success response:

- `{ items: [{ status, shipment_count }], total_shipments }`

### `GET /reports/shipments-by-office`

Purpose:

- count shipments by current responsible office

Query params:

- `status?: shipment_status`
- `client_id?: uuid`
- `created_from?: timestamp`
- `created_to?: timestamp`

Success response:

- `{ items: [{ office_id, office_name, city, shipment_count }] }`

### `GET /reports/shipments-by-client`

Purpose:

- count shipments by client

Query params:

- `status?: shipment_status`
- `office_id?: uuid`
- `created_from?: timestamp`
- `created_to?: timestamp`

Success response:

- `{ items: [{ client_id, client_name, shipment_count }] }`

### `GET /reports/shipments-by-period`

Purpose:

- count shipments created in time buckets

Query params:

- `from: timestamp`
- `to: timestamp`
- `bucket?: "day" | "week" | "month"` default `day`
- `status?: shipment_status`
- `office_id?: uuid`
- `client_id?: uuid`

Success response:

- `{ items: [{ period_start, period_end, shipment_count }] }`

## 8.9 Audit

### `GET /audit-events`

Purpose:

- list operational audit events

Required role:

- admin

Query params:

- `limit?: number` default `50`, max `100`
- `cursor?: string`
- `action_key?: string`
- `entity_type?: string`
- `entity_id?: string`
- `office_id?: uuid`
- `actor_user_id?: uuid`
- `from?: timestamp`
- `to?: timestamp`

Success response:

- `{ items: [{ id, occurred_at, actor_user_id, actor_display_name, action_key, entity_type, entity_id, entity_label, office_id, office_label, target_route, metadata_json, request_id }], page: { limit, next_cursor, has_next } }`

Failure cases:

- `400` invalid cursor/filter
- `403`

Notes:

- current reality exposes this at `/admin/audit`

# 9. Frontend / UI Specification

## 9.1 Route Map

Recommended MVP route map:

| Route                                      | Role                   | Purpose                               |
| ------------------------------------------ | ---------------------- | ------------------------------------- |
| `/{lang}/login`                            | public                 | start Auth0 login                     |
| `/callback`                                | public                 | Auth0 callback processing             |
| `/{lang}/app`                              | authenticated          | role cutover route                    |
| `/{lang}/app/no-access`                    | authenticated, no role | no-access explanation                 |
| `/{lang}/app/admin`                        | admin                  | admin dashboard                       |
| `/{lang}/app/admin/clients`                | admin                  | client list                           |
| `/{lang}/app/admin/clients/new`            | admin                  | client create                         |
| `/{lang}/app/admin/clients/{id}`           | admin                  | client detail/edit                    |
| `/{lang}/app/admin/offices`                | admin                  | office list                           |
| `/{lang}/app/admin/offices/new`            | admin                  | office create                         |
| `/{lang}/app/admin/offices/{id}`           | admin                  | office detail/edit                    |
| `/{lang}/app/admin/employees`              | admin                  | employee list                         |
| `/{lang}/app/admin/employees/new`          | admin                  | employee create                       |
| `/{lang}/app/admin/employees/{id}`         | admin                  | employee detail/edit                  |
| `/{lang}/app/admin/employees/{id}/offices` | admin                  | office assignment helper page if kept |
| `/{lang}/app/admin/shipments`              | admin                  | shipment list                         |
| `/{lang}/app/admin/shipments/new`          | admin                  | shipment create                       |
| `/{lang}/app/admin/shipments/{id}`         | admin                  | shipment detail and timeline          |
| `/{lang}/app/admin/reports`                | admin                  | reports page                          |
| `/{lang}/app/admin/audit`                  | admin                  | audit events page                     |
| `/{lang}/app/employee`                     | employee               | employee dashboard                    |
| `/{lang}/app/employee/shipments`           | employee               | shipment list                         |
| `/{lang}/app/employee/shipments/new`       | employee               | shipment create                       |
| `/{lang}/app/employee/shipments/{id}`      | employee               | shipment detail and timeline          |
| `/{lang}/app/employee/profile`             | employee               | profile summary                       |

Current reality:

- most of these routes already exist except reports

## 9.2 Console Separation

Admin console navigation:

- dashboard
- shipments
- clients
- offices
- employees
- reports
- audit log
- profile

Employee console navigation:

- dashboard
- shipments
- profile

Rule:

- if `primary_role = admin`, admin console is the default landing area
- if `primary_role = employee`, employee console is the default landing area
- if no role, redirect to no-access

## 9.3 Login Flow

Required behavior:

1. unauthenticated visit to any app page redirects to `/{lang}/login`
2. login starts Auth0 authorization flow
3. callback exchanges code for tokens
4. callback calls `/ensure-user`
5. callback calls `/me`
6. UI stores encrypted session cookie
7. user is redirected to admin, employee, or no-access route

Failure states:

- token exchange failed
- Auth0 profile incomplete
- email conflict on user linking
- local role load failed

UI response:

- redirect to localized auth error page with a clear message and retry link

## 9.4 Dashboard

### Admin dashboard

Must show:

- shipment counts by current status
- active office count
- active employee count
- active client count
- quick links to shipments, reports, and audit

Data source:

- existing list/report endpoints or dedicated lightweight dashboard endpoints if later added

### Employee dashboard

Must show:

- assigned office list
- shipments currently in employee scope
- quick actions: create shipment, open shipment list

## 9.5 Clients UI

### Clients list

Behavior:

- searchable table
- columns: name, email, phone, updated_at
- empty state: “No clients yet”
- error state with reload action

### Create client

Behavior:

- form fields: name, email, phone
- inline validation
- on success redirect to detail/edit page or list with success banner

### Edit client

Behavior:

- same fields prefilled
- save button disabled while pending
- clear not-found state if client is missing or inactive

## 9.6 Offices UI

Same pattern as clients, with fields:

- name
- city
- address

Additional delete/deactivate warning:

- if the office is used by active shipments or employee assignments, UI must show a blocking message instead of offering destructive action

## 9.7 Employees UI

### Employees list

Columns:

- name
- email
- assigned offices
- status active/inactive

### Create employee

Fields:

- user email
- optional initial office assignment multi-select

Behavior:

- if no matching provisioned user exists, show clear error: user must log in once first

### Edit employee

Fields:

- active/inactive toggle
- office assignment editor

### Office assignment

Behavior:

- multi-select list of active offices
- save sends full replacement to `PUT /employees/{id}/offices`
- duplicate office selections prevented client-side

## 9.8 Shipments UI

### Shipments list

Columns:

- shipment id
- client
- current status
- current office
- updated_at

Filters:

- status
- office
- client
- date range

Role behavior:

- admin sees all shipments
- employee sees only office-scoped shipments

States:

- loading skeleton
- empty list state
- API error state

### Create shipment

Fields:

- client selector
- current office selector
- notes

Role behavior:

- employee office selector is restricted to assigned offices
- admin office selector lists all active offices

### Shipment detail

Must show:

- core snapshot fields
- status badge
- client summary
- current responsible office summary
- transition form
- status history table
- immutable timeline panel

Transition form behavior:

- only valid next actions should be selectable
- when target status is `IN_TRANSIT`, destination office field becomes required
- for all other transitions, destination office field is hidden

### Shipment detail timeline behavior

UI structure:

- status history table first for business readability
- immutable package chain below it

Immutable package row fields:

- sequence number
- event type
- created_at
- hash and previous hash
- decoded payload
- optional expand-to-show raw SCB base64

## 9.9 Reports Page

Role:

- admin only

Structure:

- filter panel at top
- report selector tabs or segmented control:
  - by status
  - by office
  - by client
  - by period
- results rendered as tables
- summary total displayed above each table

Behavior:

- changing filters refetches current report
- empty state is explicit, not blank

## 9.10 Audit Events Page

Role:

- admin only

Columns:

- occurred_at
- actor
- action_key
- entity
- office
- target route

Behavior:

- cursor-based pagination
- filters for action, entity, office, period
- metadata drawer for JSON details

## 9.11 No-Access Page

Purpose:

- explain that the user authenticated successfully but lacks an application role

Content:

- concise explanation
- logged-in user email/name
- instruction to contact an administrator
- logout button

## 9.12 Loading, Empty, Error, and No-Access States

Required UI rules:

- all list pages need explicit loading, empty, and error states
- all detail pages need explicit not-found handling
- `403` within a console should render a localized forbidden page with a link back to the correct dashboard
- no page should silently fail or render blank content

# 10. Reporting Specification

Reports are relational by design. The MVP must not require SCB decoding for report generation.

## 10.1 Why Relational History Exists

`shipment_status_history` exists specifically so the diploma system can satisfy reporting and SQL-query requirements using normal relational patterns:

- `JOIN`
- `GROUP BY`
- aggregate functions
- date filtering
- subqueries

Strata packages remain the immutable source for auditability, not the primary report source.

## 10.2 Shipments by Status

Purpose:

- show current operational distribution of shipments

Primary source tables:

- `shipments`

Optional joins:

- `offices`
- `clients`

Filters:

- office
- client
- created date range

Aggregation:

- `GROUP BY shipments.current_status`

Expected result columns:

- `status`
- `shipment_count`

Role restriction:

- admin only

UI rendering:

- compact summary table and optional top summary cards

Example SQL approach:

- filter from `shipments`
- apply optional predicates
- group by `current_status`

## 10.3 Shipments by Office

Purpose:

- show how many shipments are currently assigned to each office

Primary source tables:

- `shipments`
- `offices`

Filters:

- current status
- client
- created date range

Aggregation:

- `GROUP BY offices.id, offices.name, offices.city`

Expected result columns:

- `office_id`
- `office_name`
- `city`
- `shipment_count`

Role restriction:

- admin only

UI rendering:

- table sorted descending by shipment count

Example SQL approach:

- join `shipments.current_office_id = offices.id`
- filter out soft-deleted offices unless historical mode is requested
- group by office columns

## 10.4 Shipments by Client

Purpose:

- show shipment volume per client

Primary source tables:

- `shipments`
- `clients`

Filters:

- office
- status
- created date range

Aggregation:

- `GROUP BY clients.id, clients.name`

Expected result columns:

- `client_id`
- `client_name`
- `shipment_count`

Role restriction:

- admin only

UI rendering:

- sortable table

Example SQL approach:

- join `shipments.client_id = clients.id`
- group by client identifiers

## 10.5 Shipments by Period

Purpose:

- show shipment creation volume over time

Primary source tables:

- `shipments`

Filters:

- `from`
- `to`
- bucket size `day|week|month`
- optional status, office, client

Aggregation:

- `GROUP BY date_trunc(bucket, shipments.created_at)`

Expected result columns:

- `period_start`
- `period_end`
- `shipment_count`

Role restriction:

- admin only

UI rendering:

- table ordered by ascending period

Example SQL approach:

- date filter in `WHERE`
- `date_trunc`
- `GROUP BY` truncated timestamp

## 10.6 Optional History-Based Analytical Queries

Useful but secondary diploma queries can also use `shipment_status_history`:

- count transitions to `DELIVERED` by month
- count cancellations by office
- latest transition per shipment using subquery or window function

These queries satisfy diploma expectations for more advanced SQL without needing Strata decoding.

## 10.7 SQL Patterns That Satisfy Diploma Expectations

Required relational patterns already supported by this model:

- `JOIN`: shipments to clients/offices, employees to users/offices
- `GROUP BY`: report aggregations
- subqueries: latest shipment action, delivered count in date range, inactive master-data checks
- filtered aggregates: counts by current status or transition target

# 11. Audit and Event Store Specification

## 11.1 What Goes Into `shipment_status_history`

`shipment_status_history` stores the relational transition projection.

Each row must contain:

- `shipment_id`
- `from_status`
- `to_status`
- `changed_at`
- `actor_user_id`
- `notes`
- `from_office_id` and `to_office_id` in the recommended final shape

Use cases:

- shipment detail status table
- report queries
- quick DB inspection

It is not the cryptographic source of truth.

## 11.2 What Goes Into `audit_events`

`audit_events` stores operational audit rows across the entire system.

Each row should capture:

- who acted
- when
- what action occurred
- which entity was affected
- office context if relevant
- route or UI target for drill-down
- metadata snapshot
- request correlation id

Examples:

- client created
- office updated
- employee assigned to office
- shipment status updated

Use cases:

- admin audit page
- operator accountability
- change review during diploma demo

## 11.3 What Goes Into Strata Packages

Packages store immutable shipment events only.

Recommended business payloads:

- `shipment.created`
  - shipment id
  - initial status
  - actor user id
  - office id
  - notes
- `shipment.status_changed`
  - shipment id
  - from status
  - to status
  - actor user id
  - from office id
  - to office id
  - notes
  - occurred_at

Packages must not be used for mutable master data such as client edits or office edits in MVP.

## 11.4 Stream and Package Chaining

Per shipment:

- create one stream
- append packages in order
- each package stores `prev_hash`
- `streams.head_hash` always points to the last package hash

Required invariants:

- no duplicate `(stream_id, seq)`
- `seq` strictly increases
- `prev_hash` of the first package is null
- every later package references the prior head

## 11.5 How `head_hash` Is Maintained

Write algorithm:

1. lock stream row
2. read current `head_hash`
3. compute next `seq`
4. insert package with `prev_hash = old head_hash`
5. update `streams.head_hash = new package hash`
6. commit

## 11.6 When `audit_events` Are Written vs Package Appends

Recommended final MVP:

- both happen within the same use-case transaction for shipment writes

Behavior:

- shipment create/status change:
  - append package
  - write history row
  - update snapshot
  - write audit row
- admin CRUD:
  - no package append
  - only audit row

## 11.7 Operational Audit Log vs Immutable Shipment Timeline

They are different on purpose.

`audit_events`:

- broad system audit
- includes clients, offices, employees, shipments
- optimized for admin browsing and filtering
- not chained cryptographically

Strata shipment timeline:

- shipment-only
- immutable chained event stream
- optimized for proving transition history
- exposes low-level package information

Why both are required:

- one supports system-wide operational oversight
- the other supports immutable shipment traceability

## 11.8 How the UI Consumes Timeline Data

UI should call:

- `GET /shipments/{id}` for current snapshot
- `GET /shipments/{id}/timeline` for immutable package chain

UI then renders:

- a business-friendly transition history
- a raw immutable package section for proof/debug

Recommended final MVP:

- do not reconstruct all shipment screens from SCB alone
- keep the UI powered primarily by relational snapshot data

# 12. Validation and Business Rules

## 12.1 Clients

Required:

- `name`

Allowed values:

- trimmed length 2..100

Optional:

- `email`
- `phone`

Rules:

- at least one of `email` or `phone` must be present in final MVP
- email max 254 chars, one `@`, no spaces, domain contains a dot
- phone must be `+` followed by 8..15 digits

Soft-delete consideration:

- cannot update a soft-deleted client through normal endpoints

Duplicate prevention:

- no hard DB uniqueness beyond optional search/warning
- UI should warn on likely duplicates by name + contact

## 12.2 Offices

Required:

- `name`
- `city`
- `address`

Rules:

- `name` 2..100
- `city` 2..100
- `address` 2..200

Soft-delete consideration:

- cannot update soft-deleted office
- cannot deactivate office while active shipments or employee assignments depend on it

Duplicate prevention:

- exact duplicate active office should be rejected

## 12.3 Employees

Create rules:

- `email` required
- email must match an existing local user
- user may have at most one active employee row

Update rules:

- only activation state and office assignments are editable in MVP

Soft-delete consideration:

- inactive employee disappears from active lists
- employee office links must be removed or ignored
- only the `employee` role should be removed

Duplicate prevention:

- unique active employee per `user_id`

## 12.4 Shipments

Create rules:

- `client_id` required and must reference active client
- `current_office_id` required and must reference active office
- initial status always `NEW`
- notes optional, recommended max length 500

Mutation rules:

- shipment cannot be deleted in MVP
- current status may change only through the transition endpoint

## 12.5 Shipment Status Updates

Rules:

- `to_status` required and must be one of the allowed statuses
- terminal states reject all changes
- `to_office_id` required only for `IN_TRANSIT`
- `to_office_id` forbidden otherwise
- non-admin actor must be assigned to the shipment’s current responsible office
- destination office must exist and be active

Invalid transition handling:

- reject with structured error
- do not append package
- do not change snapshot
- do not write history row
- do not write audit row

## 12.6 Role Assignment

Rules:

- only `admin` and `employee` are valid MVP role names
- assigning `employee` may happen implicitly when creating/reactivating employee
- no role is auto-granted during Auth0 provisioning

## 12.7 Office Assignment

Rules:

- only active employees may hold active office assignments
- only active offices may be assigned
- duplicate assignment is forbidden
- empty assignment set is allowed but disables employee shipment actions

## 12.8 Authorization Failures

Rules:

- unauthenticated: `401`
- authenticated but wrong role: `403`
- employee outside office scope: `403`

# 13. Error Handling Specification

## 13.1 Error Model

Recommended final MVP response body:

- `code: string`
- `message: string`
- `field_errors?: [{ field, code, message }]`
- `request_id?: string`

The current simple `{ code, message }` model may remain, but field-level details should be added for form UX.

## 13.2 Error Categories

Validation error:

- HTTP `400`
- invalid field values, missing required field, invalid cursor, malformed UUID

Unauthorized:

- HTTP `401`
- missing or invalid Auth0 token

Forbidden:

- HTTP `403`
- role failure or office-scope failure

Not found:

- HTTP `404`
- entity missing or soft-deleted from normal view

Conflict:

- HTTP `409`
- email already linked
- duplicate office assignment
- duplicate active office definition
- optional invalid-state conflicts if modeled that way

Internal error:

- HTTP `500`
- unexpected DB, event-store, decode, or infrastructure failure

## 13.3 API Semantics

Rules:

- never return HTML from API errors
- every error must include a machine-readable code
- request correlation id should be logged and returned when available

## 13.4 UI Handling Expectations

Form pages:

- show inline validation messages

List pages:

- show retryable error panel

Detail pages:

- distinguish `404` from generic error

No-access cases:

- redirect to no-access page when role is missing
- show forbidden page for wrong-console direct access

# 14. Testing Specification

## 14.1 Unit Tests

Must cover:

- shipment status enum parsing and display
- state machine allowed and forbidden transitions
- office-hop validation
- client validation
- office validation
- user/email linking validation

Priority:

- highest on shipment state machine correctness

## 14.2 Integration Tests

Must cover:

- repository CRUD for clients, offices, employees
- employee office assignment repository rules
- audit repository pagination
- event-store append rules:
  - strict `seq`
  - correct `prev_hash`
  - `head_hash` update

## 14.3 API Tests

Must cover:

- Auth0/dev auth extraction
- `/ensure-user`
- `/me`
- admin-only CRUD routes
- employee vs admin shipment visibility
- shipment create and status change endpoints
- timeline endpoint ordering and payload decoding

## 14.4 Report Query Tests

Must cover seeded datasets where expected aggregates are known for:

- shipments by status
- shipments by office
- shipments by client
- shipments by period

Each test should assert:

- filters work
- soft-deleted master data does not leak into active result sets unless requested
- counts are stable and deterministic

## 14.5 Authorization Tests

Must cover:

- admin access to admin endpoints
- employee rejection from admin endpoints
- no-role user rejection from business endpoints
- employee office-scope restrictions on create and transition

## 14.6 Shipment Transition Consistency Tests

Must cover:

- invalid transition does not mutate snapshot
- invalid transition does not append packages
- invalid transition does not write history
- invalid transition does not write audit
- valid transition updates all projection layers consistently

## 14.7 End-to-End Flow Tests

Recommended high-value E2E scenarios:

- first Auth0 login -> ensure user -> no-access
- admin creates client, office, employee, and assigns office
- employee creates shipment and advances it
- admin views immutable timeline and audit row
- reports reflect seeded shipment activity

# 15. Non-Functional Requirements

## Security

- authentication delegated to Auth0
- authorization enforced locally from DB roles
- session cookie in web UI must be encrypted and `HttpOnly`
- API never trusts client-supplied role claims beyond validated token identity
- input must be validated server-side even if UI already validated it

## Auditability

- shipment transitions must produce immutable packages
- business-changing actions must produce audit rows
- audit data must be append-only
- timeline must expose enough metadata to explain who changed what and where

## Maintainability

- keep layered boundaries strict
- keep state machine logic in domain layer
- keep SQL/report logic in repository layer
- avoid duplicating authorization logic in many handlers

## Responsiveness

MVP targets:

- list and detail pages should respond within about 2 seconds on typical local/demo data sizes
- report endpoints should remain responsive for low tens of thousands of shipments

## Data Consistency

- shipment create and transition writes must be atomic
- projection and immutable timeline must not diverge on successful writes
- soft-deleted rows must not appear in active CRUD screens

## Scalability Assumptions for MVP

- single PostgreSQL instance
- low-to-moderate office and shipment counts
- pagination required only for audit events in MVP

## Observability and Logging

- every HTTP request should have a request id
- log route, actor user id when available, status code, and latency
- log authorization failures and transition rejections
- do not log bearer tokens or raw PII beyond what is necessary for diagnosis

# 16. Open Questions and Design Decisions

## 1. Should shipment support sender and receiver as separate clients?

Status:

- unresolved in current schema

Recommendation:

- no for MVP

Tradeoff:

- keeps implementation and reporting simple now
- defers a likely real-world requirement

## 2. Should `IN_TRANSIT -> ACCEPTED` remain allowed?

Status:

- implemented already

Recommendation:

- yes

Tradeoff:

- makes the lifecycle slightly less linear
- is necessary for multi-office routing

## 3. Should `shipments.current_office_id` be nullable?

Status:

- nullable in schema

Recommendation:

- application-required for all active shipments; migrate to `NOT NULL` if feasible before final delivery

Tradeoff:

- stronger data quality and simpler authorization
- migration may be needed for legacy null rows

## 4. Should `shipment_status_history` store separate from/to offices?

Status:

- currently no

Recommendation:

- yes

Tradeoff:

- small schema evolution now
- much clearer reports and diploma explanation later

## 5. Should history actor reference user or employee?

Status:

- currently user

Recommendation:

- keep user

Tradeoff:

- cleaner auth alignment
- employee-specific reporting requires one extra join

## 6. Should admin CRUD stay under `/admin/*` routes?

Status:

- currently yes

Recommendation:

- no for the final public contract; use resource-root paths and keep admin as a role check

Tradeoff:

- cleaner API surface
- requires small routing refactor or aliases

## 7. Should employee updates remain a no-op timestamp touch?

Status:

- currently yes

Recommendation:

- no; define employee update around activation state and office assignment

Tradeoff:

- clearer product meaning
- requires DTO and service changes

## 8. Should shipment timeline expose raw hashes and timestamps?

Status:

- currently not fully exposed

Recommendation:

- yes

Tradeoff:

- slightly larger payload
- much better diploma demonstration of immutability

## 9. Should delete endpoints be part of MVP?

Status:

- current code has delete endpoints for clients, offices, employees

Recommendation:

- soft-deactivate behavior should exist, but it is secondary to the mandatory list/detail/create/update flows

Tradeoff:

- improves data administration
- adds guard logic around referenced entities

## 10. Should audit writes and package appends be in one transaction?

Status:

- currently not fully

Recommendation:

- yes, explicitly

Tradeoff:

- slightly more plumbing
- avoids partial-write inconsistencies

## 11. Should `/me` return one role or many?

Status:

- currently one effective role string

Recommendation:

- return `roles[]` and `primary_role`

Tradeoff:

- more precise API
- slightly more frontend logic

## 12. Should office deletion be allowed when shipments still reference that office?

Status:

- current soft-delete logic does not block this

Recommendation:

- no

Tradeoff:

- preserves consistent operations
- requires reference checks before deactivation

# 17. Recommended Final MVP Definition

## Absolutely Ships

- Auth0 login plus local user provisioning
- admin and employee role model
- no-access flow for users without role
- client CRUD
- office CRUD
- employee create/list/detail/update activation and office assignment
- shipment create/list/detail
- shipment state transitions with office-scoped employee permissions
- shipment status history projection
- immutable shipment timeline via Strata packages
- audit events page
- four reports:
  - by status
  - by office
  - by client
  - by period

## Deferred

- sender/receiver client split
- hot/db/cold timeline storage layers
- map integration
- notifications
- mobile UI
- heavy DB optimization and archival/GC features

## Nice-to-Have but Not Required for Diploma Success

- CSV export for reports
- advanced dashboard widgets
- richer audit filters
- debug toggle showing raw SCB and package hashes in more places

# 18. Delivery Plan / Vertical Slice Order

## Recommended Build Order

1. Stabilize schema decisions.
   - add any missing columns such as `shipment_status_history.from_office_id` and `to_office_id`
   - normalize role and event-type enums
   - decide whether `shipments.current_office_id` will remain nullable at DB level

2. Finish identity and authorization baseline.
   - `/ensure-user`
   - `/me`
   - role loading with `roles[]` and `primary_role`
   - no-access flow

3. Complete admin master-data slices.
   - clients
   - offices
   - employees
   - office assignment
   - soft-delete guards

4. Finalize shipment creation vertical slice.
   - create shipment
   - create stream
   - append `shipment.created`
   - snapshot/history/audit consistency
   - admin and employee UI forms

5. Finalize shipment transition vertical slice.
   - state machine
   - office-scoped authorization
   - immutable append + history + snapshot update
   - shipment detail timeline page

6. Add reporting endpoints and admin reports page.
   - by status
   - by office
   - by client
   - by period

7. Complete audit log endpoint and admin audit page.
   - pagination
   - filtering
   - metadata viewer

8. Harden error handling and transaction boundaries.
   - structured validation errors
   - request ids
   - single-transaction write use-cases

9. Close testing gaps.
   - report query tests
   - authorization matrix tests
   - projection consistency tests
   - E2E demo-path tests

10. Produce diploma artifacts from the implemented system.

- ER diagram
- UML/use-case diagrams
- SQL appendix from actual report queries
- screenshots of admin, employee, reports, timeline, and audit pages
