//! 【小白】这是收银台：把你在终端打的字，交给门口的人去办。
//!
//! 本文件几乎没有「预约规则」。规则在 `syncs` 和四本账里。
//!
//! 看三个地方就够：
//! - `open_app`：打开四个互不相连的 SQLite 文件（四本账）
//! - `demo`：完整故事。盯第 5、6 步
//! - `toggle`：手动把空位牌翻面（默认订位流程不会自动翻）
//!
//! 导读：仓库根目录 `START-HERE.md`。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use restaurant_authenticating::{Authenticating, SessionId, SqliteAuthStore};
use restaurant_availability::{Availability, SlotId, SqliteSlotStore};
use restaurant_karma::{Karma, SqliteKarmaStore};
use restaurant_reserving::{ReservationId, Reserving, SqliteReservationStore};
use restaurant_shared_kernel::{Date, DateTime, ParseError, TimeOfDay};
use restaurant_syncs::{ReserveOutcome, RestaurantSyncs, VenueId};

/// 四个概念 × 四个 SQLite 文件。UserId/SlotId 只在类型上交叉，表不交叉。
type App = RestaurantSyncs<
    SqliteSlotStore<VenueId>,
    SqliteReservationStore<restaurant_authenticating::UserId, SlotId>,
    SqliteAuthStore,
    SqliteKarmaStore<restaurant_authenticating::UserId>,
>;

/// 【小白】准备四本账。注意是四个文件，不是一张总表。
fn open_app(dir: &Path) -> App {
    fs::create_dir_all(dir).expect("create data dir");
    let availability = Availability::new(
        SqliteSlotStore::open(dir.join("availability.sqlite")).expect("availability db"),
    );
    let reserving = Reserving::new(
        SqliteReservationStore::open(dir.join("reserving.sqlite")).expect("reserving db"),
    );
    let authenticating =
        Authenticating::new(SqliteAuthStore::open(dir.join("authenticating.sqlite")).expect("auth db"));
    let karma = Karma::new(SqliteKarmaStore::open(dir.join("karma.sqlite")).expect("karma db"));
    RestaurantSyncs::new(availability, reserving, authenticating, karma)
}

fn parse_data_dir(args: &[String]) -> (PathBuf, &[String]) {
    if args.len() >= 2 && args[0] == "--data" {
        (PathBuf::from(&args[1]), &args[2..])
    } else {
        (PathBuf::from("data/concept"), args)
    }
}

fn usage() -> &'static str {
    "restaurant-concept [--data DIR] <command>

命令:
  demo
  register USER PASS
  login USER PASS
  publish VENUE START END DATE MIN MAX
  available VENUE TIME DATE PARTY
  reserve SESSION VENUE TIME DATE PARTY
  cancel RESERVATION
  grant RESERVATION NOW
  noshow RESERVATION NOW
  toggle SLOT

SQLite 文件（按概念分库，无跨概念外键）:
  DIR/availability.sqlite
  DIR/reserving.sqlite
  DIR/authenticating.sqlite
  DIR/karma.sqlite
"
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
            let user = app
                .register(&args[1], &args[2])
                .map_err(|e| format!("{e:?}"))?;
            println!("user {user}");
            Ok(())
        }
        "login" => {
            need(args, 3)?;
            let session = app
                .login(&args[1], &args[2])
                .map_err(|e| format!("{e:?}"))?;
            println!("session {session}");
            Ok(())
        }
        "publish" => {
            need(args, 7)?;
            let slot = app
                .publish(
                    VenueId::new(&args[1]),
                    TimeOfDay::parse(&args[2]).map_err(pe)?,
                    TimeOfDay::parse(&args[3]).map_err(pe)?,
                    Date::parse(&args[4]).map_err(pe)?,
                    args[5].parse().map_err(|_| "min party")?,
                    args[6].parse().map_err(|_| "max party")?,
                )
                .map_err(|e| format!("{e:?}"))?;
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
            let out = app.reserve(
                &SessionId(args[1].clone()),
                &VenueId::new(&args[2]),
                TimeOfDay::parse(&args[3]).map_err(pe)?,
                Date::parse(&args[4]).map_err(pe)?,
                args[5].parse().map_err(|_| "party")?,
            );
            println!("{}", format_reserve(&out));
            Ok(())
        }
        "cancel" => {
            need(args, 2)?;
            app.cancel(&ReservationId(args[1].clone()))
                .map_err(|e| format!("{e:?}"))?;
            println!("canceled");
            Ok(())
        }
        "grant" => {
            need(args, 3)?;
            app.grant(
                &ReservationId(args[1].clone()),
                &DateTime::parse(&args[2]).map_err(pe)?,
            )
            .map_err(|e| format!("{e:?}"))?;
            println!("granted");
            Ok(())
        }
        "noshow" => {
            need(args, 3)?;
            app.no_show(
                &ReservationId(args[1].clone()),
                &DateTime::parse(&args[2]).map_err(pe)?,
            )
            .map_err(|e| format!("{e:?}"))?;
            println!("no-show recorded; karma punished");
            Ok(())
        }
        "toggle" => {
            need(args, 2)?;
            app.availability
                .toggle_available(&SlotId(args[1].clone()))
                .map_err(|e| format!("{e:?}"))?;
            println!("toggled");
            Ok(())
        }
        _ => Err(usage().to_string()),
    }
}

