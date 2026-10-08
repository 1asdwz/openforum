package com.openforum.dto.auth;

import jakarta.validation.constraints.NotBlank;

public record GoogleLoginRequest(
        @NotBlank(message = "Google 登录凭证不能为空")
        String credential) {
}
