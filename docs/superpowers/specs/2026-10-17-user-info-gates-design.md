# 用户信息与板块门槛设计

- 日期：2026-10-17
- 状态：设计已确认，待评审
- 关联原型：[2026-10-17-user-info-gates.html](../mockups/2026-10-17-user-info-gates.html)
- 关联文档：[2026-10-03-auth-levels-design.md](./2026-10-03-auth-levels-design.md)
- 后端：`backend-rs`（Rust / Axum）
- 前端：Vue 3 + Element Plus

> 本文是「用户信息项 + 板块双门槛」的唯一契约来源。实现时以本文的字段、规则语义与 API 为准。

---

## 1. 背景与目标

现状：

- 用户只有 `bio / avatar` 等基础资料，认证体系是 10 个扁平 `AuthType`。
- 板块只有一个 `required_auth_types`（CSV），仅用于发帖/投票判定，没有浏览门槛。
- 用户无法按性别、年龄、城市、职业、学历、房产、兴趣等信息设置准入条件。

目标：

1. 用户可创建板块，并为每个板块设置**浏览门槛**与**操作门槛**（发帖、评论、点赞、投票）。
2. 门槛可组合多种用户信息，并可要求某项信息的**认证等级**。
3. 用户信息填写与门槛配置都要足够简单：资料卡 + 单项抽屉、可视化条件构建器。
4. 用户信息**完全私密**：具体值与认证徽章都不对其他用户展示，只用于服务端匹配。
5. 兼容现有认证数据与旧接口，平滑迁移。

### 1.1 已确认的产品决策

| # | 决策 |
|---|------|
| D1 | 信息维度：身份、所在地、职业学历、房产、兴趣共 10 项 |
| D2 | 条件组合：组内 AND；同一字段多选值之间 OR |
| D3 | 操作门槛自动继承浏览门槛，只配置附加条件 |
| D4 | 认证沿用低/中/高/极高四级，逐信息项独立；兴趣免认证 |
| D5 | 现有 10 类认证统一收敛为信息项，旧类型做映射与兼容 |
| D6 | 信息完全私密，不展示具体值，也不展示认证徽章 |
| D7 | 未达浏览门槛：列表可见，点入后拦截并提示缺失条件 |
| D8 | 交互采用「资料卡 + 条件构建器」，不做自然语言门槛输入 |

---

## 2. 信息项模型

### 2.1 字段目录

| field_key | 分组 | 名称 | 值类型 | 可认证 | 允许算子 |
|---|---|---|---|---|---|
| `REAL_NAME` | IDENTITY | 实名 | 文本 | 是 | 仅 `authLevel` |
| `GENDER` | IDENTITY | 性别 | 枚举 `MALE/FEMALE` | 是 | `in` / `not_in` |
| `AGE` | IDENTITY | 出生日期 | 日期 | 是 | `between` / `gte` / `lte` |
| `COUNTRY` | LOCATION | 国家 | 枚举（ISO 3166-1） | 是 | `in` / `not_in` |
| `CITY` | LOCATION | 城市 | 行政区划多选 | 是 | `in` / `not_in` |
| `OCCUPATION` | CAREER | 职业 | 分类 + 自由文本 | 是 | `in` / `not_in` |
| `EDUCATION` | CAREER | 学历 | 有序枚举 | 是 | `gte` / `lte` / `eq` |
| `PROPERTY_AREA` | PROPERTY | 房产区域 | 行政区划多选 | 是 | `in` / `not_in` |
| `PROPERTY_COUNT` | PROPERTY | 房产数量 | 整数 | 是 | `between` / `gte` / `lte` |
| `INTERESTS` | INTEREST | 兴趣标签 | 标签多选 | **否** | `in` / `not_in` |

学历有序枚举：

```
HIGH_SCHOOL(1) < COLLEGE(2) < BACHELOR(3) < MASTER(4) < DOCTOR(5)
```

