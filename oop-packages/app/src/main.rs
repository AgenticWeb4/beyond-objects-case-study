//! 【小白】命令行。目录拆成了 user / restaurant / reservation 几个包，
//! 第 5、6 步仍和朴素 DDD 一样：搜索变空，Bob 得 NoSlot。
//! 分包 ≠ 概念设计。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use restaurant_oop_reservation::ReservationId;
use restaurant_oop_shared::{Date, DateTime, TimeOfDay};
use restaurant_oop_user::SessionId;

mod ports;
mod sqlite;
mod use_cases;

use sqlite::SqliteDb;
use use_cases::{ReserveOutcome, RestaurantApp};

fn usage() -> &'static str {
    "restaurant-oop-packages [--data DIR] <command>

命令与 restaurant-concept 相同。SQLite 仍是单库 DIR/restaurant.sqlite。
对象类拆到了不同 crate，外键对象图没变。
"
}

fn parse_data_dir(args: &[String]) -> (PathBuf, &[String]) {
    if args.len() >= 2 && args[0] == "--data" {
        (PathBuf::from(&args[1]), &args[2..])
    } else {
        (PathBuf::from("data/oop-packages"), args)
    }
}

fn open_app(dir: &Path) -> RestaurantApp<SqliteDb> {
    fs::create_dir_all(dir).expect("create data dir");
    RestaurantApp::new(SqliteDb::open(dir.join("restaurant.sqlite")).expect("oop sqlite"))
}

