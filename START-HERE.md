# 从零看懂这套代码（小白版）

不会概念设计、不会 DDD、Rust 也刚入门，可以从这里开始。  
更短的论文对照表在 [`README.md`](README.md)。术语表在本页后半。

## 先忘掉「类」

日常写业务，脑子里常是：

> 有一个 User，有一个 Restaurant。预约就是 `user.reserve(restaurant)`。

这套对照要让你看见：**「预约」这件事，不一定要长在 User 或 Restaurant 身上。**  
也要看见两句反例：

- **只把类拆到不同文件夹**，仍然是面向对象旧模式。
- **把 DDD 拆成独立限界上下文**，仍然不是概念设计。

用生活类比：

| 工作 | 像什么 | 概念设计 | 朴素 DDD / 面向对象分包 | 模块化 DDD |
|---|---|---|---|---|
| 登录 | 前台核对工牌 | 单独一本「工牌账」 | 塞进 User（分包后仍在 `user` crate） | Identity 上下文 |
| 看今晚还有没有位 | 门口的空位牌 | 单独一本「空位牌」 | 塞进 Restaurant，焊上占用 | Availability 上下文（不管占用） |
| 给你一张预约确认 | 订座本上盖章 | 单独一本「订座本」 | User + Restaurant + Reservation 一起改 | Booking：客人 + 占用 + 订单，外加领域服务 |
| 放鸽子扣信用 | 信誉小黑板 | 单独一块「小黑板」 | 还是塞进 User | Reputation 上下文 |

概念设计 = **四本互不翻看的账**。要联动，由门口的人（`syncs`）按规矩问完再盖章。  
朴素 DDD = **一本通讯录**：人、店、位子都互相指着。  
面向对象分包 = **同一本通讯录，按名词撕成几本册**。册与册互相 import，表仍用外键焊在一起。  
模块化 DDD = **四本分册通讯录**：册与册不互引，但 Booking 册里仍是对象拼图；门口换成用例编排，不是 sync。

## 你只要先做这一件事

打开终端，跑四遍同一出戏（Alice 订位，Bob 再订同一时段）：

```bash
cd beyond-objects-case-study/concept
cargo run -p restaurant-concept -- --data /tmp/c demo

cd ../ddd-clean
cargo run -- --data /tmp/d demo

cd ../oop-packages
cargo run -p restaurant-oop-packages -- --data /tmp/o demo

cd ../ddd-modular
cargo run -p restaurant-ddd-modular -- --data /tmp/m demo
```

**死盯第 5 步和第 6 步：**

- 概念设计：Alice 订完，搜索**还看得到**那个时段；Bob 再订，电脑说 **Conflict**。
- 朴素 DDD：Alice 订完，搜索**变空**；Bob 再订，电脑说 **NoSlot**。
- 面向对象分包：和第 2 栏**一样**（变空 + NoSlot）。目录拆开了，切法没变。
- 模块化 DDD：第 5、6 步**对外可以和概念侧相像**（还能搜到，Bob 得 Conflict）。差别必须打开代码：Conflict 来自 `BookingService` 同时改 Guest / Occupancy / Reservation，不是一台 Reserving 机器。

同一家餐厅、同一对客人。这不是 bug，是**切法不同**。看懂这一点，后面的目录就有意义了。

## 目录用大白话

```
beyond-objects-case-study/
  START-HERE.md     ← 你在这里
  concept/          ← 概念：四本账 + 门口规矩（syncs）
  ddd-clean/        ← 朴素 DDD：人/店/订单对象 + 单库外键
  oop-packages/     ← 同一套对象，只按名词拆 crate
  ddd-modular/      ← 模块化 DDD：四个限界上下文 + 用例编排
```

`concept/` 里面：

