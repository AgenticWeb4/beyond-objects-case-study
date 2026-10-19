# Beyond Objects Case Study：餐厅预约设计对照

**完全没接触过的人请先读 [`START-HERE.md`](START-HERE.md)**（大白话 + 先跑 demo）。  
已经知道概念/DDD 的人可以直接看下面的对照表。

独立可运行项目。论文：[Beyond Objects](https://arxiv.org/abs/2606.27258)（Daniel Jackson, arXiv:2606.27258）。

同一产品行为用四套结构实现，数据库均为 **SQLite**（`rusqlite` bundled，无需本机安装 sqlite）：

| 目录 | 切法 | 持久化 |
|---|---|---|
| [`concept/`](concept) | 概念设计：模块按行为划分，个体只出借身份 | **每概念一个 .sqlite**，无跨概念外键 |
| [`ddd-clean/`](ddd-clean) | 朴素 DDD + Clean：功能派给 User / Restaurant / Reservation | **单库 + 外键对象图** |
| [`oop-packages/`](oop-packages) | 面向对象分包：同一套对象，按名词拆 crate，crate 互依 | **单库 + 外键对象图**（与朴素 DDD 相同） |
| [`ddd-modular/`](ddd-modular) | 模块化 DDD：Identity / Availability / Booking / Reputation | **每上下文一个 .sqlite**，无跨上下文外键 |

概念规格：[`MODEL.md`](MODEL.md) → [`concept/docs/prd/README.md`](concept/docs/prd/README.md)。

## 为什么对照看得到

论文把 `reserve (User, Slot): (Reservation)` 派给任何一个类都不对（§7）。DDD 用更大的聚合和领域服务补洞（§10），切法仍绕着对象转。概念把承诺（Reserving）和可订预测（Availability）分成两台机器，用 sync 组合（§12–13）。

后两栏是为了说明两句反例：

- **概念设计 ≠ 把类拆到不同 crate。**（`oop-packages/`）
- **概念设计 ≠ 把 DDD 拆成独立限界上下文。**（`ddd-modular/`）

同一条 Alice 预约成功之后的搜索：

- **概念设计：** Availability 仍列出该 slot（预测与承诺允许分叉）。Bob 再订得到 `Conflict`。
- **朴素 DDD / 面向对象分包：** `Slot.reserved = true` 焊在 Restaurant 里，搜索变为空，Bob 得到 `NoSlot`。分包只改了目录。
- **模块化 DDD：** 搜索仍列出（占用只在 Booking 的 `SlotOccupancy`）。Bob 也得到 `Conflict`——来自 `BookingService` 同时改 Guest / Occupancy / Reservation，不是 Reserving 概念机。

## 运行

先克隆仓库，再进入各实现目录运行。每次 `demo` 建议用空目录，避免旧库干扰。

```bash
git clone https://github.com/AgenticWeb4/beyond-objects-case-study.git
cd beyond-objects-case-study
```

```bash
# 概念设计
cd concept
cargo test
cargo run -p restaurant-concept -- --data /tmp/restaurant-concept demo

# 朴素 DDD + Clean
cd ../ddd-clean
cargo test
cargo run -- --data /tmp/restaurant-ddd demo

# 面向对象分包（行为与上一栏相同）
cd ../oop-packages
cargo test
cargo run -p restaurant-oop-packages -- --data /tmp/restaurant-oop demo

# 模块化 DDD
cd ../ddd-modular
cargo test
cargo run -p restaurant-ddd-modular -- --data /tmp/restaurant-ddd-modular demo
```

手工命令（四套 CLI 相同）：

```text
register alice secret
login alice secret          # 打印 session
publish maido 19:30 21:00 2026-01-10 2 6
available maido 19:30 2026-01-10 2
reserve SESSION maido 19:30 2026-01-10 2
grant R1 2026-01-10T19:35
noshow R1 2026-01-10T22:00
```

## 结构对照

```text
concept/                ddd-clean/           oop-packages/         ddd-modular/
  reserving  承诺         domain/user.rs       user/                 identity/
  availability 预测       domain/restaurant    restaurant/           availability/
  authenticating          domain/reservation   reservation/          booking/
  karma                   domain/booking.rs    booking/              reputation/
  syncs                   application/         app/ 单 sqlite        app/orchestration
```

| 概念 crate | 模块化 DDD crate |
|---|---|
| authenticating | identity |
| availability | availability |
| reserving | booking |
| karma | reputation |
| syncs | app 用例编排 |

概念 crate **互不依赖**；跨概念规则只在 `syncs/`。  
面向对象分包的 crate **互相依赖**（`reservation` → `user` + `restaurant`；`booking` → 三个对象包）。  
模块化 DDD 的 BC crate **互不依赖**；跨上下文只在 `app/` 用例里调各上下文 API。  
SQLite 文件：

- 概念：`availability.sqlite` / `reserving.sqlite` / `authenticating.sqlite` / `karma.sqlite`
- 朴素 DDD / 面向对象分包：`restaurant.sqlite`（`users.karma_strikes`、`slots.reserved`、外键）
- 模块化 DDD：`identity.sqlite` / `availability.sqlite` / `booking.sqlite` / `reputation.sqlite`

## 论文原文与本实现

| 论文 | 本实现 |
|---|---|
| Fig.1 Reserving / Fig.2 Availability / Fig.3 makeReservation、punishNoShow | 已实现 |
| UserAuthentication、Karma | 概念侧 Authenticating、Karma；模块化 DDD 为 Identity、Reputation |
| RestaurantCatalog、桌台分配 | 排除（VenueId 由 publish 传入） |
| `_getAvailableSlot` 未写 venue | 按正文补上 venue |
| “close in time” 未定 | 确认为同一 Date |
| Karma 次数未给 | 2 次 punish 后拒绝 |

英文摘要（论文主张）：

> Rather than assigning function to the individuals of the problem domain, we introduce a new category of modules which we call concepts.

中文：不要把功能派给问题域个体；引入一类新模块，称为概念。
