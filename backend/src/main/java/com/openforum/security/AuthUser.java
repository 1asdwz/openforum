package com.openforum.security;

public record AuthUser(Long id, String username, String role) {

    public boolean isAdmin() {
        return "ADMIN".equals(role);
    }
}
