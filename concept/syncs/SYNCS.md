# app RestaurantReservations
include Availability [VenueId]
include Reserving [Authenticating.U, Availability.Slot]
include Authenticating
include Karma [Authenticating.U]
include Requesting

Requesting 为 CLI 入口。RequestId 标识一次请求；每个 CLI 命令一个 flow。
VenueId 为不透明场所身份，由 publish 传入。
Conflict 对应 Reserving 的同名错误。
Unauthorized / BadStanding / NoSlot 是 where 零行的入口映射，不是概念动作。
多行 `_getAvailableSlot` 按 Slot 身份字典序取一行（论文 “any available slot”）。

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

// flow: reserve
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

// flow: karma
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
