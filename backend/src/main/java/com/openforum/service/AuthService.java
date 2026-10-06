package com.openforum.service;

import com.openforum.dto.auth.AuthResponse;
import com.openforum.dto.auth.LoginRequest;
import com.openforum.dto.auth.RegisterRequest;
import com.openforum.entity.User;
import com.openforum.entity.enums.UserRole;
import com.openforum.exception.ApiException;
import com.openforum.repository.UserRepository;
import com.openforum.security.JwtTokenProvider;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class AuthService {

    private final UserRepository userRepository;
    private final PasswordEncoder passwordEncoder;
    private final JwtTokenProvider tokenProvider;
    private final UserService userService;

    public AuthService(UserRepository userRepository, PasswordEncoder passwordEncoder,
                       JwtTokenProvider tokenProvider, UserService userService) {
        this.userRepository = userRepository;
        this.passwordEncoder = passwordEncoder;
        this.tokenProvider = tokenProvider;
        this.userService = userService;
    }

    @Transactional
    public AuthResponse register(RegisterRequest request) {
        if (userRepository.existsByUsername(request.username())) {
            throw ApiException.badRequest("用户名已被占用");
        }
        if (request.email() != null && userRepository.existsByEmail(request.email())) {
            throw ApiException.badRequest("邮箱已被占用");
        }
        User user = new User();
        user.setUsername(request.username());
        user.setEmail(request.email());
        user.setPassword(passwordEncoder.encode(request.password()));
        user.setRole(UserRole.USER);
        user.setActive(true);
        userRepository.save(user);
        return new AuthResponse(tokenProvider.createToken(user), userService.me(user.getId()));
    }

    public AuthResponse login(LoginRequest request) {
        User user = userRepository
                .findByUsernameOrEmail(request.usernameOrEmail(), request.usernameOrEmail())
                .orElseThrow(() -> ApiException.unauthorized("账号或密码错误"));
        if (!user.isActive()) {
            throw ApiException.forbidden("账号已被禁用");
        }
        if (!passwordEncoder.matches(request.password(), user.getPassword())) {
            throw ApiException.unauthorized("账号或密码错误");
        }
        return new AuthResponse(tokenProvider.createToken(user), userService.me(user.getId()));
    }
}