职业分类使用固定一级分类（`INTERNET`、`FINANCE`、`EDUCATION`、`MEDICAL`、`GOVERNMENT`、`MANUFACTURING`、`SERVICE`、`OTHER`），配合自由文本职位名。门槛匹配只匹配一级分类，避免自由文本不可控。

行政区划使用国家标准行政区划代码（省/市/区），前端用级联选择器，门槛存储代码数组。

### 2.2 值存储 JSON

`user_info.value_json`：

```json
{
  "REAL_NAME":       { "fullName": "张三" },
  "GENDER":          { "gender": "MALE" },
  "AGE":             { "birthDate": "1998-04-01" },
  "COUNTRY":         { "country": "CN" },
  "CITY":            { "country": "CN", "city": "310000" },
  "OCCUPATION":      { "category": "INTERNET", "title": "产品经理" },
  "EDUCATION":       { "education": "BACHELOR" },
  "PROPERTY_AREA":   { "cities": ["310000"], "districts": ["310115"] },
  "PROPERTY_COUNT":  { "count": 2 },
  "INTERESTS":       { "tags": ["hiking", "photography", "coffee"] }
}
```

- 年龄不直接存年龄，存出生日期，判定时按服务器当前日期计算周岁。
- 房产区域允许只选城市或精确到区；门槛按「用户选中的区域集合与条件区域集合有交集」判定。
- 修改已认证字段的值会**重置该字段认证等级为 `NONE`**，需重新认证。

### 2.3 兼容用的隐藏字段

为保持旧 `required_auth_types` 的判定语义，以下旧认证类型作为**隐藏信息项**保留：

| field_key | 说明 | 页面展示 |
|---|---|---|
| `ID_CARD` | 身份证认证记录 | 不展示 |
| `PROPERTY_CERT` | 房产证认证记录 | 不展示 |
| `ASSETS` | 资产认证 | 不展示 |
| `MARRIAGE_CERT` | 婚姻认证 | 不展示 |
| `DRIVER_LICENSE` | 驾照认证 | 不展示 |
| `CUSTOM` | 自定义认证 | 不展示 |

- `info_fields.enabled = 0`，`authable = 1`，`gate_ops` 为空，只支持 `authLevel` 条件。
- 旧规则迁移后可以继续引用这些字段；新版门槛构建器不提供它们，但保存时会原样保留已有条件。
- 用户资料页不展示这些字段。

---

## 3. 认证等级与隐私

### 3.1 等级语义

沿用四级 `AuthLevel`：

| 等级 | 含义 | 证据 |
|---|---|---|
| `NONE` | 未认证 | 无 |
| `LOW` | 已填写 | 自填，规则引擎即时通过 |
| `MEDIUM` | 已核实 | 证件/证明图片 + AI 审核 |
| `HIGH` | 强核实 | 活体视频 + 证件 + AI |
| `EXTREME` | 极高 | 预留，需移动 App |

- 每个信息项独立记录等级；升级覆盖，高等级自动满足低等级要求。
- `INTERESTS` 只有 `NONE`（免认证），不允许门槛设置认证等级。
- `AGE / GENDER / COUNTRY / CITY / REAL_NAME` 可共用一次身份证认证同时升级。
- `PROPERTY_AREA / PROPERTY_COUNT` 共用一次房产证认证同时升级。

### 3.2 认证类型 → 信息项映射

