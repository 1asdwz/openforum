package com.openforum.dto.auth;

import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;

public record RegisterRequest(
        @NotBlank(message = "用户名不能为空")
        @Size(min = 3, max = 50, message = "长度需在 3-50 之间")
        String username,

        @NotBlank(message = "邮箱不能为空")
        @Email(message = "格式不正确")
        String email,

        @NotBlank(message = "密码不能为空")
        @Size(min = 6, max = 64, message = "长度需在 6-64 之间")
        String password) {
}
