package com.openforum.entity.enums;

import com.openforum.exception.ApiException;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class AuthLevelTest {

    @Test
    void parseIsCaseInsensitiveAndDefaultsToLow() {
        assertEquals(AuthLevel.HIGH, AuthLevel.parse("high"));
        assertEquals(AuthLevel.HIGH, AuthLevel.parse("  HIGH  "));
        assertEquals(AuthLevel.LOW, AuthLevel.parse(null));
        assertEquals(AuthLevel.LOW, AuthLevel.parse("  "));
    }

    @Test
    void parseRejectsUnknownValue() {
        assertThrows(ApiException.class, () -> AuthLevel.parse("SUPER"));
    }

    @Test
    void parseOrNullReturnsNullForUnknown() {
        assertNull(AuthLevel.parseOrNull("SUPER"));
        assertNull(AuthLevel.parseOrNull(null));
        assertEquals(AuthLevel.MEDIUM, AuthLevel.parseOrNull("medium"));
    }

    @Test
    void atLeastComparesRank() {
        assertTrue(AuthLevel.HIGH.atLeast(AuthLevel.MEDIUM));
        assertTrue(AuthLevel.MEDIUM.atLeast(AuthLevel.MEDIUM));
        assertFalse(AuthLevel.LOW.atLeast(AuthLevel.HIGH));
        // null 视为无要求
        assertTrue(AuthLevel.LOW.atLeast(null));
    }

    @Test
    void ascendingReturnsChainFromLow() {
        assertEquals(List.of(AuthLevel.LOW, AuthLevel.MEDIUM, AuthLevel.HIGH), AuthLevel.HIGH.ascending());
        assertEquals(List.of(AuthLevel.LOW), AuthLevel.LOW.ascending());
    }
}
