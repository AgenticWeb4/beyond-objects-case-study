# 模块化 DDD 栏

教学目的：**概念设计 ≠ 把 DDD 拆成独立模块。**  
先读仓库根目录 [`../START-HERE.md`](../START-HERE.md)。

四个限界上下文（crate 名刻意不和概念侧同名）：

| crate | 限界上下文 | 里面住什么 |
|---|---|---|
| `identity` | 身份 | User（用户名+密码）、Session |
| `availability` | 可订预测 | Slot（`listed`，没有占用） |
| `booking` | 订位 | Guest、SlotOccupancy、Reservation、`BookingService` |
| `reputation` | 信誉 | Standing（strikes） |

跨上下文只传 `shared-kernel` 里的 ID。组合在 `app/src/orchestration.rs`：用例顺序调用各 BC 应用服务，不是 `when/where/then`。

打开这两个文件看「像概念、仍是 DDD」：

1. `app/src/orchestration.rs` — 像 syncs，但不是
2. `booking/src/booking_service.rs` — 像 Reserving，但不是