```
app/        收银台：把你在命令行打的字变成一次「请办事」
syncs/      门口的人：先问工牌、再问信誉、再看空位牌，最后才去订座本盖章
concepts/   四本账，互相不许偷看
  authenticating/   工牌
  availability/     空位牌
  reserving/        订座本
  karma/            信誉小黑板
```

`ddd-modular/` 里面（包名刻意不和概念 crate 同名）：

```
app/            收银台 + 用例编排（像 syncs，但不是）
identity/       工牌（User 只有用户名密码）
availability/   空位牌（listed，没有占用）
booking/        订位碎片：Guest + SlotOccupancy + Reservation + BookingService
reputation/     信誉小黑板
```

| 概念 crate | 模块化 DDD crate |
|---|---|
| `authenticating` | `identity` |
| `availability` | `availability`（名字碰巧相同，里面是 Slot 聚合，不是概念机） |
| `reserving` | `booking` |
| `karma` | `reputation` |
| `syncs` | `app/src/orchestration.rs` |

每个 `ddd-modular` 上下文里认这几个文件：

| 文件 | 小白说法 |
|---|---|
| `domain.rs` | 这个上下文有哪些对象、错误叫什么 |
| `ports.rs` | 「请帮我存/取」的说明书 |
| `service.rs` | 本上下文的应用服务（对内改聚合） |
| `sqlite.rs` | 真的把字写进本上下文的 SQLite 文件 |
| `booking_service.rs` | **只有 Booking 有**：同时改三块碎片的领域服务 |

`ddd-clean/` 里面（常见公司写法）：

```
domain/         业务对象：User、Restaurant、Reservation
application/    用例：把对象从数据库拿出来、改完再存回去
infrastructure/ 一个 restaurant.sqlite，表和表用外键牵在一起
main.rs         命令行
```

`oop-packages/` 里面（只是把上面的类各放一个 crate）：

```
user/           User：密码 + karma + booked_dates
restaurant/     Restaurant + Slot.reserved
reservation/    Reservation（Cargo.toml 依赖 user、restaurant）
booking/        BookingService 同时改三个对象包
app/            单库 restaurant.sqlite + CLI
```

## 建议点开的文件（按这个顺序）

1. `concept/app/src/main.rs` 里的 `demo` —— 故事剧本  
2. `concept/syncs/src/lib.rs` 里的 `reserve` —— 门口的人怎么问三句话  
3. `concept/concepts/reserving/src/actions.rs` —— 订座本自己的规矩  
4. `ddd-clean/src/domain/booking.rs` —— 朴素 DDD 一次改三个对象  
5. `oop-packages/booking/src/lib.rs` —— **还是一次改三个对象，只是三个 crate**  
6. `oop-packages/reservation/Cargo.toml` —— 订单包依赖人和店：对象图穿过包边界  
7. `ddd-modular/app/src/orchestration.rs` —— **这里像 syncs，但不是**  
8. `ddd-modular/booking/src/booking_service.rs` —— **这里像 Reserving，但不是**

文件头都有 `【小白】` 段落，先读那段，再往下看代码。

## 代码里会撞到的 Rust 词（碰到再查）

- **crate**：一个可编译的小项目。概念侧「一个概念 = 一个 crate」；面向对象分包「一个名词类 = 一个 crate」；模块化 DDD「一个限界上下文 = 一个 crate」。
- **trait**：说明书。「存储只要能 insert/get」，具体是 SQLite 还是内存都可以。
- **enum**：结果的几种可能。`Conflict` 不是崩溃，是一种正式答复。
- **Option**：有或没有。`None` = 没查到，不一定是出错。
- **Result**：成功或失败。`Err` 是业务拒绝或格式错误。
- **PhantomData**：结构体上挂着「我跟哪种 ID 一起用」，但口袋里不真塞一个 ID。跳过也没关系。

## 不要在这里学什么

密码请不要学这套（明文存放）。也没有支付、没有排桌、没有网站页面。  
它只是把论文里的餐厅例子做成**能跑的四套结构**，方便你对照。
