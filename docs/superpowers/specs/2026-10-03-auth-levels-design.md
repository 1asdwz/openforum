# OpenForum 四级认证体系设计文档

- 日期：2026-10-03
- 状态：已确认，进入实现
- 关联文档：[2026-10-03-openforum-design.md](./2026-10-03-openforum-design.md)（本文是其「用户信息认证」章节的扩展契约）
- 访问入口：`https://apprest.mytruststores.com/forum`

> 本文是该功能实现的唯一契约来源，覆盖原设计中「认证」相关的模型、API 与流程。

---

## 1. 背景与目标

现状：`UserAuthentication` 是「扁平类型」模型——10 个 `AuthType` 各自独立提交一段文本，由规则引擎即时校验，全部为同一强度，没有「证据强度」概念，也没有图片/视频证据。

目标：引入**四个认证等级**，并把等级与「认证类型」正交组合：

| 等级 | 名称 | 证据与流程 |
|------|------|-----------|
| `LOW` | 低 | 仅填写信息，规则引擎即时审核（保持现状） |
| `MEDIUM` | 中 | 信息 + 相关**证件图片**上传 → **AI 审核** |
| `HIGH` | 高 | 信息 → **视频认证**（浏览器端活体检测 + 人脸分析 + 随机动作序列）→ 通过后再上传**证件图片** → AI 审核 → **通过后自动登记实名认证** |
| `EXTREME` | 极高 | 在 `HIGH` 基础上，使用**独立移动 App 防伪录制视频** |

**本期范围**：`LOW` / `MEDIUM` / `HIGH` 全部实现；`EXTREME` 仅**预留数据结构与接口**（不实现 App 本体），界面提示需通过移动 App 完成。

### 1.1 已确认的关键决策
- 活体/人脸判定：**浏览器端检测**（MediaPipe），服务器不做视频 ML 推理。
- 文件存储：**复用 restaurant-seller 项目的同一套 MinIO**（`43.157.5.182:9090`，`minioadmin` / `mallPass@151911`）。
- 证件 AI 审核：**复用 qwen 密钥**（OpenAI 兼容端点，已验证支持图片输入）。
- 等级粒度：**每个认证类型独立分级**。
- `EXTREME` 的 App：本期仅预留接口。

---

## 2. 术语与等级定义

`AuthLevel`（枚举，带序号）：

```
LOW(1, "低")  MEDIUM(2, "中")  HIGH(3, "高")  EXTREME(4, "极高")
```

- 等级与 `AuthType` **正交**：任意类型都可按任意等级提交。
- 用户的某个类型只保留**一条主记录**（沿用唯一约束 `(user_id, type)`），记录带 `level` 字段，表示**已达成的最强等级**。
- **升级即覆盖**：重新提交更高等级会更新该记录；更高等级通过后自动视为同时满足更低等级。
- 历史留痕：由 `AuthReviewLog`（见 §4）记录每次提交/决策，主记录只保留当前状态。

---

## 3. 需求拆解（逐条验收）

| # | 需求 | 验收证据 |
|---|------|---------|
| R1 | 四个等级：低/中/高/极高 | `AuthLevel` 枚举 + `/authentications/types` 返回每类型 `levels[]` |
| R2 | 低：填写信息即可 | `LOW` 提交仅需文本字段，走规则引擎即时审核 |
| R3 | 中：证件图片上传 + AI 审核 | `POST /authentications/media` 上传到 MinIO；AI 视觉模型审核图片 |
| R4 | 高：视频先活体+人脸+随机动作 | 前端 MediaPipe 实时判定；后端校验挑战序列与视频 |
| R5 | 高：验证通过后再看证件，再 AI 审核 | 流水线顺序：挑战校验 → 视频 → 证件 → AI |
| R6 | 高：审核通过后自动登记实名认证 | 通过后 upsert `REAL_NAME` 记录（见 §6.3） |
| R7 | 极高：高 + App 防伪录制 | 预留 `attestation` + 接口返回「需 App」提示 |
| R8 | 界面显示说明事项、拍摄注意事项 | 向导页展示 `levels[].requirements/tips` |
| R9 | 子论坛按等级设门槛 | `requiredAuthTypes` 支持 `TYPE:LEVEL` |
| R10 | 不破坏现有能力 | 旧语法 `TYPE`、`AGE_n` 与既有接口保持兼容 |

---

## 4. 数据模型

