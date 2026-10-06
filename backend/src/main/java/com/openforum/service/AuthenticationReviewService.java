package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.config.OpenForumAuthProperties;
import com.openforum.entity.AuthReviewLog;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.MediaKind;
import com.openforum.entity.enums.ReviewAction;
import com.openforum.entity.enums.ReviewMethod;
import com.openforum.exception.ApiException;
import com.openforum.plugin.DocumentReviewPlugin;
import com.openforum.plugin.PluginRegistry;
import com.openforum.plugin.ReviewPlugin;
import com.openforum.plugin.ReviewResult;
import com.openforum.repository.AuthReviewLogRepository;
import com.openforum.repository.UserAuthenticationRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Map;

/**
 * 结构化审核流水线：按等级做硬性结构校验 → 调用专业审核插件 → 汇总决策 → 自动实名登记。
 */
@Service
public class AuthenticationReviewService {

    private static final Logger log = LoggerFactory.getLogger(AuthenticationReviewService.class);

    private final UserAuthenticationRepository authenticationRepository;
    private final AuthReviewLogRepository reviewLogRepository;
    private final AuthChallengeService challengeService;
    private final AuthMediaService mediaService;
    private final PluginRegistry pluginRegistry;
    private final JsonSupport jsonSupport;
    private final OpenForumAuthProperties properties;
    private final RealNameRegistrar realNameRegistrar;

    public AuthenticationReviewService(UserAuthenticationRepository authenticationRepository,
                                       AuthReviewLogRepository reviewLogRepository,
                                       AuthChallengeService challengeService,
                                       AuthMediaService mediaService,
                                       PluginRegistry pluginRegistry,
                                       JsonSupport jsonSupport,
                                       OpenForumAuthProperties properties,
                                       RealNameRegistrar realNameRegistrar) {
        this.authenticationRepository = authenticationRepository;
        this.reviewLogRepository = reviewLogRepository;
        this.challengeService = challengeService;
        this.mediaService = mediaService;
        this.pluginRegistry = pluginRegistry;
        this.jsonSupport = jsonSupport;
        this.properties = properties;
        this.realNameRegistrar = realNameRegistrar;
    }

