# 餐厅预约 — 总体 PRD

权威规格落点。确认模型见 [`MODEL.md`](../../../MODEL.md)。来源：Daniel Jackson, *Beyond Objects* [arXiv:2606.27258](https://arxiv.org/abs/2606.27258) §5–13。

## 应用目的

食客凭会话在指定餐厅的日期、时间与人数下，获得后续入座承诺，或得到可区分的拒绝；未履约（no-show）会降低后续预约资格。

受益者：食客、发布时段的餐厅经营者。不建模桌台分配、超售、支付或评价。

## 端到端场景与依据

| ID | 场景 | 概念 / sync | 可观察结果 |
|---|---|---|---|
| S1 | 登录且 karma 良好，存在匹配时段 | `makeReservation` → `Reserving/reserve` 成功 | 返回 reservation |
| S2 | 同 slot 或同用户同日已有 booked | `reserveConflict` | Conflict；状态不变 |
| S3 | 无 available slot | where 零行，不调用 reserve | NoSlot |
| S4 | session 无效 | where 无 user | Unauthorized |
| S5 | karma 不在 good standing | where karma 零行 | BadStanding |
| S6 | booked 预约逾期 no-show | `markNoShow` + `punishNoShow` | 预约 canceled；strikes+1；两次后走 S5 |
| S7 | 到场日 grant | `grantReservation` | status=granted |
| S8 | 取消 booked | `cancelReservation` | status=canceled；slot 承诺释放 |

Misfits：默认产品不在 reserve 后 `toggleAvailable`，搜索仍可能列出已承诺 slot（论文允许预测与承诺分叉）。桌台层目的未纳入。

## 概念索引

| 概念 | purpose | 规格 |
|---|---|---|
| Reserving | 为用户保留未来使用某资源时段的承诺 | [CONCEPT](../../concepts/reserving/CONCEPT.md) |
| Availability | 让使用者找到指定场所当前提供的资源时段 | [CONCEPT](../../concepts/availability/CONCEPT.md) |
| Authenticating | 确认当前请求由哪个用户身份发出 | [CONCEPT](../../concepts/authenticating/CONCEPT.md) |
| Karma | 根据过往行为决定是否仍有资格使用需守信的功能 | [CONCEPT](../../concepts/karma/CONCEPT.md) |

同步：[SYNCS.md](../../syncs/SYNCS.md)

## 同步图

- `Requesting/register|login` → 身份规则 → `Authenticating` → respond
- `Requesting/publish` → `Availability/create` → respond
- `Requesting/reserve` → **makeReservation**（合取 where：Authenticating、Karma、Availability）→ `Reserving/reserve` → respond
- `Requesting/noShow` → `Reserving/noShow` → **punishNoShow** → `Karma/punish`
- `Requesting/cancel|grant` → 对应 Reserving 动作

where 读取边：session→user，user→standing，(venue,time,date,party)→slot，reservation→user。

## 产品依赖与 MVP

- 本产品：Reserving → Availability（先发现时段再承诺）
- 本产品：makeReservation → Authenticating、Karma
- MVP = 四概念 + identity/publish/reserve/karma flows
- 有意义子集：去掉 Karma 后仍可预约（去掉 where 与 punishNoShow）；去掉 Availability 则无法用餐厅/时间入口（需已知 Slot 身份，本 CLI 未暴露）

## 验收

| 验收 | 出处 |
|---|---|
| reserve 成功后可在到场日 grant | Reserving OP |
| 同 slot 第二次 reserve 得 Conflict 且不改状态 | Reserving.reserve Conflict |
| create 后 query 能命中；toggle 后不再命中 | Availability OP |
| register+login 后 `_getUser` 返回该用户 | Authenticating OP |
| 两次 punish 后 `_inGoodStanding` 为空 | Karma OP |
| 无 slot / 无 session / 不良 karma 不调用成功 reserve | 应用场景 S3–S5 |
| noShow 成功才 punish | `punishNoShow` when 成功 case |
| 概念 crate 互不依赖 | 实现约定 |

## 排除与未决

见 [`MODEL.md` 排除表](../../../MODEL.md#排除与未决)。未决前提不得用实现猜测填补。
