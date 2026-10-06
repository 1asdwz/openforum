# 四级认证体系 实现计划

- 日期：2026-10-03
- 契约来源：[2026-10-03-auth-levels-design.md](../specs/2026-10-03-auth-levels-design.md)
- 目标：把「扁平认证」升级为「类型 × 四级等级」模型，落地 MinIO 媒体存储、证件 AI 审核、浏览器端活体检测、自动实名登记与门槛 `TYPE:LEVEL`。

> 每个阶段结束后执行该阶段的「验证」，通过后再进入下一阶段。

---

## Phase 0 基础设施与配置

### 0.1 后端依赖
- [ ] `backend/pom.xml` 新增：`io.minio:minio:8.5.7`（`<minio.version>` 属性）。
- [ ] 确认 `spring-boot-starter-web` 已含 `MultipartFile`（已具备）。

### 0.2 配置文件
- [ ] `application.yml` 与 `application-prod.yml` 增加：
```yaml
spring:
  servlet:
    multipart:
      max-file-size: 50MB
      max-request-size: 60MB
openforum:
  minio:
    enabled: ${MINIO_ENABLED:false}
    endpoint: ${MINIO_ENDPOINT:http://localhost:9000}
    access-key: ${MINIO_ACCESS_KEY:minioadmin}
    secret-key: ${MINIO_SECRET_KEY:minioadmin}
    bucket: ${MINIO_BUCKET:openforum-auth}
  auth:
    challenge-ttl-minutes: 10
    max-document-count: 5
    max-image-bytes: 10485760     # 10MB
    max-video-bytes: 52428800     # 50MB
    auto-approve-threshold: 0.85
```

### 0.3 nginx（线上）
- [ ] 服务器 `/mydata/nginx/conf.d/default.conf` 的 `location /forum/api/` 增加 `client_max_body_size 60m;`。

### 0.4 部署编排
- [ ] 线上把 `openforum-backend` 接入 `minio` 所在网络（或与 minio 同网络），注入 `MINIO_ENABLED/ENDPOINT/ACCESS_KEY/SECRET_KEY/BUCKET`，并设 `AI_ENABLED=true` 及 qwen 变量。
- [ ] 私有桶 `openforum-auth` 由后端启动时 `makeBucket`（不设公开策略）。

**验证**：`mvn -q compile` 通过；后端启动日志无配置绑定错误。

---

## Phase 1 后端数据模型

### 1.1 枚举
- [ ] 新增 `entity/enums/AuthLevel.java`：
```java
public enum AuthLevel {
    LOW(1,"低"), MEDIUM(2,"中"), HIGH(3,"高"), EXTREME(4,"极高");
    private final int rank; private final String label;
    // rank(), label(), static parse(String), atLeast(AuthLevel other)
}
```

### 1.2 扩展 `UserAuthentication`
- [ ] 新增字段（保留 `(user_id,type)` 唯一约束）：
  - `level` `@Enumerated(STRING)` `VARCHAR(20)` 默认 `LOW`
  - `media` `TEXT`（JSON id 数组）
  - `liveness` `TEXT`
  - `attestation` `TEXT`
  - 对应 getter/setter；getter 默认值 `level==null → LOW`。

### 1.3 新增实体 + Repository
- [ ] `entity/AuthMedia.java`：`id,userId,authenticationId,kind,objectKey,contentType,sizeBytes,sha256,createdAt`；`kind` 用 `entity/enums/MediaKind.java`（`DOCUMENT/SELFIE/VIDEO`）。
- [ ] `entity/AuthChallenge.java`：`id(UUID 字符串),userId,actions,nonce,expiresAt,usedAt,createdAt`。
- [ ] `entity/AuthReviewLog.java`：`id,authenticationId,level,action,method,confidence,reason,actorId,createdAt`；`action` 用 `entity/enums/ReviewAction.java`（`SUBMIT/APPROVE/REJECT/RETRY`）。
- [ ] `repository/AuthMediaRepository`：`findByUserIdAndIdIn`、`findByIdAndUserId`。
- [ ] `repository/AuthChallengeRepository`：`findByIdAndUserId`。
- [ ] `repository/AuthReviewLogRepository`：`findByAuthenticationIdOrderByCreatedAtAsc`。