### 4.1 `UserAuthentication`（扩展）
新增字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `level` | `VARCHAR(20)` | `AuthLevel`，默认 `LOW` |
| `media` | `TEXT` | JSON：媒体 id 列表 `[12,13]` |
| `liveness` | `TEXT` | JSON：活体/动作结果（见 §7.3） |
| `attestation` | `TEXT` | JSON：App 防伪信息（`EXTREME` 用，预留） |

沿用 `status / review_method / confidence / reason / rejection_reason / reviewed_by / reviewed_at`。

### 4.2 `AuthMedia`（新增）
| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | BIGINT PK | |
| `user_id` | BIGINT | 归属用户 |
| `authentication_id` | BIGINT NULL | 提交时绑定 |
| `kind` | VARCHAR(20) | `DOCUMENT` / `SELFIE` / `VIDEO` |
| `object_key` | VARCHAR(255) | MinIO 对象键 |
| `content_type` | VARCHAR(100) | |
| `size_bytes` | BIGINT | |
| `sha256` | VARCHAR(64) | 去重/防篡改 |
| `created_at` | DATETIME | |

### 4.3 `AuthChallenge`（新增）
| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | VARCHAR(36) PK | UUID，下发给前端 |
| `user_id` | BIGINT | 绑定用户 |
| `actions` | VARCHAR(255) | JSON：随机动作序列，如 `["BLINK","TURN_LEFT","OPEN_MOUTH"]` |
| `nonce` | VARCHAR(64) | 随机串，随视频水印显示 |
| `expires_at` | DATETIME | 10 分钟有效 |
| `used_at` | DATETIME NULL | 一次性消费 |
| `created_at` | DATETIME | |

### 4.4 `AuthReviewLog`（新增，审计）
`id, authentication_id, level, action(SUBMIT/APPROVE/REJECT/RETRY), method, confidence, reason, actor_id, created_at`。每次提交与决策写入，保证升级/驳回可追溯。

---

## 5. API 契约

所有响应仍为既有封装 `{ code, message, data }`。

### 5.1 类型元数据（扩展）
```
GET /authentications/types
data: [{
  type, label, description,
  fields: [{name,label,type,required,options?}],
  levels: [{
    level: "LOW|MEDIUM|HIGH|EXTREME",
    label: "低|中|高|极高",
    requirements: ["需要填写真实姓名","需要上传证件正面照片"],
    tips: ["光线充足","面部无遮挡、不戴帽子墨镜","正对镜头","按提示完成动作"]
  }]
}]
```
`requirements/tips` 由后端按「类型 × 等级」生成（界面说明与拍摄注意事项的唯一来源）。

### 5.2 挑战下发
```
GET /authentications/challenge
data: { challengeId, actions: ["BLINK","TURN_LEFT","OPEN_MOUTH"], nonce, expiresAt }
```
动作池：`BLINK`(眨眼) / `OPEN_MOUTH`(张嘴) / `TURN_LEFT`(左转头) / `TURN_RIGHT`(右转头) / `NOD`(点头)。每次随机 3 个。

### 5.3 媒体上传
```
POST /authentications/media     multipart/form-data: file, kind
data: { mediaId, kind, contentType, sizeBytes }
```
约束：`DOCUMENT/SELFIE` 仅 `image/jpeg|png|webp`，单张 ≤ 10MB，每次认证 ≤ 5 张；`VIDEO` 仅 `video/webm|mp4`，≤ 50MB。服务端校验 MIME 与大小，大小写扩展名白名单。

### 5.4 媒体读取（鉴权）
```
GET /authentications/media/{mediaId}
```
仅**本人**或**管理员**可读，服务端从 MinIO 私有桶流式返回，不做公开直链。

### 5.5 提交认证
```
POST /authentications
body: {
  type, level,
  data: { ...字段 },
  mediaIds: [12,13],          // MEDIUM+ 必填至少 1 个 DOCUMENT
  videoMediaId: 14,           // HIGH+ 必填
  challengeId: "uuid",        // HIGH+ 必填
  liveness: { ... },          // HIGH+ 必填
  attestation: { ... }        // EXTREME 必填（预留）
}
```
返回 `AuthenticationDTO`（含 `level/status/reviewMethod/confidence/reason`）。

### 5.6 我的记录（扩展）
```
GET /authentications/me
data: [{ id,type,level,status,reviewMethod,confidence,reason,rejectionReason,
         media:[{id,kind,contentType}], liveness?, attestation?, createdAt,reviewedAt }]
GET /authentications/me/approved   // 保持不变：返回已通过的类型名 string[]（如 ["REAL_NAME","AGE"]）
                                   // 等级信息仅在 /authentications/me 中暴露，避免破坏既有前端
```

