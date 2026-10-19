//! 【小白】收银台。故事和另两栏同一套。
//!
//! 第 5、6 步对外可以和概念侧相像（还能搜到，Bob 得 Conflict）。
//! 差别在打开 `booking` 和 `orchestration.rs`：仍是对象拼图 + 用例编排。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use restaurant_ddd_availability::{AvailabilityService, SlotId, SqliteSlotStore, VenueId};
use restaurant_ddd_booking::{BookingFacade, ReservationId, SqliteBookingStore};
use restaurant_ddd_identity::{IdentityService, SessionId, SqliteIdentityStore};
use restaurant_ddd_kernel::{Date, DateTime, ParseError, TimeOfDay};
use restaurant_ddd_reputation::{ReputationService, SqliteReputationStore};

mod orchestration;
use orchestration::{ReserveOutcome, RestaurantApp};

fn usage() -> &'static str {
    "restaurant-ddd-modular [--data DIR] <command>

命令与 restaurant-concept / restaurant-ddd 相同。

SQLite 文件（按限界上下文分库，无跨上下文外键）:
  DIR/identity.sqlite
  DIR/availability.sqlite
  DIR/booking.sqlite
  DIR/reputation.sqlite
"
}

fn parse_data_dir(args: &[String]) -> (PathBuf, &[String]) {
    if args.len() >= 2 && args[0] == "--data" {
        (PathBuf::from(&args[1]), &args[2..])
    } else {
        (PathBuf::from("data/ddd-modular"), args)
    }
}

/// 【小白】准备四本库。注意是四个文件。和概念侧「四本账」长得像，
/// 但每个文件里住的是该上下文的聚合/仓储，不是一台概念机。
fn open_app(dir: &Path) -> RestaurantApp {
    fs::create_dir_all(dir).expect("create data dir");
    RestaurantApp::new(
        IdentityService::new(
            SqliteIdentityStore::open(dir.join("identity.sqlite")).expect("identity db"),
        ),
        AvailabilityService::new(
            SqliteSlotStore::open(dir.join("availability.sqlite")).expect("availability db"),
        ),
        BookingFacade::new(
            SqliteBookingStore::open(dir.join("booking.sqlite")).expect("booking db"),
        ),
        ReputationService::new(
            SqliteReputationStore::open(dir.join("reputation.sqlite")).expect("reputation db"),
        ),
    )
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
            let slot = app.publish(
                VenueId::new(&args[1]),
                TimeOfDay::parse(&args[2]).map_err(pe)?,
                TimeOfDay::parse(&args[3]).map_err(pe)?,
                Date::parse(&args[4]).map_err(pe)?,
                args[5].parse().map_err(|_| "min party")?,
                args[6].parse().map_err(|_| "max party")?,
            )?;
            println!("slot {slot}");
            Ok(())
        }
        "available" => {
            need(args, 5)?;
            let slots = app.available(
                &VenueId::new(&args[1]),
                &TimeOfDay::parse(&args[2]).map_err(pe)?,
                &Date::parse(&args[3]).map_err(pe)?,
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
                    &VenueId::new(&args[2]),
                    TimeOfDay::parse(&args[3]).map_err(pe)?,
                    Date::parse(&args[4]).map_err(pe)?,
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
                &DateTime::parse(&args[2]).map_err(pe)?,
            )?;
            println!("granted");
            Ok(())
        }
        "noshow" => {
            need(args, 3)?;
            app.no_show(
                &ReservationId(args[1].clone()),
                &DateTime::parse(&args[2]).map_err(pe)?,
            )?;
            println!("no-show recorded; reputation punished");
            Ok(())
        }
        "toggle" => {
            need(args, 2)?;
            app.availability
                .toggle(&SlotId(args[1].clone()))
                .map_err(|e| format!("{e:?}"))?;
            println!("toggled");
            Ok(())
        }
        _ => Err(usage().to_string()),
    }
}