**验证**：启动后 MySQL 自动建表（`ddl-auto=update`），4 张表/新列存在。

---

## Phase 2 存储层（MinIO）

- [ ] 新增 `config/OpenForumMinioProperties.java`（`openforum.minio`，record，含 `isConfigured()`）。
- [ ] 在 `OpenForumApplication` 或 config 注册 `@EnableConfigurationProperties`（跟随现有 ai 属性的注册方式）。
- [ ] 新增 `service/storage/StorageService` 接口：
```java
String put(String objectKey, InputStream in, long size, String contentType);
InputStream get(String objectKey);           // 调用方负责关闭
boolean remove(String objectKey);
```
- [ ] 新增 `service/storage/MinioStorageService.java`（`@Service`，参考 restaurant-seller 的 `MinioClient` 用法）：
  - 惰性初始化 `MinioClient`；首次使用确保桶存在（`bucketExists` → 否 `makeBucket`）。
  - `put` 用 `PutObjectArgs`；`get` 用 `GetObjectArgs` 返回流；`remove` 用 `RemoveObjectArgs`。
  - 未配置时抛 `ApiException.badRequest("文件存储未启用")`。
- [ ] 新增 `service/storage/NoopStorageService`（`@ConditionalOnMissingBean` 或 `enabled=false` 时启用），保证本地开发不配 MinIO 也能启动。

**验证**：单测/手工 `put` 一张测试图 → MinIO 控制台可见对象；`get` 能读回。

---

## Phase 3 插件层（证件审核扩展）

- [ ] `plugin/ReviewResult.java` 增加 `Map<String,Object> extracted`（默认空），保留原三参构造重载。
- [ ] `plugin/PluginType.java` 增加 `DOCUMENT_REVIEW`。
- [ ] 新增 `plugin/DocumentReviewPlugin.java`：
```java
public interface DocumentReviewPlugin extends ForumPlugin {
    boolean isAvailable();
    ReviewMethod getReviewMethod();
    ReviewResult reviewDocuments(DocumentReviewContext ctx);
}
```
- [ ] 新增 `plugin/DocumentReviewContext.java`：`AuthType type, AuthLevel level, Map<String,Object> data, List<MediaRef> documents, MediaRef video, byte[] videoPoster`（`MediaRef` = `id,kind,contentType,byte[] bytes`，由服务层下载后传入，避免插件直连存储）。
- [ ] `PluginRegistry` 增加 `enabledDocumentReviewPlugins()`（`PluginType.DOCUMENT_REVIEW && isEnabled && isAvailable`）。
- [ ] 新增 `plugin/AiVisionReviewPlugin.java`（`@Component`，`isEnabledByDefault=false`，`isAvailable=properties.isConfigured()`，`getReviewMethod()=AI`）：
  - 复用 `AiReviewPlugin` 的 RestClient 构造与 `chat/completions` 调用；消息体用**多模态** `content` 数组：`[{type:"text",text:...},{type:"image_url",image_url:{url:"data:<mime>;base64,<b64>"}}]`。
  - SYSTEM_PROMPT 要求严格 JSON：`{"status","confidence","reason","extracted":{"fullName","idNumber"}}`。
  - 解析沿用 `stripCodeFence` + `jsonSupport.toMap` 思路（可抽取为共享工具）。
- [ ] 新增 `plugin/HeuristicDocumentPlugin.java`（`@Component`，`isEnabledByDefault=true`，`getReviewMethod()=RULE`）：
  - 校验：DOCUMENT 数量 ≥1、图片可 `ImageIO.read`、分辨率下限（如 ≥ 300px）、hash 非空。
  - 明显不合格 → `REJECTED(confidence≈0.6)`；否则 `PENDING(0.3)`（**不自动通过**）。

