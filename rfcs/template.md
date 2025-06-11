
- Feature Name: reservation-service-system
- Start Date: 2025-06-08

# Summary

预订系统的核心，该系统可以解决在某一时间预订某一资源的问题。我们会使用postgres的“EXCLUDE”constraint去
确保在给定时间内，对给定资源的预订只有一个。

# Motivation

我们需要一个普遍的解决方案针对多种预订需求：1）日历预订；2）房间预订；3）会议室预订等。

# Guide-level explanation

## Service interface

使用gRPC作为服务接口，定义如下：

```proto
enum ReservationStatus {
    UNKNOWN = 0;
    PENDING = 1;
    CONFIRMED = 2;
    BLOCKED = 3;
}

enum ReservationUpdateType {
    UNKNOWN = 0;
    CREATE = 1;
    UPDATE = 2;
    DELETE = 3;
}

message Reservation {
    string id = 1;
    string user_id = 2;
    ReservationStatus status = 3;

    string resource_id = 4;
    google.protobuf.Timestamp start = 5;
    google.protobuf.Timestamp end = 6;

    string note = 7;
}

message ReserveRequest {
    Reservation reservation = 1;
}

message ReserveResponse {
    Reservation reservation = 1;
}

message UpdateRquest {
    string note  = 1;
}

message UpdateResponse {
    Reservation reservation = 1;
}

message ConfirmRequest {
    string id  = 1;
}

message ConfirmResponse {
    Reservation reservation = 1;
}

message CancelRequest {
    string id  = 1;
}

message CancelResponse {
    Reservation reservation = 1;
}

message GetRequest {
    string id  = 1;
}

message GetResponse {
    Reservation reservation = 1;
}

message QueryRequest {
    string resouce_id  = 1;
    string user_id  = 2;

    //use status to filter result.If UNKNOWN, return all.
    ReservationStatus status = 3;
    google.protobuf.Timestamp start = 4;
    google.protobuf.Timestamp end = 5;
}

message ListenRequest {}
message ListenResponse {
    int8 op = 1;
    Reservation reservation = 2;
 }

service ReservationService {
    rpc reserve(ReserveRequest) returns (ReserveResponse);
    rpc confirm(ConfimRequest) returns (ConfirmResponse);
    rpc update(UpdateRequest) returns (UpdateResponse);
    rpc cancle(CancelRequest) returns (CancelResponse);
    rpc get(GetRequest) returns (GetResponse);
    rpc query(QueryRequest) returns (stream Reservation);
    // 其他系统可以监听最近添加的预约。
    rpc listen(ListenRequest) returns (stream Reservation);
}
```

### Database schema

我们使用PostgreSQL作为数据库，数据库结构如下：
```sql
CREATE SCHEMA rsvp;
CREATE TYPE rsvp.reservation_status AS ENUM ('unknown', 'pending', 'confirmed', 'blocked');
CREATE TYPE rsvp.reservation_update_type AS ENUM ('unknown', 'create', 'update', 'delete');

CREATE TABLE rsvp.reservations (
    id uuid NOT NULL DEFAULT uuid_generate_v4(),
    user_id VARCHAR(64) NOT NULL,
    status rsvp.reservation_status NOT NULL DEFAULT 'pending',

    resource_id VARCHAR(64) NOT NULL,
    timespan TSTZRANGE NOT NULL,

    note TEXT,

    CONSTRAINT reservation_pkey PRIMARY KEY (id),
    CONSTRAiNT reservations_conflict EXCLUDE USING GIST (resource_id WITH =, timespan WITH &&)
);
CREATE INDEX reservation_resource_id_idx ON rsvp.reservations (resource_id);
CREATE INDEX reservation_user_id_idx ON rsvp.reservations (user_id);

-- 如果user_id是null，找到时间段内资源对应的所有预订。
-- 如果resource_id是null，找到时间段内用户对应所有预订。
-- 如果两个都是null，找到时间段内所有预订。
-- 如果两个都不为null，找到时间段内对应用户和对应资源的所有预订。
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during tstzrange) RETURNS TABLE (LIKE rsvp.reservations) AS $$ $$ LANGUAGE plpgsql;

-- resevation改变记录队列
CREATE TABLE rsvp.reservation_changes (
    id serial NOT NULL,
    reservation_id uuid  NOT NULL,
    op rsvp.reservation_update_type NOT NULL
);
-- 设置一个在add/update/delete时的触发器。
CREATE OR REPLACE FUNCTION rsvp.reservations_trigger() RETURNS TRIGGER AS 
$$ 
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'create');
    ELSIF (TG_OP = 'UPDATE') THEN
        IF OLD.status <>  NEW.status THEN
            INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'update');
        END IF;
    ELSIF (TG_OP = 'DELETE') THEN
        INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (OLD.id, 'delete');
    END IF;
    NOTIFY reservation_update;
    RETURN NULL;
END;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger
    AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
    FOR EACH ROW EXECUTE PROCEDURE rsvp.reservations_trigger();
```

这里我们使用postgres的“EXCLUDE”constraint去确保在给定时间内，对给定资源的预订只有一个。
```sql
CONSTRAINT reservations_conflict EXCLUDE USING GIST (resource_id WITH =, timespan WITH &&)
```

当一个预订被添加/更新/删除时，我们使用一个trigger去通知一个通道。为了保证在DB链接中断时我们不会
丢失消息，我们使用一个队列去存储预订改变。因此当我们收到一个通知时，我们可以检查这个队列以获得
自上次检查点以后的所有改变，并且一旦我们结束了对改变的处理，我们可以从队列中删除这些改变。

# Reference-level explanation

N/A

# Drawbacks

N/A

# Rationale and alternatives

N/A

# Prior art

N/A

# Unresolved questions



# Future possibilities