/// 【小白】同一出戏。盯第 5、6 步：对外像概念侧；打开 Booking 仍是对象碎片。
fn demo(app: &mut RestaurantApp) -> Result<(), String> {
    println!("=== 模块化 DDD 演示（同一餐厅案例）===");
    let venue = VenueId::new("maido");
    let date = Date::parse("2026-01-10").unwrap();
    let time = TimeOfDay::parse("19:30").unwrap();
    let slot = app.publish(
        venue.clone(),
        time.clone(),
        TimeOfDay::parse("21:00").unwrap(),
        date.clone(),
        2,
        6,
    )?;
    println!("1. Availability 上下文发布时段 → slot {slot}");

    app.register("alice", "secret").ok();
    app.register("bob", "secret").ok();
    let alice = app.login("alice", "secret")?;
    let bob = app.login("bob", "secret")?;
    println!("2. Identity 只有登录；Reputation / Guest 不在 User 口袋");

    let found = app.available(&venue, &time, &date, 2);
    println!("3. 可订查询（只读 Availability）→ {found:?}");

    let r1 = app.reserve(&alice, &venue, time.clone(), date.clone(), 2);
    println!(
        "4. 用例编排：Identity → Reputation → Availability → BookingService → {}",
        format_reserve(&r1)
    );

    let found_after = app.available(&venue, &time, &date, 2);
    println!("5. 预约后可订查询仍为 {found_after:?}");
    println!("   （Availability 不读 Booking：占用在 SlotOccupancy，不在 listed）");

    let r2 = app.reserve(&bob, &venue, time.clone(), date.clone(), 2);
    println!(
        "6. Bob 再订同一时段 → {}（BookingService 改三块碎片后的 Conflict）",
        format_reserve(&r2)
    );

    if let ReserveOutcome::Reserved(id) = &r1 {
        app.grant(id, &DateTime::parse("2026-01-10T19:35").unwrap())?;
        println!("7. grant → Guest 去掉 booked_dates；Occupancy 释放");
    }

    let date2 = Date::parse("2026-01-11").unwrap();
    app.publish(
        venue.clone(),
        time.clone(),
        TimeOfDay::parse("21:00").unwrap(),
        date2.clone(),
        2,
        6,
    )
    .ok();
    let r3 = app.reserve(&alice, &venue, time.clone(), date2.clone(), 2);
    println!("8. 次日 Alice 再订 → {}", format_reserve(&r3));
    if let ReserveOutcome::Reserved(id) = &r3 {
        app.no_show(id, &DateTime::parse("2026-01-11T22:00").unwrap())?;
        println!("   no-show → Booking 划掉 + Reputation.punish（第 1 次）");
    }

    let date3 = Date::parse("2026-01-12").unwrap();
    app.publish(
        venue.clone(),
        time.clone(),
        TimeOfDay::parse("21:00").unwrap(),
        date3.clone(),
        2,
        6,
    )
    .ok();
    let r4 = app.reserve(&alice, &venue, time.clone(), date3.clone(), 2);
    println!("9. 再次预约 → {}", format_reserve(&r4));
    if let ReserveOutcome::Reserved(id) = &r4 {
        app.no_show(id, &DateTime::parse("2026-01-12T22:00").unwrap())?;
        println!("   no-show → Reputation.punish（第 2 次，standing 变坏）");
    }

    let date4 = Date::parse("2026-01-13").unwrap();
    app.publish(
        venue.clone(),
        time.clone(),
        TimeOfDay::parse("21:00").unwrap(),
        date4.clone(),
        2,
        6,
    )
    .ok();
    let r5 = app.reserve(&alice, &venue, time.clone(), date4.clone(), 2);
    println!("10. Alice 第三次再订 → {}", format_reserve(&r5));
    let r6 = app.reserve(&bob, &venue, time, date4, 2);
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

fn pe(err: ParseError) -> String {
    err.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use restaurant_ddd_availability::AvailabilityService;
    use restaurant_ddd_booking::BookingFacade;
    use restaurant_ddd_identity::IdentityService;
    use restaurant_ddd_reputation::ReputationService;

    fn memory_app() -> RestaurantApp {
        RestaurantApp::new(
            IdentityService::new(SqliteIdentityStore::memory().unwrap()),
            AvailabilityService::new(SqliteSlotStore::memory().unwrap()),
            BookingFacade::new(SqliteBookingStore::memory().unwrap()),
            ReputationService::new(SqliteReputationStore::memory().unwrap()),
        )
    }

    /// 和第 5、6 步对打：搜索仍在，拒绝是 Booking 的 Conflict。
    #[test]
    fn conflict_does_not_hide_availability() {
        let mut app = memory_app();
        app.register("alice", "x").unwrap();
        app.register("bob", "x").unwrap();
        let a = app.login("alice", "x").unwrap();
        let b = app.login("bob", "x").unwrap();
        let v = VenueId::new("maido");
        let t = TimeOfDay::parse("19:30").unwrap();
        let d = Date::parse("2026-01-10").unwrap();
        app.publish(
            v.clone(),
            t.clone(),
            TimeOfDay::parse("21:00").unwrap(),
            d.clone(),
            2,
            4,
        )
        .unwrap();
        assert_eq!(app.available(&v, &t, &d, 2).len(), 1);
        assert!(matches!(
            app.reserve(&a, &v, t.clone(), d.clone(), 2),
            ReserveOutcome::Reserved(_)
        ));
        assert_eq!(
            app.available(&v, &t, &d, 2).len(),
            1,
            "Availability 仍列出；占用在 Booking"
        );
        assert_eq!(app.reserve(&b, &v, t, d, 2), ReserveOutcome::Conflict);
    }

    #[test]
    fn two_noshows_block_later_reserve() {
        let mut app = memory_app();
        app.register("alice", "x").unwrap();
        let s = app.login("alice", "x").unwrap();
        let v = VenueId::new("maido");
        let t = TimeOfDay::parse("19:30").unwrap();
        for (i, day) in ["2026-01-10", "2026-01-11"].iter().enumerate() {
            let d = Date::parse(day).unwrap();
            app.publish(
                v.clone(),
                t.clone(),
                TimeOfDay::parse("21:00").unwrap(),
                d.clone(),
                2,
                4,
            )
            .unwrap();
            let ReserveOutcome::Reserved(id) = app.reserve(&s, &v, t.clone(), d.clone(), 2) else {
                panic!("reserve {i}");
            };
            app.no_show(&id, &DateTime::parse(&format!("{day}T22:00")).unwrap())
                .unwrap();
        }
        let d = Date::parse("2026-01-12").unwrap();
        app.publish(
            v.clone(),
            t.clone(),
            TimeOfDay::parse("21:00").unwrap(),
            d.clone(),
            2,
            4,
        )
        .unwrap();
        assert_eq!(app.reserve(&s, &v, t, d, 2), ReserveOutcome::BadStanding);
    }

    #[test]
    fn empty_where_paths() {
        let mut app = memory_app();
        let v = VenueId::new("maido");
        let t = TimeOfDay::parse("19:30").unwrap();
        let d = Date::parse("2026-01-10").unwrap();
        assert_eq!(
            app.reserve(&SessionId("T9".into()), &v, t.clone(), d.clone(), 2),
            ReserveOutcome::Unauthorized
        );
        app.register("alice", "x").unwrap();
        let s = app.login("alice", "x").unwrap();
        assert_eq!(app.reserve(&s, &v, t, d, 2), ReserveOutcome::NoSlot);
    }

    /// 架构看护：四个 BC crate 不许互相依赖。
    #[test]
    fn context_crates_are_independent() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let names = ["identity", "availability", "booking", "reputation"];
        for name in names {
            let toml = fs::read_to_string(root.join(format!("{name}/Cargo.toml"))).unwrap();
            for other in names {
                if other != name {
                    assert!(
                        !toml.contains(&format!("restaurant-ddd-{other}")),
                        "{name} must not depend on {other}"
                    );
                }
            }
        }
    }
}