**验证**：`GET /plugins` 可见 `ai-vision-review`、`heuristic-document`，可开关。

---

## Phase 4 DTO 扩展

- [ ] `AuthTypeMetaDTO` 增加 `List<AuthLevelMetaDTO> levels`。
- [ ] 新增 `AuthLevelMetaDTO(level,label,List<String> requirements,List<String> tips)`。
- [ ] `AuthenticationSubmitRequest` 扩展：`type, level, data, List<Long> mediaIds, Long videoMediaId, String challengeId, Map liveness, Map attestation`（`type` 仍 `@NotBlank`）。
- [ ] `RetryRequest` 同结构（全部可选）。
- [ ] `AuthenticationDTO` 扩展：`level, List<AuthMediaDTO> media, Map liveness, Map attestation`（保留既有字段顺序，新增加在末尾以减少破坏）。
- [ ] 新增 `AuthMediaDTO(id,kind,contentType,sizeBytes)`、`AuthMediaUploadResponse(mediaId,kind,contentType,sizeBytes)`、`AuthChallengeDTO(challengeId,actions,nonce,expiresAt)`。

**验证**：`mvn -q compile` 通过。

---

## Phase 5 服务层

### 5.1 等级元数据
- [ ] 新增 `service/AuthLevelCatalog.java`：
  - `List<AuthLevelMetaDTO> levelsFor(AuthType type)`：返回 4 个等级，`requirements` 按类型×等级生成（如 `REAL_NAME` 的 MEDIUM 需「真实姓名 + 证件正面照」）。
  - `tips` 为通用拍摄注意事项（光线、无遮挡、正视、按提示动作）。
  - `types()`（`AuthenticationService`）为每个 meta 填充 `levels`。

### 5.2 挑战
- [ ] 新增 `service/AuthChallengeService.java`：
  - `issue(userId)`：从动作池随机 3 个 → 存 `AuthChallenge`（`expiresAt = now + ttl`）→ 返回 DTO。
  - `consume(challengeId,userId,liveness)`：校验存在、属于本人、未过期、未使用；校验 `liveness.actions` 覆盖 `actions` 全部动作且都 `passed`；标记 `usedAt`；非法抛 `ApiException.badRequest`。

### 5.3 媒体
- [ ] 新增 `service/AuthMediaService.java`：
  - `upload(MultipartFile, MediaKind, userId)`：按 kind 校验 MIME/大小（§5.3 约束）→ 计算 sha256 → `objectKey = auth/{userId}/{yyyyMMdd}/{uuid}.{ext}` → `storageService.put` → 存 `AuthMedia` → 返回响应。
  - `load(id, userId, isAdmin)`：鉴权（本人或管理员）→ 返回「存储流 + 元数据」供控制器流式响应。
  - `materialize(ids,userId)`：下载字节供插件使用。
  - `attach(mediaIds, authenticationId, userId)`：提交时绑定（校验归属）。

### 5.4 审核流水线
- [ ] 新增 `service/AuthenticationReviewService.java`，承接现 `AuthenticationService.review` 的职责并扩展：
  1. **结构校验**（§6.1）：按 `level` 检查文本字段（复用 `RuleBasedReviewPlugin`）、DOCUMENT、challenge+liveness+VIDEO、attestation；失败 → `REJECTED`（`Rule`）。
  2. **专业审核**：`pluginRegistry.enabledDocumentReviewPlugins()` 依次调用 `reviewDocuments`；聚合决策（任一高置信 REJECTED → REJECTED；最高置信 APPROVED 且 `≥ auto-approve-threshold` → APPROVED；否则 PENDING）。
  3. 写 `AuthReviewLog`（SUBMIT + 决策）。
  4. `level ≥ HIGH` 且 APPROVED → 调 5.5 自动登记。
- [ ] `AuthenticationService`：`submit/retry` 改为设置 `level`、`media/liveness/attestation` JSON 后委托 `AuthenticationReviewService`；`retry` 允许对 `REJECTED` 或以提升等级为目的重提（若记录已 APPROVED 且新 level 更高，允许覆盖）。
- [ ] `adminList` 增加 `level` 过滤参数（`findByStatusAndLevel`）。