    @Transactional
    public UserAuthentication review(Long userId, AuthType type, AuthLevel level, Map<String, Object> data,
                                     List<Long> mediaIds, Long videoMediaId, String challengeId,
                                     Map<String, Object> liveness, Map<String, Object> attestation,
                                     UserAuthentication existing) {
        Map<String, Object> safeData = data == null ? Map.of() : data;
        UserAuthentication auth = existing != null ? existing : new UserAuthentication();
        auth.setUserId(userId);
        auth.setType(type);
        auth.setLevel(level);
        auth.setAuthData(jsonSupport.toJson(safeData));
        auth.setMedia(mediaIds == null || mediaIds.isEmpty() ? null : jsonSupport.toJson(mediaIds));
        auth.setLiveness(liveness == null || liveness.isEmpty() ? null : jsonSupport.toJson(liveness));
        auth.setAttestation(attestation == null || attestation.isEmpty() ? null : jsonSupport.toJson(attestation));
        auth.setReviewedBy(null);
        auth.setRejectionReason(null);
        auth.setStatus(AuthStatus.PENDING);
        auth = authenticationRepository.save(auth);

        logSubmit(auth, userId);

        // 1. 结构校验
        String structureError = validateStructure(userId, type, level, safeData, mediaIds, videoMediaId,
                challengeId, liveness, attestation);
        if (structureError != null) {
            return finish(auth, AuthStatus.REJECTED, ReviewMethod.RULE, 0.95, structureError, structureError, Map.of());
        }

        // 2. LOW：直接采用规则引擎结论
        if (level == AuthLevel.LOW) {
            ReviewResult rule = ruleReview(type, safeData);
            if (rule.status() == AuthStatus.REJECTED) {
                return finish(auth, AuthStatus.REJECTED, ReviewMethod.RULE,
                        rule.confidence(), rule.reason(), rule.reason(), Map.of());
            }
            return finish(auth, AuthStatus.APPROVED, ReviewMethod.RULE,
                    rule.confidence(), rule.reason(), null, Map.of());
        }

        // 3. 绑定媒体
        if (mediaIds != null && !mediaIds.isEmpty()) {
            mediaService.attach(mediaIds, auth.getId(), userId);
        }

        // 4. 专业审核
        List<DocumentReviewPlugin.MediaRef> documents = mediaService.materialize(mediaIds, userId);
        boolean hasDocument = documents.stream().anyMatch(doc -> doc.kind() == MediaKind.DOCUMENT);
        if (!hasDocument) {
            return finish(auth, AuthStatus.REJECTED, ReviewMethod.RULE, 0.9,
                    "至少需要上传 1 张证件照片", "缺少证件照片", Map.of());
        }
        DocumentReviewPlugin.DocumentReviewContext ctx =
                new DocumentReviewPlugin.DocumentReviewContext(type, level, safeData, documents, null, null);
        ReviewOutcome outcome = runPlugins(ctx);

        // 5. 活体降级：不自动通过，转人工
        boolean livenessPassed = liveness != null && Boolean.TRUE.equals(liveness.get("passed"));
        if (level.atLeast(AuthLevel.HIGH) && !livenessPassed && outcome.status() != AuthStatus.REJECTED) {
            return finish(auth, AuthStatus.PENDING, ReviewMethod.MANUAL, 0.0,
                    "活体检测未完成，转人工审核", null, outcome.extracted());
        }

        if (outcome.status() == AuthStatus.APPROVED) {
            UserAuthentication saved = finish(auth, AuthStatus.APPROVED, outcome.method(),
                    outcome.confidence(), outcome.reason(), null, outcome.extracted());
            realNameRegistrar.register(userId, type, level, safeData, outcome.extracted());
            return saved;
        }
        if (outcome.status() == AuthStatus.REJECTED) {
            return finish(auth, AuthStatus.REJECTED, outcome.method(),
                    outcome.confidence(), outcome.reason(), outcome.reason(), outcome.extracted());
        }
        return finish(auth, AuthStatus.PENDING, ReviewMethod.MANUAL, 0.0,
                outcome.reason(), null, outcome.extracted());
    }

    private String validateStructure(Long userId, AuthType type, AuthLevel level, Map<String, Object> data,
                                     List<Long> mediaIds, Long videoMediaId, String challengeId,
                                     Map<String, Object> liveness, Map<String, Object> attestation) {
        ReviewResult rule = ruleReview(type, data);
        if (rule.status() == AuthStatus.REJECTED) {
            return rule.reason();
        }
        if (!level.atLeast(AuthLevel.MEDIUM)) {
            return null;
        }
        if (mediaIds == null || mediaIds.isEmpty()) {
            return "该等级需要上传证件材料";
        }
        if (mediaIds.size() > properties.maxDocumentCount()) {
            return "证件材料数量超出上限（最多 " + properties.maxDocumentCount() + " 张）";
        }
        if (level.atLeast(AuthLevel.HIGH)) {
            if (videoMediaId == null) {
                return "该等级需要完成视频认证";
            }
            challengeService.consume(challengeId, userId, liveness);
        }
        if (level == AuthLevel.EXTREME && (attestation == null || attestation.isEmpty())) {
            return "极高等级需要通过移动 App 完成防伪录制";
        }
        return null;
    }

