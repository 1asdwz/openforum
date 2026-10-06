package com.openforum.dto.auth;

import jakarta.validation.constraints.NotBlank;

public record LoginRequest(
        @NotBlank(message = "账号不能为空")
        String usernameOrEmail,

        @NotBlank(message = "密码不能为空")
        String password) {
}
