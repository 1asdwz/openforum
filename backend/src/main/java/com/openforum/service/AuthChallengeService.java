package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.config.OpenForumAuthProperties;
import com.openforum.dto.authentication.AuthChallengeDTO;
import com.openforum.entity.AuthChallenge;
import com.openforum.exception.ApiException;
import com.openforum.repository.AuthChallengeRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.security.SecureRandom;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * 活体检测挑战：随机动作序列、一次性、绑定用户、限时有效。
 */
@Service
public class AuthChallengeService {

    private static final List<String> ACTION_POOL =
            List.of("BLINK", "OPEN_MOUTH", "TURN_LEFT", "TURN_RIGHT", "NOD");
    private static final int ACTION_COUNT = 3;

    private final AuthChallengeRepository repository;
    private final JsonSupport jsonSupport;
    private final OpenForumAuthProperties properties;
    private final SecureRandom random = new SecureRandom();

    public AuthChallengeService(AuthChallengeRepository repository, JsonSupport jsonSupport,
                                OpenForumAuthProperties properties) {
        this.repository = repository;
        this.jsonSupport = jsonSupport;
        this.properties = properties;
    }

    @Transactional
    public AuthChallengeDTO issue(Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        List<String> actions = new ArrayList<>(ACTION_POOL);
        Collections.shuffle(actions, random);
        List<String> picked = List.copyOf(actions.subList(0, ACTION_COUNT));

        AuthChallenge challenge = new AuthChallenge();
        challenge.setId(UUID.randomUUID().toString());
        challenge.setUserId(userId);
        challenge.setActions(jsonSupport.toJson(picked));
        challenge.setNonce(randomNonce());
        challenge.setExpiresAt(LocalDateTime.now().plusMinutes(properties.challengeTtlMinutes()));
        repository.save(challenge);

        return new AuthChallengeDTO(challenge.getId(), picked, challenge.getNonce(), challenge.getExpiresAt());
    }

    /**
     * 校验并消费挑战。允许「降级」模式（degraded=true，转人工），但要求挑战合法且未使用。
     */
    @Transactional
    public void consume(String challengeId, Long userId, Map<String, Object> liveness) {
        if (challengeId == null || challengeId.isBlank()) {
            throw ApiException.badRequest("缺少活体挑战标识");
        }
        AuthChallenge challenge = repository.findByIdAndUserId(challengeId, userId)
                .orElseThrow(() -> ApiException.badRequest("活体挑战无效或不属于当前用户"));
        if (challenge.getUsedAt() != null) {
            throw ApiException.badRequest("活体挑战已使用，请重新开始视频认证");
        }
        if (challenge.getExpiresAt().isBefore(LocalDateTime.now())) {
            throw ApiException.badRequest("活体挑战已过期，请重新开始视频认证");
        }
        if (liveness == null || liveness.isEmpty()) {
            throw ApiException.badRequest("缺少活体检测结果");
        }
        boolean degraded = Boolean.TRUE.equals(liveness.get("degraded"));
        if (Boolean.TRUE.equals(liveness.get("passed"))) {
            validateActions(jsonSupport.toStringList(challenge.getActions()), liveness);
        } else if (!degraded) {
            throw ApiException.badRequest("活体检测未通过，请重新进行视频认证");
        }
        challenge.setUsedAt(LocalDateTime.now());
        repository.save(challenge);
    }

    @SuppressWarnings("unchecked")
    private void validateActions(List<String> required, Map<String, Object> liveness) {
        Object actionsValue = liveness.get("actions");
        if (!(actionsValue instanceof List<?> list)) {
            throw ApiException.badRequest("活体检测结果缺少动作记录");
        }
        for (String action : required) {
            boolean passed = list.stream()
                    .filter(Map.class::isInstance)
                    .map(item -> (Map<String, Object>) item)
                    .anyMatch(item -> action.equals(String.valueOf(item.get("action")))
                            && Boolean.TRUE.equals(item.get("passed")));
            if (!passed) {
                throw ApiException.badRequest("未完成指定动作: " + action);
            }
        }
    }

    private String randomNonce() {
        byte[] bytes = new byte[16];
        random.nextBytes(bytes);
        StringBuilder sb = new StringBuilder(bytes.length * 2);
        for (byte b : bytes) {
            sb.append(String.format("%02x", b));
        }
        return sb.toString();
    }
}