/// 【小白】把论文里的餐厅故事演一遍。和第 5、6 步对照 DDD 的 demo。
fn demo(app: &mut App) -> Result<(), String> {
    println!("=== 概念设计演示（论文餐厅案例）===");
    let venue = VenueId::new("maido");
    let date = Date::parse("2026-01-10").unwrap();
    let time = TimeOfDay::parse("19:30").unwrap();
    let slot = app
        .publish(
            venue.clone(),
            time.clone(),
            TimeOfDay::parse("21:00").unwrap(),
            date.clone(),
            2,
            6,
        )
        .map_err(|e| format!("{e:?}"))?;
    println!("1. 发布 Maido 时段 → slot {slot}");

    app.register("alice", "secret").ok();
    app.register("bob", "secret").ok();
    let alice = app.login("alice", "secret").map_err(|e| format!("{e:?}"))?;
    let bob = app.login("bob", "secret").map_err(|e| format!("{e:?}"))?;
    println!("2. Alice / Bob 注册并登录");

    let found = app.available(&venue, &time, &date, 2);
    println!("3. 可订查询 → {found:?}");

    let r1 = app.reserve(&alice, &venue, time.clone(), date.clone(), 2);
    println!("4. Alice 预约 → {}", format_reserve(&r1));

    // 【小白】订完再搜：空位牌没被摘，所以还在。DDD 那边这里会变成空。
    let found_after = app.available(&venue, &time, &date, 2);
    println!("5. 预约后可订查询仍为 {found_after:?}");
    println!("   （Availability 不读取 Reserving：预测与承诺分叉）");

    let r2 = app.reserve(&bob, &venue, time.clone(), date.clone(), 2);
    // 【小白】Bob 不是「找不到位子」，而是「订座本说这份承诺已经给过了」。
    println!("6. Bob 再订同一时段 → {}", format_reserve(&r2));

    if let ReserveOutcome::Reserved(id) = &r1 {
        app.grant(id, &DateTime::parse("2026-01-10T19:35").unwrap())
            .map_err(|e| format!("{e:?}"))?;
        println!("7. Alice 当晚到场 grant → granted");
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
        app.no_show(id, &DateTime::parse("2026-01-11T22:00").unwrap())
            .map_err(|e| format!("{e:?}"))?;
        println!("   no-show → Karma.punish（第 1 次）");
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
        app.no_show(id, &DateTime::parse("2026-01-12T22:00").unwrap())
            .map_err(|e| format!("{e:?}"))?;
        println!("   no-show → Karma.punish（第 2 次，standing 变坏）");
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

    /// 内存库互不共享：四个 `Connection::open_in_memory()`，等价于四个文件。
    fn memory_app() -> App {
        RestaurantSyncs::new(
            Availability::new(SqliteSlotStore::memory().unwrap()),
            Reserving::new(SqliteReservationStore::memory().unwrap()),
            Authenticating::new(SqliteAuthStore::memory().unwrap()),
            Karma::new(SqliteKarmaStore::memory().unwrap()),
        )
    }

    /// Reserving OP：reserve 成功后，到场日可以 grant。
    #[test]
    fn op_reserve_then_grant() {
        let mut app = memory_app();
        app.register("alice", "x").unwrap();
        let s = app.login("alice", "x").unwrap();
        let v = VenueId::new("maido");
        app.publish(
            v.clone(),
            TimeOfDay::parse("19:30").unwrap(),
            TimeOfDay::parse("21:00").unwrap(),
            Date::parse("2026-01-10").unwrap(),
            2,
            4,
        )
        .unwrap();
        let ReserveOutcome::Reserved(id) = app.reserve(
            &s,
            &v,
            TimeOfDay::parse("19:30").unwrap(),
            Date::parse("2026-01-10").unwrap(),
            2,
        ) else {
            panic!("expected reserved");
        };
        app.grant(&id, &DateTime::parse("2026-01-10T19:40").unwrap())
            .unwrap();
    }

    /// 和 DDD 测试 `reserve_hides_slot_on_restaurant_aggregate` 对打：
    /// 这里搜索仍在，拒绝原因是 Conflict 不是 NoSlot。
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
        app.publish(v.clone(), t.clone(), TimeOfDay::parse("21:00").unwrap(), d.clone(), 2, 4)
            .unwrap();
        assert_eq!(app.available(&v, &t, &d, 2).len(), 1);
        assert!(matches!(
            app.reserve(&a, &v, t.clone(), d.clone(), 2),
            ReserveOutcome::Reserved(_)
        ));
        assert_eq!(
            app.available(&v, &t, &d, 2).len(),
            1,
            "Availability 仍列出已承诺 slot"
        );
        assert_eq!(
            app.reserve(&b, &v, t, d, 2),
            ReserveOutcome::Conflict
        );
    }

    /// where 零行：无效 session、以及有用户但没有时段。
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
        assert_eq!(
            app.reserve(&s, &v, t, d, 2),
            ReserveOutcome::NoSlot
        );
    }

    /// Fig.3 punishNoShow：两次后 standing 变坏，第三次预约被挡在 reserve 之前。
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
        assert_eq!(
            app.reserve(&s, &v, t, d, 2),
            ReserveOutcome::BadStanding
        );
    }

    /// 架构看护：读 Cargo.toml，禁止概念 crate 互相依赖。
    #[test]
    fn concept_crates_are_independent() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let names = ["reserving", "availability", "authenticating", "karma"];
        for name in names {
            let toml = fs::read_to_string(root.join(format!("concepts/{name}/Cargo.toml"))).unwrap();
            for other in names {
                if other != name {
                    assert!(
                        !toml.contains(&format!("restaurant-{other}")),
                        "{name} must not depend on {other}"
                    );
                }
            }
        }
    }
}
