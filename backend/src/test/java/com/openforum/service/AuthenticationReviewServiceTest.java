package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.config.OpenForumAuthProperties;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.MediaKind;
import com.openforum.entity.enums.ReviewMethod;
import com.openforum.plugin.DocumentReviewPlugin;
import com.openforum.plugin.PluginRegistry;
import com.openforum.plugin.PluginType;
import com.openforum.plugin.ReviewResult;
import com.openforum.repository.AuthReviewLogRepository;
import com.openforum.repository.UserAuthenticationRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import tools.jackson.databind.json.JsonMapper;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.atLeastOnce;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class AuthenticationReviewServiceTest {

    private static final Long USER_ID = 1L;
    private static final Long MEDIA_ID = 11L;
    private static final Long VIDEO_ID = 12L;
    private static final String CHALLENGE_ID = "challenge-1";
    private static final String ID_NUMBER = "110101199001011234";

    @Mock
    private UserAuthenticationRepository authenticationRepository;

    @Mock
    private AuthReviewLogRepository reviewLogRepository;

    @Mock
    private AuthChallengeService challengeService;

    @Mock
    private AuthMediaService mediaService;

    @Mock
    private PluginRegistry pluginRegistry;

    private AuthenticationReviewService service;

    @BeforeEach
    void setUp() {
        JsonSupport jsonSupport = new JsonSupport(JsonMapper.builder().build());
        OpenForumAuthProperties properties = new OpenForumAuthProperties(null, null, null, null, null);
        RealNameRegistrar realNameRegistrar =
                new RealNameRegistrar(authenticationRepository, reviewLogRepository, jsonSupport);
        service = new AuthenticationReviewService(authenticationRepository, reviewLogRepository,
                challengeService, mediaService, pluginRegistry, jsonSupport, properties, realNameRegistrar);

        when(authenticationRepository.save(any(UserAuthentication.class)))
                .thenAnswer(invocation -> invocation.getArgument(0));
    }

    @Test
    void highLevelWithoutVideoIsRejectedByRule() {
        UserAuthentication result = service.review(USER_ID, AuthType.ID_CARD, AuthLevel.HIGH,
                Map.of("idNumber", ID_NUMBER), List.of(MEDIA_ID), null, null, null, null, null);

        assertEquals(AuthStatus.REJECTED, result.getStatus());
        assertEquals(ReviewMethod.RULE, result.getReviewMethod());
        assertTrue(result.getRejectionReason().contains("视频"));
    }

    @Test
    void mediumLevelWithoutDocumentIsRejectedByRule() {
        UserAuthentication result = service.review(USER_ID, AuthType.ID_CARD, AuthLevel.MEDIUM,
                Map.of("idNumber", ID_NUMBER), null, null, null, null, null, null);

        assertEquals(AuthStatus.REJECTED, result.getStatus());
        assertEquals(ReviewMethod.RULE, result.getReviewMethod());
        assertTrue(result.getRejectionReason().contains("证件"));
    }

    @Test
    void extremeLevelWithoutAttestationIsRejectedByRule() {
        UserAuthentication result = service.review(USER_ID, AuthType.ID_CARD, AuthLevel.EXTREME,
                Map.of("idNumber", ID_NUMBER), List.of(MEDIA_ID), VIDEO_ID, CHALLENGE_ID,
                Map.of("passed", true), null, null);

        assertEquals(AuthStatus.REJECTED, result.getStatus());
        assertEquals(ReviewMethod.RULE, result.getReviewMethod());
        assertTrue(result.getRejectionReason().contains("App"));
    }

    @Test
    void approvedIdCardAtHighLevelRegistersRealNameAutomatically() {
        when(authenticationRepository.findByUserIdAndType(USER_ID, AuthType.REAL_NAME))
                .thenReturn(Optional.empty());
        when(mediaService.materialize(eq(List.of(MEDIA_ID)), eq(USER_ID)))
                .thenReturn(List.of(new DocumentReviewPlugin.MediaRef(
                        MEDIA_ID, MediaKind.DOCUMENT, "image/jpeg", new byte[]{1})));
        when(pluginRegistry.enabledDocumentReviewPlugins())
                .thenReturn(List.of(new ApprovingDocumentPlugin()));

        UserAuthentication result = service.review(USER_ID, AuthType.ID_CARD, AuthLevel.HIGH,
                Map.of("idNumber", ID_NUMBER, "fullName", "张三"),
                List.of(MEDIA_ID), VIDEO_ID, CHALLENGE_ID, Map.of("passed", true), null, null);

        assertEquals(AuthStatus.APPROVED, result.getStatus());
        verify(challengeService).consume(eq(CHALLENGE_ID), eq(USER_ID), any());

        ArgumentCaptor<UserAuthentication> captor = ArgumentCaptor.forClass(UserAuthentication.class);
        verify(authenticationRepository, atLeastOnce()).save(captor.capture());
        UserAuthentication realName = captor.getAllValues().stream()
                .filter(record -> record.getType() == AuthType.REAL_NAME)
                .findFirst()
                .orElseThrow(() -> new AssertionError("未生成 REAL_NAME 认证记录"));

        assertEquals(AuthLevel.HIGH, realName.getLevel());
        assertEquals(AuthStatus.APPROVED, realName.getStatus());
        assertEquals(ReviewMethod.SYSTEM, realName.getReviewMethod());
        assertNotNull(realName.getAuthData());
        assertTrue(realName.getAuthData().contains("张三"));
        assertFalse(realName.getAuthData().isBlank());
    }

    private static final class ApprovingDocumentPlugin implements DocumentReviewPlugin {

        @Override
        public String getKey() {
            return "test-document-approve";
        }

        @Override
        public String getName() {
            return "测试证件审核";
        }

        @Override
        public PluginType getType() {
            return PluginType.DOCUMENT_REVIEW;
        }

        @Override
        public boolean isEnabledByDefault() {
            return true;
        }

        @Override
        public String getDescription() {
            return "测试用高置信通过插件";
        }

        @Override
        public boolean isAvailable() {
            return true;
        }

        @Override
        public ReviewMethod getReviewMethod() {
            return ReviewMethod.AI;
        }

        @Override
        public ReviewResult reviewDocuments(DocumentReviewContext ctx) {
            return new ReviewResult(AuthStatus.APPROVED, 0.95, "AI 审核通过");
        }
    }
}
