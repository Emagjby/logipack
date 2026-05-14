# Figure: Проследимост, история на статуси, общ одит и Strata в LogiPack

```mermaid
flowchart LR
    U["Потребител<br/>(Admin или Employee)"] --> ACT["Действие в системата<br/>напр. създаване на пратка,<br/>промяна на статус,<br/>редакция на клиент/офис/служител"]
    ACT --> APP["LogiPack Hub API / LogiCore"]

    subgraph REL["Релационен модел (оперативни данни)"]
        SNAP["Текущо състояние на пратката<br/>таблица: shipments<br/>- current_status<br/>- current_office_id<br/>Използва се за ежедневна работа"]
        HIST["История на статусите<br/>таблица: shipment_status_history<br/>- from_status / to_status<br/>- changed_at<br/>- actor_user_id<br/>Използва се за проследяване на жизнения цикъл"]
        AUD["Общ одит на действията<br/>таблица: audit_events<br/>- action_key<br/>- actor_user_id<br/>- entity_type / entity_id<br/>Използва се за административен контрол"]
    end

    subgraph IMM["Неизменима история чрез Strata"]
        STR["Strata event store<br/>таблици: streams + packages<br/>- stream_id<br/>- seq<br/>- prev_hash / head_hash<br/>- SCB payload<br/>Използва се за неизменим timeline на пратката"]
    end

    APP --> SNAP
    APP --> HIST
    APP --> AUD
    APP --> STR

    SNAP --> V1["Показва какво е<br/>текущото състояние сега"]
    HIST --> V2["Показва през кои статуси<br/>е преминала пратката"]
    AUD --> V3["Показва кой е извършил<br/>значими действия в системата"]
    STR --> V4["Показва проверима и<br/>неизменима последователност<br/>на събитията за пратката"]

    SHIP["Преход на пратка<br/>NEW -> ACCEPTED -> PROCESSED -> IN_TRANSIT ..."] -. записва се в .-> HIST
    SHIP -. неизменимо се записва и в .-> STR

    CRUD["Създаване/редакция на клиент,<br/>офис или служител"] -. записва се в .-> AUD

    classDef actor fill:#e7f0fb,stroke:#245ea8,color:#0f2f57,stroke-width:1px;
    classDef app fill:#fff1cc,stroke:#a66b00,color:#5c3b00,stroke-width:1px;
    classDef rel fill:#d9f4e4,stroke:#1f7a4d,color:#0e3b24,stroke-width:1px;
    classDef imm fill:#fbe3ea,stroke:#a63d5c,color:#5a1730,stroke-width:1px;

    class U,ACT,SHIP,CRUD actor;
    class APP,V1,V2,V3,V4 app;
    class REL,SNAP,HIST,AUD rel;
    class IMM,STR imm;
```

## Кратко тълкуване

- `Текущото състояние` показва актуалната оперативна информация за пратката и се пази в `shipments`.
- `Историята на статусите` показва през кои състояния е преминала конкретната пратка и се пази в `shipment_status_history`.
- `Общият одит` описва по-широк кръг от значими действия в системата, не само по пратки, и се пази в `audit_events`.
- `Strata` се използва за неизменимата timeline история на преходите на пратката чрез `streams` и `packages`.
- По този начин в LogiPack има ясно разграничение между:
  - текуща оперативна снимка
  - историческа статусна последователност
  - общ системен одит
  - неизменима event-based история чрез Strata

## Подходящ надпис под фигурата

`Фигура X. Концептуална схема на разликата между текущото състояние на пратката, историята на статусите, общия одит на действията и неизменимата timeline история чрез Strata в системата LogiPack.`
