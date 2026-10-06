package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.common.PageResponse;
import com.openforum.dto.authentication.AuthFieldDTO;
import com.openforum.dto.authentication.AuthMediaDTO;
import com.openforum.dto.authentication.AuthTypeMetaDTO;
import com.openforum.dto.authentication.AuthenticationDTO;
import com.openforum.dto.authentication.AuthenticationSubmitRequest;
import com.openforum.dto.authentication.RetryRequest;
import com.openforum.entity.AuthMedia;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.ReviewMethod;
import com.openforum.exception.ApiException;
import com.openforum.repository.AuthMediaRepository;
import com.openforum.repository.UserAuthenticationRepository;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Sort;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Map;

@Service
public class AuthenticationService {

    private final UserAuthenticationRepository authenticationRepository;
    private final AuthMediaRepository mediaRepository;
    private final AuthenticationReviewService reviewService;
    private final AuthLevelCatalog levelCatalog;
    private final JsonSupport jsonSupport;

    public AuthenticationService(UserAuthenticationRepository authenticationRepository,
                                 AuthMediaRepository mediaRepository,
                                 AuthenticationReviewService reviewService,
                                 AuthLevelCatalog levelCatalog,
                                 JsonSupport jsonSupport) {
        this.authenticationRepository = authenticationRepository;
        this.mediaRepository = mediaRepository;
        this.reviewService = reviewService;
        this.levelCatalog = levelCatalog;
        this.jsonSupport = jsonSupport;
    }

    public List<AuthTypeMetaDTO> types() {
        return List.of(
                meta(AuthType.REAL_NAME, "实名认证", field("fullName", "真实姓名", "text", true)),
                meta(AuthType.ID_CARD, "身份证认证", field("idNumber", "身份证号", "text", true)),
                meta(AuthType.GENDER, "性别认证", field("gender", "性别", "select", true)),
                meta(AuthType.AGE, "年龄认证", field("age", "年龄", "number", true)),
                meta(AuthType.NATIONALITY, "国籍认证", field("nationality", "国籍", "text", true)),
                meta(AuthType.DRIVER_LICENSE, "驾照认证", field("licenseNumber", "驾照号", "text", true)),
                meta(AuthType.MARRIAGE_CERT, "婚姻认证",
                        field("spouseName", "配偶姓名", "text", true),
                        field("certificateNumber", "结婚证号", "text", true)),
                meta(AuthType.PROPERTY_CERT, "房产认证",
                        field("certificateNumber", "产权证号", "text", true),
                        field("propertyAddress", "房产地址", "text", true)),
                meta(AuthType.ASSETS, "资产认证", field("amount", "资产金额", "number", true)),
                meta(AuthType.CUSTOM, "自定义认证", field("description", "说明", "textarea", true)));
    }

    @Transactional
    public AuthenticationDTO submit(AuthenticationSubmitRequest request, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        AuthType type = parseType(request.type());
        AuthLevel level = AuthLevel.parse(request.level());
        UserAuthentication existing = authenticationRepository.findByUserIdAndType(userId, type).orElse(null);
        if (existing != null) {
            boolean canUpgrade = existing.getStatus() == AuthStatus.REJECTED
                    || existing.getLevel().rank() < level.rank();
            if (!canUpgrade) {
                throw ApiException.badRequest("该类型认证已存在，如需重新提交请使用重试接口");
            }
        }
        UserAuthentication result = reviewService.review(userId, type, level, request.data(),
                request.mediaIds(), request.videoMediaId(), request.challengeId(),
                request.liveness(), request.attestation(), existing);
        return toDto(result);
    }

    @Transactional
    public AuthenticationDTO retry(Long id, RetryRequest request, Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        UserAuthentication auth = require(id);
        if (!auth.getUserId().equals(userId)) {
            throw ApiException.forbidden("无权操作该认证");
        }
        AuthLevel level = request.level() == null || request.level().isBlank()
                ? auth.getLevel() : AuthLevel.parse(request.level());
        boolean rejected = auth.getStatus() == AuthStatus.REJECTED;
        boolean upgrade = level.rank() > auth.getLevel().rank();
        if (!rejected && !upgrade) {
            throw ApiException.badRequest("仅被拒绝的认证或提升等级时可以重新提交");
        }
        Map<String, Object> data = request.data() != null ? request.data() : jsonSupport.toMap(auth.getAuthData());
        UserAuthentication result = reviewService.review(userId, auth.getType(), level, data,
                request.mediaIds(), request.videoMediaId(), request.challengeId(),
                request.liveness(), request.attestation(), auth);
        return toDto(result);
    }

