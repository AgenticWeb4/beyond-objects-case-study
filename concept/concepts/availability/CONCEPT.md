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

## notes
本概念不读取预约。query 含 venue：相对论文 Fig.2 的签名补丁，依据正文按餐厅匹配。
