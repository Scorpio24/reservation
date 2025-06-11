-- 如果user_id是null，找到时间段内资源对应的所有预订。
-- 如果resource_id是null，找到时间段内用户对应所有预订。
-- 如果两个都是null，找到时间段内所有预订。
-- 如果两个都不为null，找到时间段内对应用户和对应资源的所有预订。
CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during tstzrange) RETURNS TABLE (LIKE rsvp.reservations) AS $$
BEGIN
    -- 如果两个都是null，找到时间段内所有预订。
    IF uid IS NULL AND rid IS NULL THEN
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE timespan && during;
    ELSIF uid IS NULL THEN
        -- 如果user_id是null，找到时间段内资源对应的所有预订。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
    ELSIF rid IS NULL THEN
        -- 如果resource_id是null，找到时间段内用户对应所有预订。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during  @> timespan;
    ELSE
        -- 如果两个都不为null，找到时间段内对应用户和对应资源的所有预订。
        RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND resource_id = rid AND during  @> timespan;
    END IF;
END;
$$ LANGUAGE plpgsql;