    private ReviewOutcome runPlugins(DocumentReviewPlugin.DocumentReviewContext ctx) {
        List<DocumentReviewPlugin> plugins = pluginRegistry.enabledDocumentReviewPlugins();
        if (plugins.isEmpty()) {
            return new ReviewOutcome(AuthStatus.PENDING, ReviewMethod.MANUAL, 0.0,
                    "等待人工审核", Map.of());
        }
        ReviewResult bestApproved = null;
        ReviewMethod bestMethod = ReviewMethod.MANUAL;
        ReviewOutcome bestRejected = null;
        Map<String, Object> extracted = Map.of();
        double threshold = properties.autoApproveThreshold();
        for (DocumentReviewPlugin plugin : plugins) {
            ReviewResult result;
            try {
                result = plugin.reviewDocuments(ctx);
            } catch (Exception ex) {
                log.warn("证件审核插件 {} 执行失败: {}", plugin.getKey(), ex.getMessage());
                continue;
            }
            if (!result.extracted().isEmpty()) {
                extracted = result.extracted();
            }
            if (result.status() == AuthStatus.REJECTED) {
                if (bestRejected == null || result.confidence() > bestRejected.confidence()) {
                    bestRejected = new ReviewOutcome(AuthStatus.REJECTED, plugin.getReviewMethod(),
                            result.confidence(), result.reason(), result.extracted());
                }
            } else if (result.status() == AuthStatus.APPROVED) {
                if (bestApproved == null || result.confidence() > bestApproved.confidence()) {
                    bestApproved = result;
                    bestMethod = plugin.getReviewMethod();
                }
            }
        }
        if (bestRejected != null) {
            return bestRejected;
        }
        if (bestApproved != null && bestApproved.confidence() >= threshold) {
            return new ReviewOutcome(AuthStatus.APPROVED, bestMethod, bestApproved.confidence(),
                    bestApproved.reason(), extracted);
        }
        return new ReviewOutcome(AuthStatus.PENDING, ReviewMethod.MANUAL, 0.0,
                "材料已收到，等待人工复核", extracted);
    }

    private ReviewResult ruleReview(AuthType type, Map<String, Object> data) {
        return pluginRegistry.byKey("rule-based-review")
                .filter(ReviewPlugin.class::isInstance)
                .map(ReviewPlugin.class::cast)
                .map(plugin -> plugin.review(type, data))
                .orElse(new ReviewResult(AuthStatus.APPROVED, 0.5, "未启用规则引擎"));
    }

    private UserAuthentication finish(UserAuthentication auth, AuthStatus status, ReviewMethod method,
                                      double confidence, String reason, String rejectionReason,
                                      Map<String, Object> extracted) {
        auth.setStatus(status);
        auth.setReviewMethod(method);
        auth.setConfidence(confidence);
        auth.setReason(reason);
        auth.setRejectionReason(rejectionReason);
        if (status != AuthStatus.PENDING) {
            auth.setReviewedAt(LocalDateTime.now());
        }
        UserAuthentication saved = authenticationRepository.save(auth);
        logDecision(saved, status, method, confidence, reason);
        return saved;
    }

    private void logSubmit(UserAuthentication auth, Long userId) {
        AuthReviewLog reviewLog = new AuthReviewLog();
        reviewLog.setAuthenticationId(auth.getId());
        reviewLog.setLevel(auth.getLevel());
        reviewLog.setAction(ReviewAction.SUBMIT);
        reviewLog.setMethod(auth.getReviewMethod());
        reviewLog.setActorId(userId);
        reviewLog.setReason("提交 " + auth.getType() + ":" + auth.getLevel());
        if (auth.getId() != null) {
            reviewLogRepository.save(reviewLog);
        }
    }

    private void logDecision(UserAuthentication auth, AuthStatus status, ReviewMethod method,
                             double confidence, String reason) {
        AuthReviewLog reviewLog = new AuthReviewLog();
        reviewLog.setAuthenticationId(auth.getId());
        reviewLog.setLevel(auth.getLevel());
        reviewLog.setAction(status == AuthStatus.APPROVED ? ReviewAction.APPROVE
                : status == AuthStatus.REJECTED ? ReviewAction.REJECT : ReviewAction.SUBMIT);
        reviewLog.setMethod(method);
        reviewLog.setConfidence(confidence);
        reviewLog.setReason(reason);
        reviewLogRepository.save(reviewLog);
    }

    private record ReviewOutcome(AuthStatus status, ReviewMethod method, double confidence,
                                 String reason, Map<String, Object> extracted) {
    }
}
