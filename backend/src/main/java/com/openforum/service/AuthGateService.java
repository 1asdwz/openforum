package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.repository.UserAuthenticationRepository;
import org.springframework.stereotype.Service;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

/**
 * Evaluates the {@code required_auth_types} gate of a sub-forum.
 * Elements are one of:
 * <ul>
 *     <li>{@code TYPE} — 该类型已通过，等级 ≥ LOW（兼容旧数据）</li>
 *     <li>{@code TYPE:LEVEL} — 该类型已通过且等级 ≥ LEVEL，如 {@code REAL_NAME:HIGH}</li>
 *     <li>{@code AGE_n} — 兼容旧语法：AGE 已通过且年龄 ≥ n</li>
 * </ul>
 */
@Service
public class AuthGateService {

    private static final Pattern AGE_PATTERN = Pattern.compile("^AGE_(\\d+)$");

    private final UserAuthenticationRepository authenticationRepository;
    private final JsonSupport jsonSupport;

    public AuthGateService(UserAuthenticationRepository authenticationRepository, JsonSupport jsonSupport) {
        this.authenticationRepository = authenticationRepository;
        this.jsonSupport = jsonSupport;
    }

    public List<String> parseRequirements(String requiredCsv) {
        if (requiredCsv == null || requiredCsv.isBlank()) {
            return List.of();
        }
        List<String> requirements = new ArrayList<>();
        for (String part : requiredCsv.split(",")) {
            String trimmed = part.trim().toUpperCase();
            if (!trimmed.isEmpty()) {
                requirements.add(trimmed);
            }
        }
        return requirements;
    }

    public GateResult evaluate(Long userId, String requiredCsv) {
        List<String> requirements = parseRequirements(requiredCsv);
        if (requirements.isEmpty()) {
            return new GateResult(true, "无认证门槛", List.of());
        }
        if (userId == null) {
            return new GateResult(false, "请先登录后再访问该板块", List.copyOf(requirements));
        }
        Map<AuthType, UserAuthentication> approved = authenticationRepository
                .findByUserIdAndStatus(userId, AuthStatus.APPROVED)
                .stream()
                .collect(Collectors.toMap(UserAuthentication::getType, Function.identity(), (a, b) -> a));

        List<String> missing = new ArrayList<>();
        for (String requirement : requirements) {
            Matcher matcher = AGE_PATTERN.matcher(requirement);
            if (matcher.matches()) {
                int minimumAge = Integer.parseInt(matcher.group(1));
                UserAuthentication ageAuth = approved.get(AuthType.AGE);
                Integer ageValue = ageAuth == null ? null : intValue(jsonSupport.toMap(ageAuth.getAuthData()).get("age"));
                if (ageValue == null || ageValue < minimumAge) {
                    missing.add(requirement);
                }
                continue;
            }
            int separator = requirement.indexOf(':');
            String typePart = separator > 0 ? requirement.substring(0, separator) : requirement;
            String levelPart = separator > 0 ? requirement.substring(separator + 1) : null;
            AuthType type = parseType(typePart);
            AuthLevel requiredLevel = AuthLevel.parseOrNull(levelPart);
            if (requiredLevel == null) {
                requiredLevel = AuthLevel.LOW;
            }
            UserAuthentication record = type == null ? null : approved.get(type);
            if (record == null || !record.getLevel().atLeast(requiredLevel)) {
                missing.add(requirement);
            }
        }
        if (missing.isEmpty()) {
            return new GateResult(true, "已满足认证门槛", List.of());
        }
        return new GateResult(false, "需要完成认证: " + String.join(", ", missing), List.copyOf(missing));
    }

    private AuthType parseType(String value) {
        try {
            return AuthType.valueOf(value);
        } catch (IllegalArgumentException ex) {
            return null;
        }
    }

    private Integer intValue(Object value) {
        if (value instanceof Number number) {
            return number.intValue();
        }
        if (value == null) {
            return null;
        }
        try {
            return Integer.parseInt(String.valueOf(value).trim());
        } catch (NumberFormatException ex) {
            return null;
        }
    }

    public record GateResult(boolean passed, String message, List<String> missing) {
    }
}
