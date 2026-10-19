//! 【小白】这是「旧写法」：人、店、订单都是对象，预约方法不知道该写在谁身上。
//!
//! 常见公司分层还在（domain / application / infrastructure），
//! 但切模块的刀仍然沿着 User、Restaurant 这些**名词**走。
//!
//! 对比 `../concept/`：那边按「干什么」切开（登录 / 空位 / 订座 / 信誉）。
//! 导读：`START-HERE.md`。

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

#[cfg(test)]
mod tests {
    use crate::application::use_cases::{ReserveOutcome, RestaurantApp};
    use crate::domain::ids::{Date, DateTime, TimeOfDay};
    use crate::infrastructure::SqliteDb;

    fn app() -> RestaurantApp<SqliteDb> {
        RestaurantApp::new(SqliteDb::memory().unwrap())
    }

    /// 和概念测试 `conflict_does_not_hide_availability` 对打：
    /// 这里搜索变空，Bob 得到 NoSlot（找不到对象），不是承诺冲突。
    #[test]
    fn reserve_hides_slot_on_restaurant_aggregate() {
        let mut app = app();
        app.register("alice", "x").unwrap();
        app.register("bob", "x").unwrap();
        let a = app.login("alice", "x").unwrap();
        let b = app.login("bob", "x").unwrap();
        let t = TimeOfDay::parse("19:30").unwrap();
        let d = Date::parse("2026-01-10").unwrap();
        app.publish(
            "maido",
            t.clone(),
            TimeOfDay::parse("21:00").unwrap(),
            d.clone(),
            2,
            4,
        )
        .unwrap();
        assert_eq!(app.available("maido", &t, &d, 2).len(), 1);
        assert!(matches!(
            app.reserve(&a, "maido", t.clone(), d.clone(), 2),
            ReserveOutcome::Reserved(_)
        ));
        assert!(
            app.available("maido", &t, &d, 2).is_empty(),
            "DDD 把 reserved 焊在 Slot 上，搜索不再列出"
        );
        assert_eq!(app.reserve(&b, "maido", t, d, 2), ReserveOutcome::NoSlot);
    }

    /// karma 是 User 上的字段，不是独立概念。两次 no-show 后同样拒绝。
    #[test]
    fn two_noshows_punish_user_karma_field() {
        let mut app = app();
        app.register("alice", "x").unwrap();
        let s = app.login("alice", "x").unwrap();
        let t = TimeOfDay::parse("19:30").unwrap();
        for day in ["2026-01-10", "2026-01-11"] {
            let d = Date::parse(day).unwrap();
            app.publish(
                "maido",
                t.clone(),
                TimeOfDay::parse("21:00").unwrap(),
                d.clone(),
                2,
                4,
            )
            .unwrap();
            let ReserveOutcome::Reserved(id) = app.reserve(&s, "maido", t.clone(), d, 2) else {
                panic!("reserve");
            };
            app.no_show(&id, &DateTime::parse(&format!("{day}T22:00")).unwrap())
                .unwrap();
        }
        let d = Date::parse("2026-01-12").unwrap();
        app.publish(
            "maido",
            t.clone(),
            TimeOfDay::parse("21:00").unwrap(),
            d.clone(),
            2,
            4,
        )
        .unwrap();
        assert_eq!(
            app.reserve(&s, "maido", t, d, 2),
            ReserveOutcome::BadStanding
        );
    }
}