### 5.7 重试（扩展）
```
POST /authentications/{id}/retry
body: 同 5.5 的可选字段（data/level/mediaIds/videoMediaId/challengeId/liveness/attestation）
```

### 5.8 管理端（扩展）
```
GET  /admin/authentications?status=&level=&page=&size=   // 新增 level 过滤
POST /admin/authentications/{id}/approve   body:{reason}
POST /admin/authentications/{id}/reject    body:{reason}
```
管理端记录包含 `media[]` 与 `liveness`，可通过 5.4 查看媒体。

---

## 6. 审核流水线

`AuthenticationReviewService` 按等级执行**结构化流水线**，插件负责「专业判断」，服务负责「顺序与决策」。

### 6.1 结构校验（按等级，硬性）
| 等级 | 硬性要求 |
|------|---------|
| `LOW` | 该类型必填文本字段齐全（复用 `RuleBasedReviewPlugin`） |
| `MEDIUM` | `LOW` + 至少 1 个 `DOCUMENT` 媒体 |
| `HIGH` | `MEDIUM` + 合法且未过期的 `challenge`（动作序列全部完成）+ 1 个 `VIDEO` 媒体 + `liveness.passed=true` |
| `EXTREME` | `HIGH` + `attestation`（App 签名/设备指纹/nonce，预留） |

结构校验失败 → 直接 `REJECTED`（`reviewMethod=RULE`）。

### 6.2 专业审核（插件）
结构校验通过后，按顺序调用已启用的 `DocumentReviewPlugin`：
- **`AiVisionReviewPlugin`**（配置了 AI 时启用）：把证件图片 + 声明的字段 + 视频关键帧交给 OpenAI 兼容视觉模型，返回 `{status, confidence, reason, extracted:{fullName,...}}`。
- **`HeuristicDocumentPlugin`**（默认启用）：检查媒体存在性、数量、图像可解码、清晰度阈值；只能判 `REJECTED`（明显不合格）或 `PENDING`，**不自动通过**。

决策规则：
- 存在任一高置信 `REJECTED` → `REJECTED`。
- 最高置信 `APPROVED` 且 `confidence ≥ 0.85` → `APPROVED`（`reviewMethod=AI`）。
- 其余 → `PENDING`（进入人工审核队列，`reviewMethod=MANUAL`）。

> 未配置 AI 时，`MEDIUM/HIGH` 一律 `PENDING`，由管理员人工审核；流程与状态机完全一致。

### 6.3 自动登记实名认证
当产生 `APPROVED` 且 `level ≥ HIGH` 时：
1. 从提交数据或 AI `extracted` 中取得 `fullName`（与可选 `idNumber`）。
2. 若存在 `fullName`：upsert 该用户的 `REAL_NAME` 记录为 `APPROVED`，`level = HIGH`，`reviewMethod = SYSTEM`，`reason = "由 <type>:<level> 认证自动登记"`。
3. 若用户已有 `REAL_NAME` 且等级 ≥ 目标，则不降级。

触发类型：`REAL_NAME / ID_CARD / DRIVER_LICENSE / MARRIAGE_CERT / PROPERTY_CERT / NATIONALITY`（不含纯数值类 `AGE/ASSETS`）。

---

## 7. 浏览器端活体 / 人脸 / 动作（HIGH）

### 7.1 技术选型
- 库：`@mediapipe/tasks-vision` 的 `FaceLandmarker`（blendshapes + 变换矩阵），纯浏览器运行。
- 资源：WASM 从 `node_modules/@mediapipe/tasks-vision/wasm` 复制到 `dist/mediapipe`（`vite-plugin-static-copy`）；模型 `face_landmarker.task` 放入 `public/models/`，构建时随静态资源发布。
- 运行时路径（`base=/forum/`）：WASM `/forum/mediapipe`，模型 `/forum/models/face_landmarker.task`。

### 7.2 动作判定（客户端，实时）
| 动作 | 判据 |
|------|------|
| `BLINK` | `eyeBlinkLeft>0.5 && eyeBlinkRight>0.5` 出现并回落到 `<0.3` 记一次 |
| `OPEN_MOUTH` | `jawOpen>0.5` |
| `TURN_LEFT/RIGHT` | 由面部变换矩阵估计偏航角，超过阈值并回正 |
| `NOD` | 俯仰角变化超过阈值并回正 |