### 5.5 自动实名登记
- [ ] 新增 `service/RealNameRegistrar.java`：
  - 触发类型白名单（§6.3）。
  - 取 `fullName`（提交数据优先，其次 AI `extracted.fullName`）；无则跳过。
  - upsert `REAL_NAME`：`APPROVED / level=HIGH / reviewMethod=SYSTEM`；等级不降级。
  - 写 `AuthReviewLog(action=APPROVE, method=SYSTEM)`。

### 5.6 门槛
- [ ] `AuthGateService.evaluate` 改造（§9）：解析 `TYPE` / `TYPE:LEVEL` / `AGE_n`；approved map 由 `type → AuthLevel`；满足条件为 `recordLevel.atLeast(requiredLevel)`；`AGE_n` 逻辑不变。

**验证**：单测
- `AuthLevel.parse/atLeast`；
- 门槛：`REAL_NAME:HIGH` 对 `MEDIUM` 用户 `missing` 非空；`AGE_18` 兼容通过；
- 结构校验：HIGH 缺视频 → REJECTED；
- 自动登记：`ID_CARD:HIGH` APPROVED 后生成 `REAL_NAME(HIGH, SYSTEM)`。

---

## Phase 6 控制器与安全

- [ ] `AuthenticationController` 新增：
  - `GET /authentications/challenge`
  - `POST /authentications/media`（`multipart/form-data`：`file`,`kind`）
  - `GET /authentications/media/{mediaId}`（返回 `ResponseEntity<StreamingResponseBody>` 或 `Resource`，带 `Content-Type`/`Content-Disposition:inline`）
  - `GET /authentications/types`、`POST /authentications`、`GET /me`、`GET /me/approved`、`POST /{id}/retry` 保持，接扩展 DTO。
- [ ] `AdminController.authentications` 增加 `level` 参数。
- [ ] `SecurityConfig`：`/authentications/types` 继续 public；新增 `/authentications/challenge`、`/authentications/media`、`/authentications/media/**` 保持 **authenticated**（**不要**加入 PUBLIC_PATHS）。

**验证**：未带 token 访问 `/authentications/media/**` 返回 401；带 token 本人可读，他人 403。

---

## Phase 7 前端类型与 API

- [ ] `types/index.ts`：
  - `export type AuthLevel = 'LOW'|'MEDIUM'|'HIGH'|'EXTREME'`
  - `AuthLevelMeta {level,label,requirements,tips}`；`AuthTypeMeta.levels?`
  - `AuthRecord` 增加 `level?/media?/liveness?/attestation?`
  - `AuthMediaInfo {id,kind,contentType,sizeBytes}`
  - `AuthChallenge {challengeId,actions,nonce,expiresAt}`
  - `AuthenticationSubmitPayload`（type/level/data/mediaIds/videoMediaId/challengeId/liveness/attestation）
- [ ] `utils/format.ts` 增加 `AUTH_LEVEL_LABELS`（低/中/高/极高）与 `authLevelLabel()`。
- [ ] `api/index.ts` `authenticationApi` 增加：
  - `challenge()`、`uploadMedia(file,kind)`（`FormData`，`Content-Type: multipart/form-data`）、`mediaUrl(id)`、`submit(payload)`（改签名）、`retry(id,payload?)`。
- [ ] `router/index.ts` 增加路由：`/authentication/:type/:level` → `AuthLevelWizard.vue`（`meta.requiresAuth`）。

**验证**：`npm run type-check` 通过。

---

## Phase 8 前端组件

### 8.1 依赖与静态资源
- [ ] `package.json` 增加 `@mediapipe/tasks-vision` 与 `vite-plugin-static-copy`。
- [ ] `vite.config.ts`：`viteStaticCopy` 将 `node_modules/@mediapipe/tasks-vision/wasm` → `dist/mediapipe`；模型放 `public/models/face_landmarker.task`（构建随 `public` 发布）。

