
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
    string user_id = 3;
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
    int
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

CREATE TABLE reservation (
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
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during: tstzrange) RETURNS TABLE rsvp.reservations AS $$ $$ LANGUAGE plpgsql;

-- resevation改变记录队列
CREATE TABLE rsvp.reservation_changes (
    id serial PRIMARY KEY,
    reservation_id uuid  NOT NULL,
    op rsvp.reservation_update_type NOT NULL,
)
-- 设置一个在add/update/delete时的触发器。
CREATE OR REPLACE FUNCTION rsvp.reservation_trigger() RETURNS TRIGGER AS 
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
    NOTIFY reservation_update, NEW.id;
    RETURN NULL;
END;
END
$$ LANGUAGE plpgsql;
CREATE TRIGGER rsvp.reservation_trigger
    AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
    FOR EACH ROW EXECUTE PROCEDURE rsvp.reservation_trigger();
```

# Reference-level explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

# Drawbacks

Why should we *not* do this?

# Rationale and alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?
- If this is a language proposal, could this be done in a library or macro instead? Does the proposed change make Rust code easier or harder to read, understand, and maintain?

# Prior art

Discuss prior art, both the good and the bad, in relation to this proposal.
A few examples of what this can include are:

- For language, library, cargo, tools, and compiler proposals: Does this feature exist in other programming languages and what experience have their community had?
- For community proposals: Is this done by some other community and what were their experiences with it?
- For other teams: What lessons can we learn from what other communities have done here?
- Papers: Are there any published papers or great posts that discuss this? If you have some relevant papers to refer to, this can serve as a more detailed theoretical background.

This section is intended to encourage you as an author to think about the lessons from other languages, provide readers of your RFC with a fuller picture.
If there is no prior art, that is fine - your ideas are interesting to us whether they are brand new or if it is an adaptation from other languages.

Note that while precedent set by other languages is some motivation, it does not on its own motivate an RFC.
Please also take into consideration that rust sometimes intentionally diverges from common language features.

# Unresolved questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?

# Future possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the language and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and language in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.