要求：**同屏仅一个人脸**；未检出人脸或有第二张脸时暂停计时并在界面提示。全部动作按顺序完成方可进入下一步。

### 7.3 录制与提交
- 用 `MediaRecorder` 在整个动作流程中录制，产出 `webm`，叠加 `nonce` 水印（Canvas 绘制帧）。
- `liveness` 载荷（客户端上报，作为**建议性证据**）：
```json
{
  "engine": "mediapipe-face-landmarker",
  "challengeId": "uuid",
  "faceCount": 1,
  "actions": [{"action":"BLINK","passed":true,"atMs":1200}],
  "passed": true,
  "startedAt": "...", "endedAt": "..."
}
```
- 服务端**硬门槛**为：挑战合法且未使用、动作全部完成、视频存在；视频内容由 AI/人工复核，客户端载荷不作为唯一依据。

### 7.4 降级
摄像头无权限、模型加载失败或设备能力不足时：界面提示并降级为「录制视频 + 人工审核」（仍要求 `HIGH`，状态为 `PENDING`），不阻断流程。

---

## 8. 前端设计

### 8.1 页面
- 认证中心（改造 `AuthenticationCenter.vue`）：按类型展示卡片，卡片内以等级标签展示**已达成的最高等级**与状态，按钮进入向导。
- 认证向导（新增路由 `/auth/:type/:level`，组件 `AuthLevelWizard.vue`）步骤：
  1. **说明与注意事项**：显示该等级 `requirements` 与 `tips`（拍摄注意事项）。
  2. **信息填写**（复用 `AuthTypeForm.vue`）。
  3. **证件上传**（`MEDIUM+`）：`DocumentUpload.vue`，支持拍照/选择，显示尺寸与合规提示。
  4. **视频认证**（`HIGH+`）：`VideoCapture.vue` —— 摄像头预览 → 动作引导 → 录制 → 上传。
  5. **App 防伪**（`EXTREME`）：占位说明「需通过 OpenForum 移动 App 完成」，不提供网页提交。
  6. **提交**与结果回显。

### 8.2 组件与服务
- `components/DocumentUpload.vue`、`components/VideoCapture.vue`、`components/AuthLevelWizard.vue`
- `utils/liveness.ts`：MediaPipe 封装（加载模型、逐帧判定、动作状态机、录制）。
- `types/index.ts` 扩展：`AuthLevel`、`AuthTypeMeta.levels`、`AuthRecord.level/media/liveness`。

### 8.3 文案
所有「说明相关事项、拍摄注意事项」来自后端 `levels[].requirements/tips`，避免前后端文案分叉。

---

## 9. 子论坛门槛语法

`requiredAuthTypes`（逗号分隔，忽略大小写），单项支持三种形式：

| 形式 | 含义 |
|------|------|
| `TYPE` | 该类型已通过，等级 ≥ `LOW`（兼容旧数据） |
| `TYPE:LEVEL` | 该类型已通过，且等级 ≥ `LEVEL`，如 `REAL_NAME:HIGH` |
| `AGE_n` | 兼容旧语法：`AGE` 已通过且年龄 ≥ n |

示例：`REAL_NAME:HIGH,AGE_18,DRIVER_LICENSE:MEDIUM`

`AuthGateService.evaluate` 改为：
1. 取该用户所有 `APPROVED` 记录，键为 `type`，值为 `level`。
2. 对每项要求：解析出 `type` 与所需 `level`（默认 `LOW`），命中且 `recordLevel ≥ requiredLevel` 视为满足。
3. `AGE_n` 分支保持原逻辑（读 `AGE` 记录的 `age` 字段）。
4. 未满足项汇总进 `missing`，用于前端提示。

---

## 10. 插件扩展

新增接口（均继承 `ForumPlugin`，由 `PluginRegistry` 自动收集）：

```java
public interface DocumentReviewPlugin extends ForumPlugin {
    ReviewResult reviewDocuments(DocumentReviewContext ctx); // 证件图片 + 声明字段
    ReviewResult reviewVideo(VideoReviewContext ctx);        // 视频（可选）
}
```
- `PluginType` 新增 `DOCUMENT_REVIEW`（管理端 `/plugins` 可见与开关）。
- 实现：`AiVisionReviewPlugin`（`isAvailable()` 依赖 AI 配置）、`HeuristicDocumentPlugin`（默认启用）。
- `ReviewResult` 扩展 `extracted` 字段（Map），承载 AI 抽取的 `fullName/idNumber` 供 §6.3 使用。