### 8.2 `utils/liveness.ts`
- [ ] 封装：`createFaceLandmarker()`（WASM `/forum/mediapipe`、模型 `/forum/models/face_landmarker.task`）、`detectAction(frame, action)`（blendshapes/矩阵阈值，§7.2）、动作状态机（顺序推进）、`faceCount` 判定。
- [ ] 摄像头与录制：`getUserMedia({video:true})`、`MediaRecorder` 产出 `webm`，Canvas 叠加 `nonce` 水印。
- [ ] 导出 `runLiveness({videoEl,canvasEl,actions,nonce,onProgress}) => Promise<{blob, liveness}>`。
- [ ] 失败/不支持 → 抛出可识别错误，供降级。

### 8.3 `components/DocumentUpload.vue`
- [ ] props：`meta`、`minCount`、`maxCount`；emit `change(mediaIds, files)`。
- [ ] 支持 `<input type=file accept=image/* capture>`，逐张调用 `uploadMedia`，展示缩略图、删除、合规提示、上传进度。

### 8.4 `components/VideoCapture.vue`
- [ ] 步骤：摄像头授权 → 预览+动作引导（显示当前动作与完成勾选）→ 录制（含 nonce）→ 上传 `VIDEO` → 回传 `videoMediaId + liveness`。
- [ ] 降级：无权限/加载失败 → 「仅录制视频，转人工审核」。

### 8.5 `components/AuthLevelWizard.vue`
- [ ] 按 `route.params.type/level` 加载该类型 meta 与对应 `levels[]`。
- [ ] 步骤条：说明与注意事项 → 信息填写（复用 `AuthTypeForm`，改为受控取数据）→ 证件上传（MEDIUM+）→ 视频认证（HIGH+，先 `challenge()`）→（EXTREME 占位）→ 提交。
- [ ] 提交组装 `AuthenticationSubmitPayload`；结果页展示 status/reason，可 `retry`。

### 8.6 `AuthenticationCenter.vue` 改造
- [ ] 卡片展示「已达成的最高等级」标签（来自 `/me` 的 `level`）+ 状态。
- [ ] 「提交/重新提交」按钮跳转向导（`/authentication/:type/:level`），提供等级选择（低/中/高；极高置灰提示需 App）。

**验证**：`npm run build` 通过；本地起 dev，Chrome 授权摄像头完成一次 HIGH 全流程（可用 mock 图片/短视频）。

---

## Phase 9 管理端

- [ ] `Admin.vue` 待审认证列表：显示 `level` 标签、媒体缩略图（`authenticationApi.mediaUrl(id)`）、活体结果摘要；支持按 level 过滤。

**验证**：管理端能看到带等级的 PENDING 记录并查看媒体。

---

## Phase 10 测试与上线

- [ ] 后端单测（Phase 5 验证项）+ 结构校验/门槛/自动登记用例补齐。
- [ ] 构建产物：后端 `mvn -q package`；前端 `npm run build`。
- [ ] 部署：
  - 上传前端 `dist` 到 `/mydata/nginx/html/forum/` 并 `chmod -R a+rX`。
  - 重启/更新后端容器，注入 MinIO 与 AI 环境变量。
  - 确认 nginx `client_max_body_size 60m` 生效。
- [ ] 线上端到端（§14.2）：
  1. `LOW` 提交即时通过；
  2. `MEDIUM` 上传证件图 → AI 审核；
  3. `HIGH` challenge → 视频 → 提交 → 自动 `REAL_NAME`；
  4. `EXTREME` 返回「需 App」；
  5. 门槛 `REAL_NAME:HIGH` 子论坛：低等级 403、高等级通过。

---

## 回滚与兼容

- 新增字段均为可空/带默认值，旧数据 `level=LOW` 兼容；`/me/approved` 保持 `string[]` 不变。
- 未配置 MinIO/AI 时：`MEDIUM/HIGH` 走人工审核（PENDING），接口不报错。
- 门槛旧语法 `TYPE`、`AGE_n` 行为不变。