fn main() -> ExitCode {
    let all: Vec<String> = std::env::args().skip(1).collect();
    let (dir, args) = parse_data_dir(&all);
    if args.is_empty() {
        eprint!("{}", usage());
        return ExitCode::FAILURE;
    }
    match run(&dir, args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run(dir: &Path, args: &[String]) -> Result<(), String> {
    let mut app = open_app(dir);
    match args[0].as_str() {
        "demo" => demo(&mut app),
        "register" => {
            need(args, 3)?;
            println!("user {}", app.register(&args[1], &args[2])?);
            Ok(())
        }
        "login" => {
            need(args, 3)?;
            println!("session {}", app.login(&args[1], &args[2])?);
            Ok(())
        }
        "publish" => {
            need(args, 7)?;
            let id = app.publish(
                &args[1],
                TimeOfDay::parse(&args[2])?,
                TimeOfDay::parse(&args[3])?,
                Date::parse(&args[4])?,
                args[5].parse().map_err(|_| "min")?,
                args[6].parse().map_err(|_| "max")?,
            )?;
            println!("slot {id}");
            Ok(())
        }
        "available" => {
            need(args, 5)?;
            let slots = app.available(
                &args[1],
                &TimeOfDay::parse(&args[2])?,
                &Date::parse(&args[3])?,
                args[4].parse().map_err(|_| "party")?,
            );
            if slots.is_empty() {
                println!("(none)");
            } else {
                for s in slots {
                    println!("{s}");
                }
            }
            Ok(())
        }
        "reserve" => {
            need(args, 6)?;
            println!(
                "{}",
                format_reserve(&app.reserve(
                    &SessionId(args[1].clone()),
                    &args[2],
                    TimeOfDay::parse(&args[3])?,
                    Date::parse(&args[4])?,
                    args[5].parse().map_err(|_| "party")?,
                ))
            );
            Ok(())
        }
        "cancel" => {
            need(args, 2)?;
            app.cancel(&ReservationId(args[1].clone()))?;
            println!("canceled");
            Ok(())
        }
        "grant" => {
            need(args, 3)?;
            app.grant(
                &ReservationId(args[1].clone()),
                &DateTime::parse(&args[2])?,
            )?;
            println!("granted");
            Ok(())
        }
        "noshow" => {
            need(args, 3)?;
            app.no_show(
                &ReservationId(args[1].clone()),
                &DateTime::parse(&args[2])?,
            )?;
            println!("no-show recorded; user.karma punished");
            Ok(())
        }
        _ => Err(usage().to_string()),
    }
}

fn demo(app: &mut RestaurantApp<SqliteDb>) -> Result<(), String> {
    println!("=== 面向对象分包演示（同一餐厅案例）===");
    let date = Date::parse("2026-01-10")?;
    let time = TimeOfDay::parse("19:30")?;
    let slot = app.publish("maido", time.clone(), TimeOfDay::parse("21:00")?, date.clone(), 2, 6)?;
    println!("1. restaurant crate 的 Restaurant 聚合发布时段 → slot {slot}");

    app.register("alice", "secret").ok();
    app.register("bob", "secret").ok();
    let alice = app.login("alice", "secret")?;
    let bob = app.login("bob", "secret")?;
    println!("2. user crate：认证 + karma + booked_dates 仍在同一 User");

    println!("3. 可订查询 → {:?}", app.available("maido", &time, &date, 2));
    let r1 = app.reserve(&alice, "maido", time.clone(), date.clone(), 2);
    println!(
        "4. booking crate 同时改 user / restaurant / reservation 三个包 → {}",
        format_reserve(&r1)
    );
    println!(
        "5. 预约后可订查询 → {:?}（Slot.reserved=true；分包没有拆开预测和承诺）",
        app.available("maido", &time, &date, 2)
    );
    let r2 = app.reserve(&bob, "maido", time.clone(), date.clone(), 2);
    println!(
        "6. Bob 再订 → {}（找不到 slot，不是概念侧的 Conflict）",
        format_reserve(&r2)
    );

    if let ReserveOutcome::Reserved(id) = &r1 {
        app.grant(id, &DateTime::parse("2026-01-10T19:35")?)?;
        println!("7. grant → User 去掉 booked_dates；Slot 仍 reserved");
    }

    let date2 = Date::parse("2026-01-11")?;
    app.publish("maido", time.clone(), TimeOfDay::parse("21:00")?, date2.clone(), 2, 6)?;
    let r3 = app.reserve(&alice, "maido", time.clone(), date2.clone(), 2);
    println!("8. 次日再订 → {}", format_reserve(&r3));
    if let ReserveOutcome::Reserved(id) = &r3 {
        app.no_show(id, &DateTime::parse("2026-01-11T22:00")?)?;
        println!("   no-show → User.punish()（karma 仍在 user crate）");
    }
    let date3 = Date::parse("2026-01-12")?;
    app.publish("maido", time.clone(), TimeOfDay::parse("21:00")?, date3.clone(), 2, 6)?;
    let r4 = app.reserve(&alice, "maido", time.clone(), date3.clone(), 2);
    println!("9. 再订 → {}", format_reserve(&r4));
    if let ReserveOutcome::Reserved(id) = &r4 {
        app.no_show(id, &DateTime::parse("2026-01-12T22:00")?)?;
        println!("   第二次 punish，User.in_good_standing=false");
    }
    let date4 = Date::parse("2026-01-13")?;
    app.publish("maido", time.clone(), TimeOfDay::parse("21:00")?, date4.clone(), 2, 6)?;
    let r5 = app.reserve(&alice, "maido", time.clone(), date4.clone(), 2);
    println!("10. Alice 再订 → {}", format_reserve(&r5));
    let r6 = app.reserve(&bob, "maido", time, date4, 2);
    println!("    Bob 仍可订 → {}", format_reserve(&r6));
    Ok(())
}

fn format_reserve(out: &ReserveOutcome) -> String {
    match out {
        ReserveOutcome::Reserved(id) => format!("reservation {id}"),
        ReserveOutcome::Unauthorized => "Unauthorized".into(),
        ReserveOutcome::BadStanding => "BadStanding".into(),
        ReserveOutcome::NoSlot => "NoSlot".into(),
        ReserveOutcome::Conflict => "Conflict".into(),
    }
}

fn need(args: &[String], n: usize) -> Result<(), String> {
    if args.len() >= n {
        Ok(())
    } else {
        Err(usage().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> RestaurantApp<SqliteDb> {
        RestaurantApp::new(SqliteDb::memory().unwrap())
    }

    #[test]
    fn reserve_hides_slot_same_as_ddd_clean() {
        let mut app = app();
        app.register("alice", "x").unwrap();
        app.register("bob", "x").unwrap();
        let a = app.login("alice", "x").unwrap();
        let b = app.login("bob", "x").unwrap();
        let t = TimeOfDay::parse("19:30").unwrap();
        let d = Date::parse("2026-01-10").unwrap();
        app.publish("maido", t.clone(), TimeOfDay::parse("21:00").unwrap(), d.clone(), 2, 4)
            .unwrap();
        assert_eq!(app.available("maido", &t, &d, 2).len(), 1);
        assert!(matches!(
            app.reserve(&a, "maido", t.clone(), d.clone(), 2),
            ReserveOutcome::Reserved(_)
        ));
        assert!(
            app.available("maido", &t, &d, 2).is_empty(),
            "分包之后 reserved 仍焊在 Slot 上"
        );
        assert_eq!(app.reserve(&b, "maido", t, d, 2), ReserveOutcome::NoSlot);
    }

    #[test]
    fn two_noshows_punish_user_karma_field() {
        let mut app = app();
        app.register("alice", "x").unwrap();
        let s = app.login("alice", "x").unwrap();
        let t = TimeOfDay::parse("19:30").unwrap();
        for day in ["2026-01-10", "2026-01-11"] {
            let d = Date::parse(day).unwrap();
            app.publish("maido", t.clone(), TimeOfDay::parse("21:00").unwrap(), d.clone(), 2, 4)
                .unwrap();
            let ReserveOutcome::Reserved(id) = app.reserve(&s, "maido", t.clone(), d, 2) else {
                panic!("reserve");
            };
            app.no_show(&id, &DateTime::parse(&format!("{day}T22:00")).unwrap())
                .unwrap();
        }
        let d = Date::parse("2026-01-12").unwrap();
        app.publish("maido", t.clone(), TimeOfDay::parse("21:00").unwrap(), d.clone(), 2, 4)
            .unwrap();
        assert_eq!(
            app.reserve(&s, "maido", t, d, 2),
            ReserveOutcome::BadStanding
        );
    }

    /// 和概念 / 模块化 DDD 相反：对象包必须互相依赖。
    #[test]
    fn object_crates_depend_on_each_other() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let reservation = fs::read_to_string(root.join("reservation/Cargo.toml")).unwrap();
        assert!(
            reservation.contains("restaurant-oop-user"),
            "订单包依赖用户包：对象图穿过包边界"
        );
        assert!(
            reservation.contains("restaurant-oop-restaurant"),
            "订单包依赖餐厅包"
        );
        let booking = fs::read_to_string(root.join("booking/Cargo.toml")).unwrap();
        assert!(booking.contains("restaurant-oop-user"));
        assert!(booking.contains("restaurant-oop-restaurant"));
        assert!(booking.contains("restaurant-oop-reservation"));
    }
}
