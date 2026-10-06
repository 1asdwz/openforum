package com.openforum.security;

import com.openforum.exception.ApiException;
import org.springframework.security.core.Authentication;
import org.springframework.security.core.context.SecurityContextHolder;

import java.util.Optional;

public final class SecurityUtils {

    private SecurityUtils() {
    }

    public static Optional<AuthUser> currentUser() {
        Authentication authentication = SecurityContextHolder.getContext().getAuthentication();
        if (authentication != null && authentication.getPrincipal() instanceof AuthUser authUser) {
            return Optional.of(authUser);
        }
        return Optional.empty();
    }

    public static AuthUser requireUser() {
        return currentUser().orElseThrow(() -> ApiException.unauthorized("请先登录"));
    }

    public static AuthUser requireAdmin() {
        AuthUser user = requireUser();
        if (!user.isAdmin()) {
            throw ApiException.forbidden("需要管理员权限");
        }
        return user;
    }
}
