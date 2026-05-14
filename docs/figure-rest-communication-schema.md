# Figure: Примерна схема на REST комуникация между клиент и сървър

```mermaid
sequenceDiagram
    autonumber
    participant C as Клиент (Web UI / Browser)
    participant A as LogiPack Hub REST API
    participant D as PostgreSQL Database

    Note over C,A: GET заявка - извличане на данни
    C->>A: GET /shipments
    A->>D: SELECT * FROM shipments
    D-->>A: Списък с пратки
    A-->>C: 200 OK + JSON response

    Note over C,A: POST заявка - създаване на нов ресурс
    C->>A: POST /shipments { client_id, current_office_id, notes }
    A->>D: INSERT INTO shipments ...
    A->>D: INSERT INTO shipment_status_history ...
    D-->>A: Нов запис / потвърждение
    A-->>C: 201 Created + JSON response

    Note over C,A: PUT заявка - обновяване на съществуващ ресурс
    C->>A: PUT /clients/{id} { name, email, phone }
    A->>D: UPDATE clients SET ...
    D-->>A: Обновен запис / потвърждение
    A-->>C: 200 OK + JSON response
```

## Кратко тълкуване

- `GET` заявката се използва за извличане на ресурси без промяна в състоянието на системата.
- `POST` заявката се използва за създаване на нов ресурс, например нова пратка.
- `PUT` заявката се използва за обновяване на вече съществуващ ресурс, например данните за клиент.
- Комуникацията е в `JSON` формат, а сървърът връща стандартни HTTP статус кодове като `200 OK` и `201 Created`.

## Подходящ надпис под фигурата

`Фигура X. Примерна схема на REST комуникация между клиентската част и сървърната част на системата чрез GET, POST и PUT заявки.`