    public List<AuthenticationDTO> my(Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        return authenticationRepository.findByUserIdOrderByCreatedAtDesc(userId).stream()
                .map(this::toDto)
                .toList();
    }

    public List<String> approved(Long userId) {
        if (userId == null) {
            throw ApiException.unauthorized("请先登录");
        }
        return authenticationRepository.findByUserIdAndStatus(userId, AuthStatus.APPROVED).stream()
                .map(auth -> auth.getType().name())
                .sorted()
                .toList();
    }

    public PageResponse<AuthenticationDTO> adminList(String status, String level, int page, int size) {
        AuthStatus authStatus = parseStatus(status);
        AuthLevel authLevel = AuthLevel.parseOrNull(level);
        var pageable = PageRequest.of(page, size, Sort.by(Sort.Direction.ASC, "createdAt"));
        var result = authLevel == null
                ? authenticationRepository.findByStatus(authStatus, pageable)
                : authenticationRepository.findByStatusAndLevel(authStatus, authLevel, pageable);
        return PageResponse.of(result.map(this::toDto));
    }

    @Transactional
    public AuthenticationDTO approve(Long id, String reason, Long adminId) {
        UserAuthentication auth = require(id);
        auth.setStatus(AuthStatus.APPROVED);
        auth.setReviewMethod(ReviewMethod.MANUAL);
        auth.setReviewedBy(adminId);
        auth.setReviewedAt(LocalDateTime.now());
        auth.setReason(reason == null || reason.isBlank() ? "管理员通过" : reason);
        auth.setRejectionReason(null);
        authenticationRepository.save(auth);
        return toDto(auth);
    }

    @Transactional
    public AuthenticationDTO reject(Long id, String reason, Long adminId) {
        UserAuthentication auth = require(id);
        auth.setStatus(AuthStatus.REJECTED);
        auth.setReviewMethod(ReviewMethod.MANUAL);
        auth.setReviewedBy(adminId);
        auth.setReviewedAt(LocalDateTime.now());
        auth.setRejectionReason(reason == null || reason.isBlank() ? "管理员驳回" : reason);
        authenticationRepository.save(auth);
        return toDto(auth);
    }

    private UserAuthentication require(Long id) {
        return authenticationRepository.findById(id)
                .orElseThrow(() -> ApiException.notFound("认证记录不存在"));
    }

    private AuthType parseType(String value) {
        if (value == null || value.isBlank()) {
            throw ApiException.badRequest("type 不能为空");
        }
        try {
            return AuthType.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            throw ApiException.badRequest("未知认证类型: " + value);
        }
    }

    private AuthStatus parseStatus(String value) {
        if (value == null || value.isBlank()) {
            return AuthStatus.PENDING;
        }
        try {
            return AuthStatus.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            throw ApiException.badRequest("非法的状态: " + value);
        }
    }

    private AuthTypeMetaDTO meta(AuthType type, String label, AuthFieldDTO... fields) {
        return new AuthTypeMetaDTO(type.name(), label, List.of(fields), levelCatalog.levelsFor(type));
    }

    private AuthFieldDTO field(String name, String label, String type, boolean required) {
        return new AuthFieldDTO(name, label, type, required);
    }

    private AuthenticationDTO toDto(UserAuthentication auth) {
        List<AuthMediaDTO> media = jsonSupport.toLongList(auth.getMedia()).stream()
                .map(mediaRepository::findById)
                .filter(java.util.Optional::isPresent)
                .map(java.util.Optional::get)
                .map(this::toMediaDto)
                .toList();
        return new AuthenticationDTO(
                auth.getId(),
                auth.getUserId(),
                auth.getType().name(),
                label(auth.getType()),
                auth.getStatus().name(),
                auth.getReviewMethod() == null ? null : auth.getReviewMethod().name(),
                auth.getConfidence(),
                auth.getReason(),
                auth.getRejectionReason(),
                jsonSupport.toMap(auth.getAuthData()),
                auth.getCreatedAt(),
                auth.getReviewedAt(),
                auth.getLevel().name(),
                media,
                jsonSupport.toMap(auth.getLiveness()),
                jsonSupport.toMap(auth.getAttestation()));
    }

    private AuthMediaDTO toMediaDto(AuthMedia media) {
        return new AuthMediaDTO(media.getId(), media.getKind().name(), media.getContentType(), media.getSizeBytes());
    }

    private String label(AuthType type) {
        return types().stream()
                .filter(meta -> meta.type().equals(type.name()))
                .map(AuthTypeMetaDTO::label)
                .findFirst()
                .orElse(type.name());
    }
}
