package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.repository.UserAuthenticationRepository;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class AuthGateServiceTest {

    private static final Long USER_ID = 1L;

    @Mock
    private UserAuthenticationRepository repository;

    @Mock
    private JsonSupport jsonSupport;

    private AuthGateService service;

    @BeforeEach
    void setUp() {
        service = new AuthGateService(repository, jsonSupport);
    }

    private UserAuthentication approved(AuthType type, AuthLevel level, String authData) {
        UserAuthentication record = new UserAuthentication();
        record.setUserId(USER_ID);
        record.setType(type);
        record.setLevel(level);
        record.setStatus(AuthStatus.APPROVED);
        record.setAuthData(authData);
        return record;
    }

    @Test
    void emptyRequirementAlwaysPasses() {
        assertTrue(service.evaluate(USER_ID, "  ").passed());
        assertTrue(service.evaluate(USER_ID, null).passed());
    }

    @Test
    void anonymousUserFailsWhenRequirementPresent() {
        AuthGateService.GateResult result = service.evaluate(null, "REAL_NAME");
        assertFalse(result.passed());
        assertTrue(result.missing().contains("REAL_NAME"));
    }

    @Test
    void higherLevelSatisfiesLowerRequirement() {
        when(repository.findByUserIdAndStatus(USER_ID, AuthStatus.APPROVED))
                .thenReturn(List.of(approved(AuthType.REAL_NAME, AuthLevel.HIGH, null)));

        assertTrue(service.evaluate(USER_ID, "REAL_NAME:MEDIUM").passed());
        assertTrue(service.evaluate(USER_ID, "REAL_NAME").passed());
    }

    @Test
    void lowerLevelDoesNotSatisfyHigherRequirement() {
        when(repository.findByUserIdAndStatus(USER_ID, AuthStatus.APPROVED))
                .thenReturn(List.of(approved(AuthType.REAL_NAME, AuthLevel.MEDIUM, null)));

        AuthGateService.GateResult result = service.evaluate(USER_ID, "REAL_NAME:HIGH");
        assertFalse(result.passed());
        assertTrue(result.missing().contains("REAL_NAME:HIGH"));
    }

    @Test
    void ageRequirementUsesAgeFieldOfApprovedRecord() {
        when(repository.findByUserIdAndStatus(USER_ID, AuthStatus.APPROVED))
                .thenReturn(List.of(approved(AuthType.AGE, AuthLevel.LOW, "{\"age\":20}")));
        when(jsonSupport.toMap("{\"age\":20}")).thenReturn(Map.of("age", 20));

        assertTrue(service.evaluate(USER_ID, "AGE_18").passed());
        assertFalse(service.evaluate(USER_ID, "AGE_21").passed());
    }

    @Test
    void typeLevelAndAgeCombineInSingleGate() {
        when(repository.findByUserIdAndStatus(USER_ID, AuthStatus.APPROVED))
                .thenReturn(List.of(
                        approved(AuthType.REAL_NAME, AuthLevel.HIGH, null),
                        approved(AuthType.AGE, AuthLevel.LOW, "{\"age\":30}")));
        when(jsonSupport.toMap("{\"age\":30}")).thenReturn(Map.of("age", 30));

        assertTrue(service.evaluate(USER_ID, "REAL_NAME:HIGH,AGE_18").passed());
    }
}
