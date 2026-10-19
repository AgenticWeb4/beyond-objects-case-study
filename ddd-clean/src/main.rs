//! 【小白】命令行。故事和概念侧同一套，但第 5、6 步的拒绝理由不同。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use restaurant_ddd::application::use_cases::{ReserveOutcome, RestaurantApp};
use restaurant_ddd::domain::ids::{Date, DateTime, ReservationId, SessionId, TimeOfDay};
use restaurant_ddd::infrastructure::SqliteDb;

fn usage() -> &'static str {
    "restaurant-ddd [--data DIR] <command>

命令与 restaurant-concept 相同。SQLite 单库 DIR/restaurant.sqlite，
外键把 User / Restaurant / Slot / Reservation 焊成对象图。
"
}

fn parse_data_dir(args: &[String]) -> (PathBuf, &[String]) {
    if args.len() >= 2 && args[0] == "--data" {
        (PathBuf::from(&args[1]), &args[2..])
    } else {
        (PathBuf::from("data/ddd"), args)
    }
}

fn open_app(dir: &Path) -> RestaurantApp<SqliteDb> {
    fs::create_dir_all(dir).expect("create data dir");
    RestaurantApp::new(SqliteDb::open(dir.join("restaurant.sqlite")).expect("ddd sqlite"))
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
    println!("=== DDD + Clean 演示（同一餐厅案例）===");
    let date = Date::parse("2026-01-10")?;
    let time = TimeOfDay::parse("19:30")?;
    let slot = app.publish("maido", time.clone(), TimeOfDay::parse("21:00")?, date.clone(), 2, 6)?;
    println!("1. Restaurant 聚合发布时段 → slot {slot}");

    app.register("alice", "secret").ok();
    app.register("bob", "secret").ok();
    let alice = app.login("alice", "secret")?;
    let bob = app.login("bob", "secret")?;
    println!("2. User 聚合：认证 + karma 字段住在同一对象");

    println!("3. 可订查询 → {:?}", app.available("maido", &time, &date, 2));
    let r1 = app.reserve(&alice, "maido", time.clone(), date.clone(), 2);
    println!("4. BookingService.reserve 同时改 User / Slot / Reservation → {}", format_reserve(&r1));
    // 【小白】位子被标占用，搜索跟着变空。概念侧这里还看得到牌。
    println!(
        "5. 预约后可订查询 → {:?}（Slot.reserved=true，预测与承诺焊死）",
        app.available("maido", &time, &date, 2)
    );
    let r2 = app.reserve(&bob, "maido", time.clone(), date.clone(), 2);
    // 【小白】Bob 得到的是「找不到位子」，不是「这份承诺已经给过了」。
    println!("6. Bob 再订 → {}（找不到 slot，不是 Reserving 的 Conflict）", format_reserve(&r2));

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
        println!("   no-show → User.punish()（karma 在 User 上）");
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
