package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.entity.AuthReviewLog;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.ReviewAction;
import com.openforum.entity.enums.ReviewMethod;
import com.openforum.repository.AuthReviewLogRepository;
import com.openforum.repository.UserAuthenticationRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;

/**
 * 当 HIGH 及以上认证通过后，自动登记/升级该用户的实名认证（REAL_NAME）。
 */
@Service
public class RealNameRegistrar {

    private static final Logger log = LoggerFactory.getLogger(RealNameRegistrar.class);

    private static final Set<AuthType> TRIGGER_TYPES = Set.of(
            AuthType.ID_CARD,
            AuthType.DRIVER_LICENSE,
            AuthType.MARRIAGE_CERT,
            AuthType.PROPERTY_CERT,
            AuthType.NATIONALITY);

    private final UserAuthenticationRepository authenticationRepository;
    private final AuthReviewLogRepository reviewLogRepository;
    private final JsonSupport jsonSupport;

    public RealNameRegistrar(UserAuthenticationRepository authenticationRepository,
                             AuthReviewLogRepository reviewLogRepository, JsonSupport jsonSupport) {
        this.authenticationRepository = authenticationRepository;
        this.reviewLogRepository = reviewLogRepository;
        this.jsonSupport = jsonSupport;
    }

    @Transactional
    public void register(Long userId, AuthType sourceType, AuthLevel level,
                         Map<String, Object> data, Map<String, Object> extracted) {
        if (!TRIGGER_TYPES.contains(sourceType) || !level.atLeast(AuthLevel.HIGH)) {
            return;
        }
        String fullName = firstNonBlank(
                asString(extracted.get("fullName")),
                asString(data.get("fullName")),
                asString(extracted.get("name")),
                asString(data.get("name")));
        if (fullName == null) {
            log.debug("自动实名登记跳过：源认证 {} 未提供 fullName", sourceType);
            return;
        }
        UserAuthentication existing = authenticationRepository
                .findByUserIdAndType(userId, AuthType.REAL_NAME).orElse(null);
        if (existing != null && existing.getStatus() == AuthStatus.APPROVED
                && existing.getLevel().atLeast(AuthLevel.HIGH)) {
            return;
        }

        UserAuthentication target = existing != null ? existing : new UserAuthentication();
        target.setUserId(userId);
        target.setType(AuthType.REAL_NAME);

        Map<String, Object> realNameData = new LinkedHashMap<>(jsonSupport.toMap(target.getAuthData()));
        realNameData.put("fullName", fullName);
        String idNumber = firstNonBlank(
                asString(extracted.get("idNumber")),
                asString(data.get("idNumber")));
        if (idNumber != null) {
            realNameData.put("idNumber", idNumber);
        }
        target.setAuthData(jsonSupport.toJson(realNameData));
        target.setLevel(AuthLevel.HIGH);
        target.setStatus(AuthStatus.APPROVED);
        target.setReviewMethod(ReviewMethod.SYSTEM);
        target.setConfidence(1.0);
        target.setReason("由 " + sourceType.name() + ":" + level.name() + " 认证自动登记");
        target.setRejectionReason(null);
        target.setReviewedAt(LocalDateTime.now());
        authenticationRepository.save(target);

        AuthReviewLog reviewLog = new AuthReviewLog();
        reviewLog.setAuthenticationId(target.getId());
        reviewLog.setLevel(AuthLevel.HIGH);
        reviewLog.setAction(ReviewAction.APPROVE);
        reviewLog.setMethod(ReviewMethod.SYSTEM);
        reviewLog.setConfidence(1.0);
        reviewLog.setReason("由 " + sourceType.name() + ":" + level.name() + " 认证自动登记实名");
        reviewLogRepository.save(reviewLog);
    }

    private String asString(Object value) {
        if (value == null) {
            return null;
        }
        String text = String.valueOf(value).trim();
        return text.isEmpty() ? null : text;
    }

    private String firstNonBlank(String... values) {
        for (String value : values) {
            if (value != null && !value.isBlank()) {
                return value;
            }
        }
        return null;
    }
}