| 认证类型 | 授予的信息项 | 说明 |
|---|---|---|
| `REAL_NAME` | `REAL_NAME` | 兼容旧数据 |
| `GENDER` | `GENDER` | 兼容旧数据 |
| `AGE` | `AGE` | 兼容旧数据 |
| `NATIONALITY` | `COUNTRY` | 旧键映射 |
| `ID_CARD` | `REAL_NAME`、`GENDER`、`AGE`、`COUNTRY`、`CITY` | 一次证件审核，多项联动认证；`ID_CARD` 本身也是隐藏字段 |
| `PROPERTY_CERT` | `PROPERTY_AREA`、`PROPERTY_COUNT` | 一次房产证审核，多项联动认证；`PROPERTY_CERT` 本身也是隐藏字段 |
| `EDUCATION` | `EDUCATION` | 新增 |
| `OCCUPATION` | `OCCUPATION` | 新增 |
| `CITY` | `CITY` | 新增，单独认证入口 |
| `ASSETS` / `MARRIAGE_CERT` / `DRIVER_LICENSE` / `CUSTOM` | 不映射到可见信息项 | 保留为隐藏字段，仅供旧规则兼容 |

### 3.3 隐私

- 具体信息值、认证等级、认证徽章**只对本人可见**。
- `GET /users/:id` 不再返回认证徽章；`Profile.vue` 的原「已通过认证」区域移除。
- 门槛判定在服务端完成；对外只返回「是否满足当前板块门槛」。
- 未满足时，缺失条件只返回给当前登录用户本人，不向其他用户泄露。
- 管理员在审核认证材料时可查看用户提交的证据，这是审核流程的例外，不对外展示。

---

## 4. 板块门槛

### 4.1 两个权限维度

- **浏览门槛** `browse_rule`：能否查看板块详情、帖子列表、帖子详情、评论。
- **操作门槛** `operate_rule`：能否发帖、评论、点赞、投票。

操作门槛**始终先校验浏览门槛**，再校验附加条件。界面在操作页签展示只读的继承摘要。

### 4.2 规则 JSON

```json
{
  "version": 1,
  "conditions": [
    { "field": "GENDER", "op": "in", "values": ["MALE"] },
    { "field": "AGE", "op": "between", "min": 25, "max": 40 },
    { "field": "CITY", "op": "in", "values": ["310000", "330100"] },
    { "field": "EDUCATION", "op": "gte", "value": "BACHELOR", "authLevel": "MEDIUM" }
  ]
}
```

字段：

- `field`：信息项 `field_key`。
- `op`：见 §2.1 允许算子；当条件只要求认证等级时可省略。
- `values`：多值算子使用；数组内为 OR。
- `value`：单值算子使用。
- `min` / `max`：区间算子使用。
- `authLevel`：可选；要求该字段认证等级 ≥ 指定值。`INTERESTS` 禁止携带。

空规则 `{ "version": 1, "conditions": [] }` 表示无门槛。

### 4.3 判定语义

对每个条件：

1. 用户该字段必须已填写（`value_json` 非空）。
2. 值条件满足（同一字段多值 OR）；仅含 `authLevel` 的条件跳过此步。
3. 若带 `authLevel`，该字段 `auth_status = APPROVED` 且等级 rank ≥ 要求。
4. 所有条件同时满足才算通过。

年龄按 `birthDate` 计算周岁后比较；房产区域按集合交集判定；学历按序号比较。

**豁免**：板块创建者、`OWNER` / `MODERATOR` 成员、全局 `ADMIN` 跳过浏览与操作门槛，避免创建者把自己锁在板块外。

### 4.4 旧规则兼容

- `sub_forums.required_auth_types` 保留字段。
- 读取优先级：`operate_rule` 非空 → 使用新规则；否则回退到旧 CSV 判定，语义等同「操作门槛」。
- 旧的 `TYPE`、`TYPE:LEVEL`、`AGE_n` 在迁移时转换为新条件：
  - `TYPE` → `{field:"TYPE", authLevel:"LOW"}`（仅要求该信息项已认证/已填写）。
  - `TYPE:LEVEL` → `{field:"TYPE", authLevel:"LEVEL"}`。
  - `NATIONALITY` → `COUNTRY`；`AGE_n` → `{field:"AGE", op:"gte", min:n}`。
- 首次通过新版界面保存时，写入 `operate_rule` 并保留 `required_auth_types` 不再参与判定。

---

## 5. 数据模型

