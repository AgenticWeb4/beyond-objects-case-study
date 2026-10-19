# concept Karma [U]

## purpose
根据用户过往行为决定其是否仍有资格使用需守信的功能。

## state
strikes: U -> Number
初态：无记录的 U 视为 0。Number ≥ 0。
good standing 当且仅当 strikes(u) < 2。阈值 2 为演示确认。

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

## notes
reward / 恢复未建模。不引用 Reserving。
