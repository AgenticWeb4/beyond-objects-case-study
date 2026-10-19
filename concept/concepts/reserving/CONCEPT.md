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
canceled / granted 不占用上述唯一性。Time / Date / Number 为值，不是类型参数。

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

## notes
应用实例化与 sync 见 SYNCS，不在本卡片。canceled 释放 slot 与同日唯一性为本模型确认（论文未写）。同日一人一约是对论文 “close to time” 的确认简化。