### 5.1 `info_fields`（字段目录）

| 字段 | 类型 | 说明 |
|---|---|---|
| `field_key` | VARCHAR(40) PK | 如 `GENDER` |
| `group_key` | VARCHAR(20) | `IDENTITY/LOCATION/CAREER/PROPERTY/INTEREST` |
| `label` | VARCHAR(40) | 中文名 |
| `value_type` | VARCHAR(20) | `text/enum/multi_enum/date/int/ordinal/tags/region` |
| `options_json` | TEXT | 枚举选项、职业分类、学历序号 |
| `authable` | TINYINT | 是否可认证 |
| `gate_ops` | VARCHAR(120) | 允许算子 CSV |
| `sort_order` | INT | 展示顺序 |
| `enabled` | TINYINT | 是否启用 |

- 启动时幂等种子写入 10 个字段。
- 新增维度只需插入一行并在前端支持对应 `value_type`，无需改表结构；后台管理界面不在本期范围。

### 5.2 `user_info`（用户信息值 + 认证状态）

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | BIGINT PK | |
| `user_id` | BIGINT | |
| `field_key` | VARCHAR(40) | |
| `value_json` | TEXT | 值 JSON |
| `auth_level` | VARCHAR(20) | `NONE/LOW/MEDIUM/HIGH/EXTREME` |
| `auth_status` | VARCHAR(20) | `NONE/PENDING/APPROVED/REJECTED` |
| `auth_id` | BIGINT NULL | 关联 `user_authentications.id` |
| `created_at` | DATETIME | |
| `updated_at` | DATETIME | |

唯一键：`(user_id, field_key)`。

### 5.3 `sub_forums` 扩展

| 字段 | 类型 | 说明 |
|---|---|---|
| `browse_rule` | TEXT NULL | 浏览门槛 JSON |
| `operate_rule` | TEXT NULL | 操作门槛 JSON |
| `gate_version` | INT NOT NULL DEFAULT 0 | 迁移标记 |

### 5.4 迁移

新增 `backend-rs/src/migrate.rs`，启动时幂等执行：

1. `CREATE TABLE IF NOT EXISTS info_fields / user_info`。
2. 通过 `information_schema` 检查并补 `sub_forums.browse_rule / operate_rule / gate_version`。
3. 种子写入 `info_fields`：10 个可见字段 + 6 个隐藏兼容字段（§2.3）。
4. `gate_version = 0` 的板块：把 `required_auth_types` 转为 `operate_rule`，置 `gate_version = 1`。
5. 从 `user_authentications`（`APPROVED`）回填 `user_info`：
   - 按 §3.2 映射写入可见字段与隐藏字段的值、等级。
   - 已存在的新值不覆盖；只补缺失项。
6. 迁移失败不阻塞启动，但记录错误日志；门槛判定遇到缺失表时回退旧逻辑。

---

## 6. API 契约

所有响应保持 `{ code, message, data }`。门槛拒绝的 403 需要携带 `data`，因此扩展 `ApiError` 支持可选 `data`。

### 6.1 信息目录

```
GET /info/fields
data: [{
  groupKey, groupLabel,
  fields: [{
    fieldKey, label, valueType, options,
    authable, gateOps, authLevels
  }]
}]
```

### 6.2 我的资料

```
GET /info/me
data: {
  completeness: { filled: 7, total: 10, percent: 70 },
  groups: [{
    groupKey, groupLabel,
    items: [{
      fieldKey, label, value, valueText,
      authLevel, authStatus, canUpgrade,
      privacy: "PRIVATE"
    }]
  }]
}
```

### 6.3 填写 / 修改 / 免认证

```
PUT /info/me/:fieldKey
body: { "value": { ... } }
data: { fieldKey, valueText, authLevel, authStatus }
```

