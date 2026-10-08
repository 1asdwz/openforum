package com.openforum.controller;

import com.openforum.common.ApiResponse;
import com.openforum.dto.auth.AuthResponse;
import com.openforum.dto.auth.GoogleLoginRequest;
import com.openforum.dto.auth.LoginRequest;
import com.openforum.dto.auth.RegisterRequest;
import com.openforum.dto.user.UserDTO;
import com.openforum.security.SecurityUtils;
import com.openforum.service.AuthService;
import com.openforum.service.UserService;
import jakarta.validation.Valid;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/auth")
public class AuthController {

    private final AuthService authService;
    private final UserService userService;

    public AuthController(AuthService authService, UserService userService) {
        this.authService = authService;
        this.userService = userService;
    }

    @PostMapping("/register")
    public ApiResponse<AuthResponse> register(@Valid @RequestBody RegisterRequest request) {
        return ApiResponse.ok(authService.register(request));
    }

    @PostMapping("/login")
    public ApiResponse<AuthResponse> login(@Valid @RequestBody LoginRequest request) {
        return ApiResponse.ok(authService.login(request));
    }

    @PostMapping("/google")
    public ApiResponse<AuthResponse> google(@Valid @RequestBody GoogleLoginRequest request) {
        return ApiResponse.ok(authService.loginWithGoogle(request.credential()));
    }

    @GetMapping("/me")
    public ApiResponse<UserDTO> me() {
        return ApiResponse.ok(userService.me(SecurityUtils.requireUser().id()));
    }
}
