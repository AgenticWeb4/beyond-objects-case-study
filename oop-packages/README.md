# 面向对象分包栏

教学目的：**把 User / Restaurant / Reservation 拆成多个 crate，仍然不是概念设计。**

先读仓库根目录 [`../START-HERE.md`](../START-HERE.md)。

| crate | 里面住什么 | 和概念侧的差别 |
|---|---|---|
| `user` | User（密码 + karma + booked_dates） | 不是 Authenticating；口袋没拆 |
| `restaurant` | Restaurant + `Slot.reserved` | 不是 Availability；预测和占用焊死 |
| `reservation` | Reservation，字段指着人和店 | **依赖** user、restaurant 两个包 |
| `booking` | BookingService 同时改三个对象 | 不是 Reserving，也不是 syncs |

单库 `restaurant.sqlite`，外键对象图与 `ddd-clean/` 相同。  
第 5、6 步也相同：搜索变空，Bob 得 `NoSlot`。