- 可认证字段：`LOW` 直接通过，写入 `auth_level = LOW / auth_status = APPROVED`。
- 已认证字段值变更：重置为 `NONE`，返回提示「资料已更新，需重新认证」。
- 审核中（`PENDING`）的字段不可修改值，返回 409「该信息项正在审核中」。
- `INTERESTS`：`auth_level = NONE`，始终可保存。
- 非法值/未知字段/值类型不匹配：400。

### 6.4 认证提交（扩展）

沿用 `POST /authentications`，新增/调整：

- `type` 支持 `EDUCATION`、`OCCUPATION`、`CITY`、`PROPERTY_CERT`。
- `ID_CARD` 的 `data` 必须包含：`fullName, idNumber, gender, birthDate, country, city`。
- `PROPERTY_CERT` 的 `data` 必须包含：`cities, districts, count`。
- 审核通过后，按 §3.2 写入 `user_info` 并更新等级。
- `GET /authentications/types` 同步返回新类型、字段和等级说明。

### 6.5 板块创建 / 更新

```
POST /forums
PUT  /forums/:id
body: {
  ...,
  "browseRule": { "version": 1, "conditions": [...] },
  "operateRule": { "version": 1, "conditions": [...] }
}
```

- `operateRule.conditions` 只存附加条件，不重复浏览条件。
- 规则校验失败返回 400，并指出具体条件索引与原因。
- 读取详情时返回 `browseRule`、`operateRule`、`legacyRequiredAuthTypes`。

### 6.6 门槛拦截

```
GET /forums/:id                      403
GET /forums/:id/members              403
GET /forums/:id/custom-page          403
GET /posts?forumId=...               403
GET /posts/:id                       403
GET /comments/post/:postId           403
```

搜索接口复用 `GET /posts` 的可见板块过滤，不额外返回受限帖子。

403 body：

```json
{
  "code": 403,
  "message": "你还未满足本板块的浏览门槛",
  "data": {
    "gate": "BROWSE",
    "forumId": 12,
    "forumName": "上海互联网交流",
    "loginRequired": false,
    "missing": [
      { "fieldKey": "CITY", "label": "城市", "reason": "VALUE_MISMATCH", "required": "上海 或 杭州" },
      { "fieldKey": "EDUCATION", "label": "学历", "reason": "AUTH_LEVEL_LOW", "required": "本科（认证 ≥ 中）" }
    ]
  }
}
```

- 未登录：`loginRequired: true`，前端跳登录并回跳。
- 缺失原因枚举：`NOT_FILLED` / `VALUE_MISMATCH` / `AUTH_LEVEL_LOW` / `AUTH_PENDING`。
- 首页信息流只返回用户可浏览板块的帖子。

### 6.7 操作拦截

```
POST /posts                 403（操作门槛）
POST /comments              403（操作门槛，按帖子所属板块）
POST /posts/:id/vote        403
POST /comments/:id/vote     403
```

403 结构与浏览拦截一致，`gate` 为 `OPERATE`。

### 6.8 覆盖人数预估（P2）

```
POST /forums/gate/preview
body: { "rule": {...} }
data: { matched: 3210, total: 45210 }
```

- 允许延迟与缓存；P1 可先返回 `null`，界面隐藏该模块。

---

## 7. 前端交互

### 7.1 资料与认证页 `/profile`

- 入口：左侧导航与头像菜单新增「资料与认证」。
- 左侧 `LeftRail`，主区顶部为完善度进度与隐私说明。
- 按 `IDENTITY / LOCATION / CAREER / PROPERTY / INTEREST` 分组展示信息行。
- 每行：名称、当前值、认证徽章（仅本人可见）、隐私锁、「去认证 / 升级认证 / 编辑」。
- 点击按钮打开右侧抽屉（移动端全屏）：
  - 值编辑控件按 `valueType` 渲染。
  - 等级选择：低（直接保存）、中（证件 + AI）、高（视频 + 证件）、极高（禁用，提示需 App）。
  - 中/高等级跳转现有 `/authentication/:type/:level` 向导，完成后回到资料页。
  - 兴趣为内联标签编辑，无等级选择。
