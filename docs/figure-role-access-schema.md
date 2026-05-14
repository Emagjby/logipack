# Figure: Схема на ролево разграничение между Admin и Employee

```mermaid
flowchart TB
    U["Автентикиран потребител"] --> AUTH["Auth0 идентичност"]
    AUTH --> MAP["Локално ролево картографиране<br/>users -> user_roles -> roles"]

    MAP -->|role = admin| ADMIN["Admin"]
    MAP -->|role = employee| EMP["Employee"]

    subgraph ADMIN_ZONE["Admin Console"]
        direction TB
        A1["Табло"]
        A2["Пратки"]
        A3["Клиенти"]
        A4["Офиси"]
        A5["Служители"]
        A6["Справки"]
        A7["Одитни събития"]
        A8["Профил"]
    end

    subgraph EMP_ZONE["Employee Console"]
        direction TB
        E1["Табло"]
        E2["Пратки"]
        E3["Профил"]
    end

    ADMIN --> ADMIN_ZONE
    EMP --> EMP_SCOPE["Ограничение по офис<br/>employee_offices"]
    EMP_SCOPE --> EMP_ZONE

    A2 --> A2D["Създаване на пратка"]
    A2 --> A2S["Промяна на статус"]
    A2 --> A2T["Преглед на timeline"]

    E2 --> E2D["Създаване на пратка<br/>само в разрешен офис"]
    E2 --> E2S["Промяна на статус<br/>само за пратки в разрешен офис"]
    E2 --> E2T["Преглед на timeline<br/>само за достъпни пратки"]

    EMP -. няма достъп .-> A3
    EMP -. няма достъп .-> A4
    EMP -. няма достъп .-> A5
    EMP -. няма достъп .-> A6
    EMP -. няма достъп .-> A7

    classDef admin fill:#d9f4e4,stroke:#1f7a4d,color:#0e3b24,stroke-width:1px;
    classDef employee fill:#e7f0fb,stroke:#245ea8,color:#0f2f57,stroke-width:1px;
    classDef scope fill:#fff1cc,stroke:#a66b00,color:#5c3b00,stroke-width:1px;
    class ADMIN,ADMIN_ZONE,A1,A2,A3,A4,A5,A6,A7,A8,A2D,A2S,A2T admin;
    class EMP,EMP_ZONE,E1,E2,E3,E2D,E2S,E2T employee;
    class MAP,AUTH,U,EMP_SCOPE scope;
```

## Кратко тълкуване

- `Admin` има пълен достъп до административните и оперативните модули: `Пратки`, `Клиенти`, `Офиси`, `Служители`, `Справки`, `Одитни събития`, `Профил`.
- `Employee` има достъп само до оперативната част: `Пратки`, `Профил`, `Табло`.
- Достъпът на `Employee` до модула `Пратки` е ограничен по офис чрез връзката `employees <-> offices` (`employee_offices`).
- `Employee` няма достъп до административните модули `Клиенти`, `Офиси`, `Служители`, `Справки` и `Одитни събития`.