---

## 11. 存储与部署（MinIO）

复用 restaurant-seller 的同一 MinIO 实例（服务器 `43.157.5.182`）：

| 项 | 值 |
|----|----|
| 容器 | `minio`（宿主 `9090` → 容器 `9000`，控制台 `9001`） |
| 凭证 | `minioadmin` / `mallPass@151911` |
| 后端访问 | `MINIO_ENDPOINT=http://minio:9000`（把 `minio` 接入 `openforum-net`） |
| 桶 | `openforum-auth`（**私有**，不设公开读策略）；对象键 `auth/{userId}/{yyyyMMdd}/{uuid}.{ext}` |
| 公共直链 | `https://apprest.mytruststores.com/file`（nginx `/file/` → MinIO，仅用于非敏感资源；认证媒体**不**走直链） |

后端新增环境变量：
```
MINIO_ENABLED=true
MINIO_ENDPOINT=http://minio:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=mallPass@151911
MINIO_BUCKET=openforum-auth
```
Maven 依赖：`io.minio:minio:8.5.x`。

### 11.1 上传大小上限（必须）
- nginx `location /forum/api/` 增加 `client_max_body_size 60m;`（否则上传 413）。
- Spring：`spring.servlet.multipart.max-file-size=50MB`、`max-request-size=60MB`。

### 11.2 AI 配置（qwen）
```
AI_ENABLED=true
OPENAI_API_KEY=<qwen key>
OPENAI_API_BASE=https://ws-tblxqj4wtf35a4k5.eu-central-1.maas.aliyuncs.com/compatible-mode/v1
OPENAI_API_MODEL=qwen3.7-plus
```
（`OpenForumAiProperties` 已支持上述变量；`AiReviewPlugin` 与新增的 `AiVisionReviewPlugin` 共用。）

---

## 12. 安全与隐私

- 认证媒体存**私有桶**，仅经 5.4 鉴权接口读取；桶不设公开策略。
- 上传校验：MIME + 大小 + 扩展名白名单；非法即拒。
- `challenge` 一次性、绑定用户、10 分钟过期；已用或过期不可复用。
- 客户端 `liveness` 仅供参考，最终以视频复核为准，防止伪造载荷。
- 认证数据（含证件信息）当前以 JSON 明文入库；加密存储列为后续增强项，本期不做。

---

## 13. 错误处理

| 场景 | 行为 |
|------|------|
| 等级校验不通过（如 `HIGH` 缺视频） | `400`，`message` 指明缺失项 |
| 挑战过期/已用/非本人 | `400`，提示重新开始视频认证 |
| 媒体类型/大小非法 | `400`，提示允许的格式与上限 |
| AI 不可用 | 记录降级为 `PENDING`，不报错中断 |
| MinIO 上传失败 | `500`，提示稍后重试 |

---

## 14. 测试与验收

### 14.1 后端单测
- `AuthLevel` 解析与比较；门槛 `TYPE:LEVEL` 判定（含 `AGE_n` 兼容）。
- 各等级结构校验（缺证件/缺视频/缺挑战分别驳回）。
- 自动实名登记：`ID_CARD:HIGH` 通过后生成 `REAL_NAME`（等级 `HIGH`）。

### 14.2 线上端到端（`https://apprest.mytruststores.com/forum`）
1. 注册/登录取 token。
2. `LOW` 提交 → 即时 `APPROVED`。
3. `MEDIUM`：上传证件图 → AI 审核 → 期望 `APPROVED`（或 `PENDING`）。
4. `HIGH`：`GET /challenge` → 上传视频 → 携带 `liveness` 提交 → 审核通过 → 校验 `REAL_NAME` 自动登记。
5. `EXTREME`：返回「需 App」提示。
6. 门槛：新建 `requiredAuthTypes=REAL_NAME:HIGH` 的子论坛，低等级用户发帖被 `403`，高等级用户通过。

### 14.3 浏览器手测
- Chrome 摄像头授权下完成随机动作序列并成功录制/上传；拒绝授权时走降级路径。

---

## 15. 范围外（本期不做）

- 独立移动 App 的实现（仅预留 `attestation` 与接口）。
- 服务端视频 ML 推理 / 深度防伪（活体判定在浏览器端 + AI/人工复核）。
- 认证数据的加密存储与密钥托管。