- 变更已认证值前弹出确认：「修改后需重新认证该信息项」。
- 顶部提供「去完善」按钮，定位到第一个未填写项。

### 7.2 门槛构建器

复用于 `CreateForum.vue` 与 `ForumSettings.vue`，抽成 `GateRuleEditor.vue`：

- 两个页签：浏览门槛 / 操作门槛。
- 条件行：字段选择 → 算子（按字段类型自动限定）→ 值控件 → 认证等级（可选）。
- 值控件：
  - 枚举：单选/多选
  - 年龄：最小/最大年龄输入（条件存年龄区间，用户资料存出生日期）
  - 行政区划：级联多选
  - 职业：一级分类多选
  - 学历：下拉（至少/至多/等于）
  - 整数：数字输入
  - 兴趣：标签多选
- 操作页签顶部只读展示「已继承浏览门槛」摘要。
- 底部实时生成自然语言摘要，并提示组合语义（组内 AND、字段内 OR）。
- 常用模板：`18+`、`本地`、`实名`、`本科以上`，一键追加条件。
- 保存前校验：空条件、非法算子、区间上下限、`INTERESTS` 携带 `authLevel`。

### 7.3 浏览器拦截页

- 路由进入 `/f/:slug` 或 `/post/:id` 收到 403 时，展示独立提示页：
  - 板块名称
  - 缺失条件清单（中文）
  - 「去完善资料」按钮
  - 未登录时显示「登录后查看」
- 不渲染任何帖子内容、评论或模糊预览。

### 7.4 公开资料页

- `Profile.vue` 移除「已通过认证」徽章区域。
- 其他用户看不到性别、年龄、城市、职业、学历、房产、兴趣或认证状态。

### 7.5 前端改造点

| 文件 | 改动 |
|---|---|
| `src/router/index.ts` | 新增 `/profile`（需登录） |
| `src/api/index.ts` | 新增 `infoApi`；论坛 API 增加规则字段；错误对象保留 `data` |
| `src/types/index.ts` | 新增 `InfoField`、`UserInfoItem`、`GateRule`、`GateCondition` |
| `src/views/UserInfo.vue`（新增） | 资料与认证页、分组资料卡、认证抽屉 |
| `src/components/GateRuleEditor.vue`（新增） | 浏览/操作门槛条件构建器与摘要 |
| `src/views/CreateForum.vue` | 接入 `GateRuleEditor`，移除扁平认证选择与年龄输入 |
| `src/views/ForumSettings.vue` | 同上，并回显已有规则 |
| `src/views/ForumPage.vue` | 处理 403 门槛拦截，渲染提示页 |
| `src/views/PostDetail.vue` | 处理 403 门槛拦截 |
| `src/views/Profile.vue` | 移除公开认证徽章 |
| `src/views/AuthenticationCenter.vue` | 展示新增认证类型，入口跳转携带 `fieldKey` |
| `src/views/AuthLevelWizard.vue` | 支持新类型与 `fieldKey`，提交后回跳资料页 |

---

## 8. 后端改造点

| 文件 | 改动 |
|---|---|
| `src/enums.rs` | 新增信息项枚举、学历序号、职业分类、值类型 |
| `src/auth_gate.rs` | 重写为 `gate` 语义：规则解析、条件匹配、缺失原因；保留旧 CSV 兼容函数 |
| `src/routes/info.rs`（新增） | `GET /info/fields`、`GET /info/me`、`PUT /info/me/:fieldKey` |
| `src/routes/forums.rs` | 创建/更新接收规则；详情返回规则；浏览拦截 |
| `src/routes/posts.rs` | 列表/详情浏览拦截；发帖操作拦截；投票操作拦截 |
| `src/routes/comments.rs` | 评论树浏览拦截；发评论、评论投票操作拦截 |
| `src/routes/authentications.rs` | 新类型、字段映射、审核通过后联动写 `user_info` |
| `src/routes/users.rs` | 移除公开认证徽章输出 |
| `src/models.rs` | `SubForum` 增加规则字段；新增 `UserInfo`、`InfoField` |
| `src/migrate.rs`（新增） | 建表、种子、回填、旧规则转换 |
| `src/error.rs` | 支持带 `data` 的业务错误 |
| `src/routes/mod.rs` | 注册 `/info/*` |

