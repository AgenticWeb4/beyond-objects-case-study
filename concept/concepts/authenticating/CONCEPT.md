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

## notes
论文称 UserAuthentication。密码以明文存于内存演示库，不是安全设计。
