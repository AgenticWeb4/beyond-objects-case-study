# 餐厅预约：概念设计模型

> 来源：Daniel Jackson, *Beyond Objects*，[arXiv:2606.27258](https://arxiv.org/abs/2606.27258) §5–13。  
> 配套规格见 [`concept/docs/prd/README.md`](concept/docs/prd/README.md) 与模块旁的 CONCEPT/SYNCS 文档。  
> 论文 Fig.1–3 为简化规格；下列给出本案例的概念与同步规格，并标明推断与未决。

## 应用目的与场景

**受益者：** 食客（要入座承诺或明确拒绝）；餐厅经营者（发布可订时段）。

**应用目的：** 食客凭会话在指定餐厅的日期、时间与人数下，获得后续入座承诺，或得到可区分的拒绝；未履约（no-show）会降低后续预约资格。

**现状（论文对象切分）：** 把 `reserve` 派给 User / Slot / Reservation / Restaurant 都说得通，也都不对；双不变量（一槽一约、同时段一人一约）把功能撕碎；User 再叠认证与 karma，成为系统指纹。见论文 §7–8。

**约束：** 本演示不建模桌台分配、超售、鉴权协议或支付。时段预测与入座承诺允许分叉（论文 §13）。

**来源：** 应用目的由论文餐厅案例与 Fig.3 的 `makeReservation` / `punishNoShow` **推断**为可评价产品目标。论文未写完整产品范围。

### 端到端场景

**S1 成功预约（兑现应用目的）**  
条件：Alice 已登录且 karma 良好；Maido 在 2026-01-10 19:30 有可订时段、人数 2 落在 min–max。  
动作：`Requesting/reserve` → where 解析 session、karma、slot → `Reserving/reserve` 成功 → `Requesting/respond` 返回预约身份。  
结果：存在 booked 预约，by Alice、for 该 slot。  
依据：论文 Fig.3 `makeReservation`；Reserving OP。

**S2 时段已被承诺（Conflict）**  
条件：同 slot 已有 booked 预约。  
动作：再次 `Reserving/reserve` → `[error: Conflict]`，状态不变。  
结果：请求得到 Conflict。  
依据：Reserving.reserve 冲突 case。Availability 仍可能显示该 slot（零同步预测与承诺分叉）。

**S3 无匹配时段（where 零行）**  
条件：该餐厅/时间/人数下没有 available slot。  
动作：`makeReservation` 的 where 为空，不调用 `Reserving/reserve`。  
结果：`Requesting/respond` 返回 `NoSlot`。  
依据：论文 where 语义 + 本仓「需响应入口必须覆盖零行」约定。

**S4 会话无效**  
条件：session 无法解析为用户。  
结果：`Unauthorized`，不写预约。  
依据：Fig.3 `UserAuthentication._getUser`；零行策略同上。

**S5 karma 不良**  
条件：用户不在 good standing。  
结果：`BadStanding`，不写预约。  
依据：Fig.3 `Karma._inGoodStanding`。

**S6 no-show 惩罚后再次预约被拒**  
条件：Alice 有 booked 预约；当前时间已过承诺时间；连续惩罚使 standing 变坏。  
动作：`Reserving/noShow` → `punishNoShow` → `Karma/punish`。  
结果：再次 `reserve` 走 S5。  
依据：Fig.3 `punishNoShow`；惩罚阈值为本演示确认（见下）。

**S7 到场兑现**  
条件：预约为 booked，当前日期接近承诺日期。  
动作：`Reserving/grant`。  
结果：status=granted，兑现 Reserving 目的。

**S8 取消**  
条件：预约为 booked。  
动作：`Reserving/cancel`。  
结果：status=canceled；该 slot 可被他人重新承诺（canceled 释放 slot：**确认**，论文未写）。

### Misfits

- 搜索仍列出已被承诺的 slot（S2）：预测与承诺分叉。这是论文显式允许的产品形态，不是实现缺陷；若产品要隐藏，应加 sync，而不是让 Availability 读取 Reserving。
- 桌台分配、迟到改承诺时间、同一晚两家餐厅：论文 §13 列为 underspecified / 另一概念；本模型排除。
- Fig.2 的 `_getAvailableSlot` 签名未含 venue，但 Fig.3 与正文要求按餐厅匹配。**推断并确认**：query 增加 `venue`。

---

## Concepts

# concept Reserving [U, S]

## purpose
为用户保留未来使用某资源时段的承诺。

## state
reservations: set Reservation
user: Reservation -> U
slot: Reservation -> S
time: Reservation -> Time
date: Reservation -> Date
partySize: Reservation -> Number
status: Reservation -> Status

Reservation 由本概念创建。Status = booked | canceled | granted。  
初态 reservations 为空；每项预约恰有一个 user、slot、time、date、partySize、status。  
不变量：任一 S 最多对应一个 status=booked 的预约；任一 (U, Date) 最多对应一个 status=booked 的预约。  
canceled / granted 不占用上述唯一性（释放承诺）。Time / Date / Number 为值，不是类型参数。

## actions
reserve [user: U; slot: S; time: Time; date: Date; partySize: Number] => [reservation: Reservation]
  requires 该 slot 无 booked 预约，且该 user 在该 date 无 booked 预约
  ensures 原子创建 status=booked 的预约并返回身份；其他预约不变

reserve [user: U; slot: S; time: Time; date: Date; partySize: Number] => [error: Conflict]
  requires 该 slot 已有 booked 预约，或该 user 在该 date 已有 booked 预约
  ensures 返回 Conflict；状态不变

cancel [reservation: Reservation] => []
  requires 预约存在且 status=booked
  ensures 仅将该预约改为 canceled

cancel [reservation: Reservation] => [error: NotBooked]
  requires 预约不存在，或 status ≠ booked
  ensures 返回 NotBooked；状态不变

grant [reservation: Reservation; now: DateTime] => []
  requires 预约存在且 status=booked，且 now 的日期等于预约 date
  ensures 仅将该预约改为 granted

grant [reservation: Reservation; now: DateTime] => [error: NotGrantable]
  requires 预约不存在，或 status ≠ booked，或 now 的日期不等于预约 date
  ensures 返回 NotGrantable；状态不变

noShow [reservation: Reservation; now: DateTime] => []
  requires 预约存在且 status=booked，且 now 晚于预约的 (date, time)
  ensures 仅将该预约改为 canceled

noShow [reservation: Reservation; now: DateTime] => [error: NotNoShow]
  requires 预约不存在，或 status ≠ booked，或 now 不晚于预约的 (date, time)
  ensures 返回 NotNoShow；状态不变

## operational principle
after reserve [user: u; slot: s; time: t; date: d; partySize: n] => [reservation: r]
then 在预约仍为 booked、且到场日期为 d 时 grant [reservation: r; now: d+t] => []，使用者获得对该资源的承诺兑现。

## queries
_user [reservation: Reservation] => [user: U]
  returns 该预约的 user；无匹配为零行

论证：reserve 创建 booked 承诺；grant 在到场日把承诺标为已兑现，OP 的结果满足「为用户保留后续使用」的 purpose。cancel / noShow 把 booked 改为 canceled，从而释放 slot 与当日唯一性，使他人或次日可再预约。  
反例：若 grant 不存在，则只有「记下预约」而无兑现，purpose 落空——因此 grant 不可省略。  
未决映射：论文用 canceled/granted 两枚 flag 与 firing condition；本模型改为 Status 与显式 error case，行为等价于「前置不满足则动作不发生」。  
「同日一人一约」是对论文 “time close to time” 的**确认简化**（论文承认窗口未定）。

边界：专一——承诺，不含搜索或惩罚；完整——reserve/cancel/grant/noShow 覆盖承诺生命周期；独立——U、S 仅为身份；熟悉——预约。保留。

---

# concept Availability [V]

## purpose
让使用者找到指定场所当前提供的资源时段。

## state
slots: set Slot
venue: Slot -> V
start: Slot -> Time
end: Slot -> Time
date: Slot -> Date
minParty: Slot -> Number
maxParty: Slot -> Number
available: set Slot

Slot 由本概念创建。初态 slots/available 为空。  
每个 Slot 恰有一个 venue、start、end、date、minParty、maxParty。  
available ⊆ slots。available 表示预测仍对外提供，不表示未被承诺。

## actions
create [venue: V; start: Time; end: Time; date: Date; minParty: Number; maxParty: Number] => [slot: Slot]
  requires end 晚于 start，且 maxParty ≥ minParty
  ensures 创建新 Slot，写入各关系，并加入 available；其他 slot 不变

create [venue: V; start: Time; end: Time; date: Date; minParty: Number; maxParty: Number] => [error: InvalidWindow]
  requires end 不晚于 start，或 maxParty < minParty
  ensures 返回 InvalidWindow；状态不变

toggleAvailable [slot: Slot] => []
  requires slot 属于 slots
  ensures slot 在 available 中则移除，否则加入；其他 slot 不变

toggleAvailable [slot: Slot] => [error: NotFound]
  requires slot 不属于 slots
  ensures 返回 NotFound；状态不变

## operational principle
after create [venue: v; start: t; end: e; date: d; minParty: m; maxParty: n] => [slot: s]
then s 在 available 中且 venue(s)=v；_getAvailableSlot 在匹配的时间与人数下返回 s。toggleAvailable [slot: s] => [] 后，同一查询不再返回 s，除非再次 toggle。

## queries
_getAvailableSlot [venue: V; time: Time; date: Date; partySize: Number] => [slot: Slot]
  returns 满足全部条件的 available slot，每项一行：venue 匹配、start=time、date 匹配、minParty ≤ partySize ≤ maxParty。无匹配为零行；多匹配为多行。

论证：create 产生可查询时段；toggle 把「仍提供」与「已撤下」分开，OP 展示找到与撤下，兑现「找到当前提供的时段」。本概念不读取预约。  
反例：若把「已被 reserve」写进 available 的维护，则预测机器依赖承诺机器，独立性被破坏。

边界：专一——可用性预测；完整——发布、撤下、查询；独立——只依赖场所身份 V；熟悉——可订时段。保留。  
query 增 venue：相对 Fig.2 的签名补丁，依据正文「matching available slot for the given restaurant」。

---

# concept Authenticating [U]

## purpose
确认当前请求由哪个用户身份发出。

## state
users: set U
username: U -> Username
password: U -> Password
sessions: set Session
sessionUser: Session -> U

U 与 Session 由本概念创建。Username 唯一。初态皆空。  
每个 session 恰对应一个 U。

## actions
register [username: Username; password: Password] => [user: U]
  requires 该 username 未被使用
  ensures 创建用户并保存凭据

register [username: Username; password: Password] => [error: UsernameTaken]
  requires username 已存在
  ensures 返回 UsernameTaken；状态不变

login [username: Username; password: Password] => [session: Session]
  requires 存在匹配的 username 与 password
  ensures 创建新 session 并绑定该用户

login [username: Username; password: Password] => [error: InvalidCredentials]
  requires 无匹配凭据
  ensures 返回 InvalidCredentials；状态不变

logout [session: Session] => []
  requires session 属于 sessions
  ensures 删除该 session

logout [session: Session] => [error: NotFound]
  requires session 不属于 sessions
  ensures 返回 NotFound；状态不变

## operational principle
after register [username: n; password: p] => [user: u]
and login [username: n; password: p] => [session: s]
then _getUser [session: s] => [user: u]。

## queries
_getUser [session: Session] => [user: U]
  returns 该 session 绑定的用户；无效 session 为零行

论证：register+login 后 query 能解析身份，兑现「确认谁在发请求」。凭据仅存于此概念，不进入 Reserving。  
边界：专一——认证；完整——注册/登录/解析；独立——U 为身份；熟悉——登录。保留。  
论文名 UserAuthentication；本模型用更短的 Authenticating。密码以明文存于内存演示库，**不是**安全设计。

---

# concept Karma [U]

## purpose
根据用户过往行为决定其是否仍有资格使用需守信的功能。

## state
strikes: U -> Number
初态：无记录的 U 视为 0。Number ≥ 0。  
good standing 当且仅当 strikes(u) < 2。阈值 2 为演示**确认**，论文只写 “hasn’t been punished too many times”。

## actions
punish [user: U] => []
  requires true
  ensures strikes(u) 增加 1；无记录则视为从 0 增至 1。其他用户不变

## operational principle
after punish [user: u] => [] 发生两次
then _inGoodStanding [user: u] 为零行（不再处于 good standing）。

## queries
_inGoodStanding [user: U] => [user: U]
  returns 若该用户 strikes < 2 则一行（回显 user）；否则零行。无记录视为 0 strike，故为一行。

论证：两次 no-show 惩罚后 query 为空，应用即可拒绝预约，兑现「资格随失信下降」。Karma 不调用 Reserving。  
边界：专一——声誉资格；完整——惩罚与查询足以支持 Fig.3；独立——只依赖 U；熟悉——信誉。保留。  
reward / 恢复未建模（排除）。

---

## Synchronizations

外部入口 Requesting（伪概念，封装 CLI；非独立 CONCEPT）：

```text
Requesting/register: [username: Username; password: Password] => [request: RequestId]
Requesting/login: [username: Username; password: Password] => [request: RequestId]
Requesting/publish: [venue: VenueId; start: Time; end: Time; date: Date; minParty: Number; maxParty: Number] => [request: RequestId]
Requesting/reserve: [session: Session; venue: VenueId; time: Time; date: Date; partySize: Number] => [request: RequestId]
Requesting/cancel: [reservation: Reservation] => [request: RequestId]
Requesting/grant: [reservation: Reservation; now: DateTime] => [request: RequestId]
Requesting/noShow: [reservation: Reservation; now: DateTime] => [request: RequestId]
Requesting/respond: [request: RequestId; ...] => []
```

`VenueId` 为不透明场所身份，由经营者在 publish 时提供，不另建 RestaurantCatalog（论文提及但本 MVP **排除**）。

```text
# app RestaurantReservations
include Availability [VenueId]
include Reserving [Authenticating.U, Availability.Slot]
include Authenticating
include Karma [Authenticating.U]
include Requesting

// flow: identity
sync registerUser
when {
  Requesting/register: [username: ?n; password: ?p] => [request: ?request]
}
then {
  Authenticating/register: [username: ?n; password: ?p]
}

sync registered
when {
  Requesting/register: [username: ?n; password: ?p] => [request: ?request]
  Authenticating/register: [username: ?n; password: ?p] => [user: ?user]
}
then {
  Requesting/respond: [request: ?request; user: ?user]
}

sync registerRejected
when {
  Requesting/register: [username: ?n; password: ?p] => [request: ?request]
  Authenticating/register: [username: ?n; password: ?p] => [error: ?error]
}
then {
  Requesting/respond: [request: ?request; error: ?error]
}

sync loginUser
when {
  Requesting/login: [username: ?n; password: ?p] => [request: ?request]
}
then {
  Authenticating/login: [username: ?n; password: ?p]
}

sync loggedIn
when {
  Requesting/login: [username: ?n; password: ?p] => [request: ?request]
  Authenticating/login: [username: ?n; password: ?p] => [session: ?session]
}
then {
  Requesting/respond: [request: ?request; session: ?session]
}

sync loginRejected
when {
  Requesting/login: [username: ?n; password: ?p] => [request: ?request]
  Authenticating/login: [username: ?n; password: ?p] => [error: ?error]
}
then {
  Requesting/respond: [request: ?request; error: ?error]
}

// flow: publish
sync publishSlot
when {
  Requesting/publish: [venue: ?v; start: ?s; end: ?e; date: ?d; minParty: ?min; maxParty: ?max] => [request: ?request]
}
then {
  Availability/create: [venue: ?v; start: ?s; end: ?e; date: ?d; minParty: ?min; maxParty: ?max]
}

sync published
when {
  Requesting/publish: [venue: ?v; start: ?s; end: ?e; date: ?d; minParty: ?min; maxParty: ?max] => [request: ?request]
  Availability/create: [venue: ?v; start: ?s; end: ?e; date: ?d; minParty: ?min; maxParty: ?max] => [slot: ?slot]
}
then {
  Requesting/respond: [request: ?request; slot: ?slot]
}

sync publishRejected
when {
  Requesting/publish: [venue: ?v; start: ?s; end: ?e; date: ?d; minParty: ?min; maxParty: ?max] => [request: ?request]
  Availability/create: [venue: ?v; start: ?s; end: ?e; date: ?d; minParty: ?min; maxParty: ?max] => [error: ?error]
}
then {
  Requesting/respond: [request: ?request; error: ?error]
}

// flow: reserve  （论文 Fig.3 makeReservation）
sync makeReservation
when {
  Requesting/reserve: [session: ?session; venue: ?venue; time: ?time; date: ?date; partySize: ?n] => [request: ?request]
}
where {
  Authenticating/_getUser: [session: ?session] => [user: ?user]
  Karma/_inGoodStanding: [user: ?user] => [user: ?user]
  Availability/_getAvailableSlot: [venue: ?venue; time: ?time; date: ?date; partySize: ?n] => [slot: ?slot]
}
then {
  Reserving/reserve: [user: ?user; slot: ?slot; time: ?time; date: ?date; partySize: ?n]
}

sync reserved
when {
  Requesting/reserve: [session: ?session; venue: ?venue; time: ?time; date: ?date; partySize: ?n] => [request: ?request]
  Reserving/reserve: [user: ?user; slot: ?slot; time: ?time; date: ?date; partySize: ?n] => [reservation: ?reservation]
}
then {
  Requesting/respond: [request: ?request; reservation: ?reservation]
}

sync reserveConflict
when {
  Requesting/reserve: [session: ?session; venue: ?venue; time: ?time; date: ?date; partySize: ?n] => [request: ?request]
  Reserving/reserve: [user: ?user; slot: ?slot; time: ?time; date: ?date; partySize: ?n] => [error: ?error]
}
then {
  Requesting/respond: [request: ?request; error: ?error]
}

# where 零行：不触发 makeReservation。mediator 将零行映射为下列应用错误（非概念动作）：
#   无 user → Unauthorized
#   有 user 但 karma 空 → BadStanding
#   有 user 且 karma 通过但 slot 空 → NoSlot
#   slot 多行：按 Slot 身份字典序取一行，对应论文 “any available slot”

sync cancelReservation
when {
  Requesting/cancel: [reservation: ?r] => [request: ?request]
}
then {
  Reserving/cancel: [reservation: ?r]
}

sync grantReservation
when {
  Requesting/grant: [reservation: ?r; now: ?now] => [request: ?request]
}
then {
  Reserving/grant: [reservation: ?r; now: ?now]
}

sync markNoShow
when {
  Requesting/noShow: [reservation: ?r; now: ?now] => [request: ?request]
}
then {
  Reserving/noShow: [reservation: ?r; now: ?now]
}

// flow: karma  （论文 Fig.3 punishNoShow）
sync punishNoShow
when {
  Reserving/noShow: [reservation: ?r; now: ?now] => []
}
where {
  Reserving/_user: [reservation: ?r] => [user: ?user]
}
then {
  Karma/punish: [user: ?user]
}
```

失败策略：需响应的入口均有成功/拒绝分支；`punishNoShow` 为后台联动，无独立 HTTP 响应。Reserving.noShow 成功完成后才 punish；noShow 被拒绝则不惩罚。where 零行不调用 reserve（有意忽略写动作，改为入口错误）。不使用跨概念事务；各动作原子维护自身不变量。

未把「reserve 成功后 toggleAvailable」写入默认规则（论文 Fig.3 没有）。若加入则是收紧同步，另见排除表。

---

## 同步图与产品子集

同步图（规则节点）：

- `Requesting/register` → registerUser → `Authenticating/register` → registered | registerRejected → respond
- `Requesting/login` → loginUser → `Authenticating/login` → loggedIn | loginRejected → respond
- `Requesting/publish` → publishSlot → `Availability/create` → published | publishRejected → respond
- `Requesting/reserve` → makeReservation（where：Authenticating、Karma、Availability）→ `Reserving/reserve` → reserved | reserveConflict → respond
- `Requesting/cancel|grant|noShow` → 对应 Reserving 动作 → respond
- `Reserving/noShow` → punishNoShow（where：Reserving._user）→ `Karma/punish`

产品依赖（外在，应用选择）：

- Reserving → Availability（本产品要「按餐厅时间找时段再承诺」；已知 slot 身份的纯承诺产品可不纳入 Availability）
- makeReservation → Authenticating（本产品用 session 而非直接传 user）
- makeReservation → Karma（本产品用声誉门槛；无门槛产品可去掉 punishNoShow 与 where 中的 karma）

MVP：Availability + Reserving + Authenticating + Karma + 上述 reserve/noShow 规则。不含 RestaurantCatalog、桌台分配、评价、通知。

---

## 排除与未决

| 项 | 结论 | 来源 | 影响 |
|---|---|---|---|
| RestaurantCatalog | 排除 | 论文 §13 提及，非 Fig.1–3 | VenueId 由 publish 传入 |
| 桌台分配概念 | 排除 | 论文 §13 | 承诺可能无法物理入座 |
| reserve 成功后 toggleAvailable | 排除出默认产品 | Fig.3 无此规则 | 搜索可仍显示已承诺 slot |
| 评价 / Review | 排除出实现 | 论文 §9 只作 OO 导航反例 | DDD 对照文中叙述，不写进本概念应用 |
| 同日一人一约窗口 | 确认为同 Date | 论文 §13 underspecified | 同晚两家餐厅会被 Conflict |
| grant / noShow 时间窗 | 确认为：grant 同日；noShow 晚于承诺时刻 | 论文 underspecified | 演示用 `--now` |
| Karma 阈值 | 确认为 2 次 punish | 论文未给数字 | 两次 no-show 后拒绝 |
| 多 slot「any」 | 确认为按 SlotId 字典序取一 | 论文 “any” | 同时段多桌时确定性取一 |
| 密码存储 | 明文内存 | 演示 | 不可用于真实认证 |
| 并发重放 / 分布式 | 未决 | 单进程内存 | 仅顺序与 `&mut` 隔离 |

概念选择与 sync 支持 S1–S8；局部 OP 不推出桌台层目的已达成。