判定性能：

- 单次请求最多加载一次 `user_info`。
- 首页信息流先加载全部板块规则（板块数量小），在内存中计算可见板块集合，再用 SQL `forum_id IN (...)` 过滤。
- 规则 JSON 解析结果可按 `forum_id + updated_at` 做进程内缓存；P1 可先不做缓存。

---

## 9. 验收标准

| # | 验收项 |
|---|---|
| R1 | 用户可查看并按 10 个信息项填写资料，兴趣无需认证 |
| R2 | 每个可认证信息项可独立选择并达到低/中/高等级，极高显示需 App |
| R3 | 未填写/未认证/等级不足的字段在门槛判定中正确失败，并返回明确原因 |
| R4 | 浏览门槛可组合多字段；同字段多值 OR；不同字段 AND |
| R5 | 操作门槛自动继承浏览门槛，并支持附加条件 |
| R6 | 未达浏览门槛时详情/列表/帖子/评论均返回 403，且不含内容 |
| R7 | 未达操作门槛时发帖/评论/点赞/投票均返回 403 |
| R8 | 首页信息流不出现用户无权浏览的板块帖子 |
| R9 | 任何其他用户接口都不返回具体信息或认证徽章 |
| R10 | 修改已认证字段值会重置该字段认证等级 |
| R11 | 旧 `required_auth_types` 板块在迁移后行为不变 |
| R12 | 创建/管理板块时可用同一构建器配置两类门槛，保存并回显正确 |
| R13 | 前端 `npm run build` 通过；Rust `cargo check --release` 通过 |
| R14 | 迁移可重复执行，不产生重复数据 |

---

## 10. 测试

### 10.1 单元测试

- 每种算子的匹配：`in/not_in/eq/gte/lte/between`。
- 多选字段 OR、跨字段 AND。
- 年龄按出生日期计算（含生日当天）。
- 学历序号比较。
- `authLevel` 组合判定与 `INTERESTS` 拒绝。
- 缺失原因分类：未填写、值不匹配、等级不足、审核中。

### 10.2 集成测试

- 浏览门槛：论坛详情、帖子列表、帖子详情、评论树。
- 操作门槛：发帖、评论、点赞、投票。
- 首页信息流过滤。
- 旧规则回退。
- 隐私：`GET /users/:id` 不含敏感字段。

### 10.3 数据迁移测试

- 用旧 `required_auth_types` 样本验证转换后的操作门槛行为一致。
- 用旧 `user_authentications` 样本验证 `user_info` 回填。
- 连续执行迁移两次，数据不重复。

### 10.4 前端验证

- `npm run build`。
- 浏览器流程：填写资料 → 升级认证 → 创建板块设置双门槛 → 用另一账号验证拦截与提示。

---

## 11. 分期

### P1：模型与门槛闭环

- 表结构、字段目录、迁移。
- 资料与认证页（低等级填写 + 认证入口）。
- 门槛构建器、双门槛判定、浏览/操作拦截、拦截页。
- 旧规则兼容、隐私收敛。

### P2：认证与运营增强

- 各字段中/高等级材料与 AI 审核接入。
- 覆盖人数预估 `POST /forums/gate/preview`。
- 字段目录后台管理界面。
- 门槛模板的可配置化。

## 12. 不在本期范围

- 极高等级的移动 App 本体。
- 自然语言门槛输入。
- 跨板块统一门槛模板库。
- 用户信息的对外展示与选择性公开（本期固定完全私密）。
